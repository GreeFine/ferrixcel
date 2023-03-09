use std::env;

use chrono::{NaiveDate, NaiveDateTime, Utc};
use futures::StreamExt;
use mongodb::bson::{doc, to_bson};
use mongodb::{options::ClientOptions, Client};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Default)]
struct Position {
    x: u64,
    y: u64,
}

#[derive(Debug, Deserialize, Serialize)]
struct Date(NaiveDateTime);

impl Default for Date {
    fn default() -> Self {
        Date({
            NaiveDate::from_ymd_opt(2022, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .expect("invalid time")
        })
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct GridValue {
    timestamp: Date,
    position: Position,
    content: String,
    user: String,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct NewGridValue {
    position: Position,
    content: String,
}

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

pub async fn get_canvas() -> Result<Vec<GridValue>, mongodb::error::Error> {
    let handle = create_handle().await;
    let cursor = handle.find(doc! {}, None).await?;
    let res: Vec<GridValue> = cursor.map(|m| m.unwrap()).collect().await;
    Ok(res)
}

pub async fn create_grid_value(
    new_box: NewGridValue,
    username: String,
) -> Result<(), mongodb::error::Error> {
    let handle = create_handle().await;
    let position = to_bson(&new_box.position).unwrap();
    let new = GridValue {
        timestamp: Date(Utc::now().naive_utc()),
        position: new_box.position,
        user: username,
        content: new_box.content,
    };
    let replaced = handle
        .find_one_and_replace(doc! { "position": position }, &new, None)
        .await?;
    if replaced.is_none() {
        handle.insert_one(new, None).await?;
    }
    Ok(())
}
