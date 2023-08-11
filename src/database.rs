use std::env;

use futures::StreamExt;
use mongodb::{bson::doc, options::ClientOptions, Client};

use crate::models::GridValue;

pub async fn create_handle() -> mongodb::Collection<GridValue> {
    let mongo_uri = env::var("MONGO_URI")
        .unwrap_or_else(|_| "mongodb://root:example@localhost:27017".to_string());
    let client_options = ClientOptions::parse(mongo_uri)
        .await
        .expect("Unable to connect to the database");
    let client = Client::with_options(client_options).unwrap();
    let db = client.database("ferrixcel");

    db.collection::<GridValue>("canvas")
}

pub async fn get_grid() -> Result<Vec<GridValue>, mongodb::error::Error> {
    let handle = create_handle().await;
    let cursor = handle.find(doc! {}, None).await?;
    let res: Vec<GridValue> = cursor.map(|m| m.unwrap()).collect().await;
    Ok(res)
}
