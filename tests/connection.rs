use formulafix::get_db_connection;
use std::env;

#[tokio::test]
async fn check_db() {
    let url = env::var("MONGODB_URL").unwrap();
    let db = get_db_connection(&url).await.unwrap();
    assert_eq!(db.name(), formulafix::DB_NAME);
}
