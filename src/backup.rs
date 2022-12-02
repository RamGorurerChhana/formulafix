use crate::arguments::Arguments;
use crate::connection::handle_error;
use crate::constants::*;
use futures::stream::TryStreamExt;
use mongodb::bson::doc;
use mongodb::bson::Document;
use mongodb::Database;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

pub async fn take_backup(args: &Arguments, db: &Database) {
    let bo_collection = match args.flow_type_id() {
        1 => FORMULA_DATA_COLLECTION,
        2 => CHANGE_FORMULA_DATA_COLLECTION,
        5 => CHANGE_PACK_CODE_DATA_COLLECTION,
        6 => CHANGE_PACK_CODE_DATA_COLLECTION,
        _ => unreachable!("Invalid flow_type_id"),
    };
    let collections = vec![
        WORK_FLOW_MASTER_COLLECTION.to_string(),
        bo_collection.to_string(),
        WORK_ITEMS_COLLECTION.to_string(),
        WORK_ITEMS_HIST_COLLECTION.to_string(),
        VIEW_WORK_FLOW_HISTORY_COLLECTION.to_string(),
        args.merge_collection(),
    ];
    let handles = collections
        .into_iter()
        .map(|c| {
            let db = db.clone();
            let w = args.work_flow_id();
            tokio::spawn(async move { handle_error(create_backup(&db, w, &c).await) })
        })
        .collect::<Vec<_>>();
    for handle in handles {
        let r = handle.await;
        if let Ok(false) = r {
            std::process::exit(1);
        }
    }
}

type RetType = Result<(), Box<dyn Error>>;
async fn create_backup(db: &Database, work_flow_id: u32, collection: &str) -> RetType {
    let fname = format!("{collection}_{work_flow_id}.json");
    println!("─➤ Creating backup file {fname}");
    let path = Path::new(fname.as_str());
    if path.exists() {
        println!("─➤ Remove existing file {fname}");
        fs::remove_file(path)?;
    }
    let mut file = File::create(fname)?;
    let f = doc! {"workFlowId": work_flow_id};
    let collection = db.collection::<Document>(collection);
    let mut cursor = collection.find(f, None).await?;
    let mut items = vec![];
    while let Some(item) = cursor.try_next().await? {
        items.push(item);
    }
    let result = serde_json::to_string(&items)?;
    file.write_all(result.as_bytes())?;
    Ok(())
}
