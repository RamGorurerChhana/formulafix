use crate::constants::DB_NAME;
use mongodb::{options::ClientOptions, Client, Database};
use std::error::Error;

pub async fn get_db_connection(db_url: &str) -> Result<Database, Box<dyn Error>> {
    let client_options = ClientOptions::parse(db_url).await?;
    let client = Client::with_options(client_options)?;
    let db = client.database(DB_NAME);
    Ok(db)
}

type UnitResult = Result<(), Box<dyn Error>>;
pub fn handle_error(r: UnitResult) -> bool {
    if r.is_err() {
        println!("{:?}", r);
        return false;
    }

    true
}
