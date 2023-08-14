use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use strum_macros::AsRefStr;

#[derive(Debug, Deserialize, Serialize)]
pub struct Date(pub NaiveDateTime);

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
    pub timestamp: Date,
    pub position: Position,
    pub value: Option<String>,
    pub user: String,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize, Default)]
pub struct Position {
    column: u64,
    row: u64,
}

#[derive(Debug, Serialize)]
pub struct Broadcast<'a, T: Serialize> {
    pub who: &'a str,
    pub kind: &'a str,
    pub payload: T,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NewGridValue {
    pub position: Position,
    pub value: Option<String>,
}

#[derive(Debug, Clone, AsRefStr, Deserialize, Serialize)]
pub enum ActionKind {
    NewGridValue(NewGridValue),
    /// Used to broadcast deselected positions
    Select(Vec<Position>),
    /// Used only by the server to broadcast deselected positions
    #[serde(skip_serializing)]
    Deselect(Vec<Position>),
}
