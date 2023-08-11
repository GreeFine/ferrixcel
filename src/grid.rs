use chrono::Utc;
use mongodb::bson::{doc, to_bson};

use crate::database::create_handle;
use crate::models::{Date, GridValue, NewGridValue};

pub async fn create_value(
    new_box: NewGridValue,
    username: String,
) -> Result<(), mongodb::error::Error> {
    let handle = create_handle().await;
    let position = to_bson(&new_box.position).unwrap();
    let new = GridValue {
        timestamp: Date(Utc::now().naive_utc()),
        position: new_box.position,
        user: username,
        value: new_box.value,
    };
    let replaced = handle
        .find_one_and_replace(doc! { "position": position }, &new, None)
        .await?;
    if replaced.is_none() {
        handle.insert_one(new, None).await?;
    }
    Ok(())
}
