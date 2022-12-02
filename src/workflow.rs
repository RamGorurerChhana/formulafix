use crate::arguments::Arguments;
use crate::connection::handle_error;
use crate::constants::*;
use futures::stream::TryStreamExt;
use mongodb::bson::doc;
use mongodb::bson::Document;
use mongodb::Database;
use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

type RetType = Result<(), Box<dyn Error>>;
pub async fn discard_workflow(args: &Arguments, db: &Database) -> RetType {
    let wf_handle = {
        let db = db.clone();
        let wfid = args.work_flow_id();
        let ftid = args.flow_type_id();
        tokio::spawn(async move { handle_error(discard_from_wfm(&db, wfid, ftid).await) })
    };
    let wi_handle = {
        let db = db.clone();
        let wfid = args.work_flow_id();
        tokio::spawn(async move { handle_error(archive_work_items(&db, wfid).await) })
    };
    let _ = wf_handle.await;
    let _ = wi_handle.await;
    Ok(())
}

pub async fn pull_back_workflow(args: &Arguments, db: &Database) -> RetType {
    let wfid = args.work_flow_id();
    let ftid = args.flow_type_id() as i64;
    let stage = args.ret_stage().unwrap() as i64;
    let merge_coll = args.merge_collection();
    println!("─➤ Pull back workFlowId {wfid}, flowTypeId {ftid}, stage {stage}");
    let wfm = db.collection::<Document>(WORK_FLOW_MASTER_COLLECTION);
    let wi = db.collection::<Document>(WORK_ITEMS_COLLECTION);
    let wih = db.collection::<Document>(WORK_ITEMS_HIST_COLLECTION);
    let mc = db.collection::<Document>(merge_coll.as_str());
    let stage_filter = doc! {"workFlowId": wfid, "flowTypeId": ftid, "mileStoneId": stage};
    let item = wih.find_one(stage_filter, None).await?;
    if item.is_none() {
        println!("─➤ No item found in workItemsHist for stage {stage}");
        return Ok(());
    }
    let f = doc! {"workFlowId": wfid};
    #[rustfmt::skip]
    println!("─➤ Deleting all items from {} for workFlowId {wfid}", merge_coll.as_str());
    mc.delete_many(f.clone(), None).await?;
    println!("─➤ Deleting all items from workItems for workFlowId {wfid}");
    wi.delete_many(f.clone(), None).await?;
    let mut item = item.unwrap();
    item.remove("_id");
    let wfs = match item.get("workFlowStatus") {
        Some(s) => s.to_string(),
        None => panic!("workFlowStatus not found"),
    };
    let wfs = wfs.as_str().trim_matches('"');
    println!("─➤ Insert into workItems for workFlowId {wfid}");
    wi.insert_one(item, None).await?;
    println!("─➤ Updating status to pending in workItems {wfid}");
    let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let t = t.as_secs() as i64;
    let filter = doc! {"workFlowId": wfid, "users.status": "resolved"};
    let update = doc! {"$set":{"users.$.status": "pending", "users.$.updatedTimeUnix": t, "updatedTimeUnix": t}};
    wi.update_many(filter, update, None).await?;
    #[rustfmt::skip]
    println!("─➤ Updating status to {} in workFlowMaster {wfid}", wfs);
    let update = doc! {"$set":{"workFlowStatus": wfs, "updatedTimeUnix": t, "extraNote": "Returned by formulafix script"}};
    wfm.update_one(f.clone(), update, None).await?;
    Ok(())
}

async fn discard_from_wfm(db: &Database, work_flow_id: u32, flow_type_id: u8) -> RetType {
    println!("─➤ Discarding from workFlowMaster {work_flow_id}");
    let wfs = match flow_type_id {
        1 => "Formula Discarded",
        2 => "Formula Change Discarded",
        5 => "Add Packcode Discarded",
        6 => "Delete Packcode Discarded",
        _ => unreachable!(),
    };
    let wfm = db.collection::<Document>(WORK_FLOW_MASTER_COLLECTION);
    let f = doc! {"workFlowId": work_flow_id};
    let workflow = wfm.find_one(f.clone(), None).await?;
    if workflow.is_none() {
        return Ok(());
    }
    let mut workflow = workflow.unwrap();
    workflow.remove("_id");
    workflow.get_mut("workFlowStatus").and_then(|val| {
        *val = wfs.into();
        Some(())
    });
    workflow.get_mut("updatedTimeUnix").and_then(|val| {
        let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        *val = (t.as_secs() as i64).into();
        Some(())
    });
    let wfh = db.collection::<Document>(WORK_FLOW_HIST_COLLECTION);
    println!("─➤ Inserting into workFlowHist {work_flow_id}");
    wfh.insert_one(workflow, None).await?;
    println!("─➤ Deleting from workFlowMaster {work_flow_id}");
    wfm.delete_one(f, None).await?;
    Ok(())
}

async fn archive_work_items(db: &Database, work_flow_id: u32) -> RetType {
    println!("─➤ Archiving workItems {work_flow_id}");
    let wi = db.collection::<Document>(WORK_ITEMS_COLLECTION);
    let wih = db.collection::<Document>(WORK_ITEMS_HIST_COLLECTION);
    let f = doc! {"workFlowId": work_flow_id};
    let mut cursor = wi.find(f.clone(), None).await?;
    while let Some(mut item) = cursor.try_next().await? {
        item.remove("_id");
        println!("─➤ Inserting into workItemsHist {work_flow_id}");
        wih.insert_one(item, None).await?;
    }
    println!("─➤ Updating status to resolved in workItemsHist {work_flow_id}");
    let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let t = t.as_secs() as i64;
    let filter = doc! {"workFlowId": work_flow_id, "users.status": "pending"};
    let update = doc! {"$set":{"users.$.status": "resolved", "users.$.updatedTimeUnix": t, "updatedTimeUnix": t, "extraNote": "Updated by formulafix script"}};
    wih.update_many(filter, update, None).await?;
    println!("─➤ Deleting from workItems {work_flow_id}");
    wi.delete_many(f, None).await?;
    Ok(())
}
