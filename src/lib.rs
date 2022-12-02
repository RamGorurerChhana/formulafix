mod arguments;
mod backup;
mod connection;
mod constants;
mod workflow;

pub use arguments::get_args;
pub use backup::take_backup;
pub use connection::get_db_connection;
pub use constants::*;
pub use workflow::discard_workflow;
pub use workflow::pull_back_workflow;

#[cfg(test)]
mod tests {
    use mongodb::bson::doc;
    use mongodb::bson::Document;

    use super::arguments::Arguments;
    use super::*;

    const DISCARD_WORKFLOW_ID: u32 = 9177;
    const DISCARD_FLOW_TYPE_ID: Option<u8> = Some(5);

    #[tokio::test]
    async fn check_discard() {
        let url = std::env::var("MONGODB_URL").unwrap();
        let db = get_db_connection(&url).await.unwrap();
        let args = Arguments::new(
            url.clone(),
            true,
            false,
            None,
            DISCARD_WORKFLOW_ID,
            DISCARD_FLOW_TYPE_ID,
            None,
        );
        let r = discard_workflow(&args, &db).await;
        assert!(!r.is_err());

        let wfm = db.collection::<Document>(WORK_FLOW_MASTER_COLLECTION);
        let f = doc! {"workFlowId": DISCARD_WORKFLOW_ID};
        let r = wfm.find_one(f.clone(), None).await.unwrap();
        assert!(r.is_none());

        let wfh = db.collection::<Document>(WORK_FLOW_HIST_COLLECTION);
        let r = wfh.find_one(f.clone(), None).await.unwrap();
        assert!(r.is_some());
        let status = r.unwrap().get_str("workFlowStatus").unwrap().to_string();
        assert_eq!(status, "Add Packcode Discarded".to_string());

        let wi = db.collection::<Document>(WORK_ITEMS_COLLECTION);
        let count = wi.count_documents(f.clone(), None).await.unwrap();
        assert_eq!(count, 0);
    }
}
