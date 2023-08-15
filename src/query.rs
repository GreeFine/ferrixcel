use std::ops::Deref;

use chrono::{Duration, NaiveDate};
use serde::Serialize;
use sqlx::{
    postgres::{PgPoolOptions, PgRow},
    types::Uuid,
    FromRow, Postgres, Row, ValueRef,
};

use crate::introspection::{self, ColumnInfo};

#[derive(Debug, Serialize)]
pub struct BytesRow {
    values: Vec<Option<Vec<u8>>>,
}

impl Deref for BytesRow {
    type Target = Vec<Option<Vec<u8>>>;

    fn deref(&self) -> &Self::Target {
        &self.values
    }
}

impl FromRow<'_, PgRow> for BytesRow {
    fn from_row(row: &PgRow) -> sqlx::Result<Self> {
        let row_size = row.len();
        let mut values = Vec::with_capacity(row_size);
        for index in 0..row_size {
            let value = row.try_get_raw(index)?;

            let value_parsed = if value.is_null() {
                None as Option<Vec<u8>>
            } else {
                match value.format() {
                    sqlx::postgres::PgValueFormat::Text => {
                        return Err(sqlx::Error::Decode("()".into()))
                    }
                    sqlx::postgres::PgValueFormat::Binary => {
                        Some(value.as_bytes().unwrap().to_vec())
                    }
                }
            };

            values.push(value_parsed);
        }
        Ok(Self { values })
    }
}

pub async fn query_table(table_name: &str) -> Result<Vec<Vec<serde_json::Value>>, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect("postgres://postgres:secret@127.0.0.1:5432/pdv-lab")
        .await?;

    let columns = introspection::list_columns(table_name).await?;

    let primary_key = columns
        .iter()
        .find_map(|c| c.is_primary_key.then_some(&c.column_name))
        .unwrap();
    let raw_rows: Vec<BytesRow> = sqlx::query_as::<Postgres, BytesRow>(&format!(
        "SELECT * FROM \"{table_name}\" ORDER BY $1 LIMIT 1000;"
    ))
    .bind(primary_key)
    .fetch_all(&pool)
    .await?;

    let rows = parse_rows(columns, raw_rows);
    Ok(rows)
}

fn parse_rows(columns: Vec<ColumnInfo>, raw_rows: Vec<BytesRow>) -> Vec<Vec<serde_json::Value>> {
    let mut values_parsed: Vec<Vec<serde_json::Value>> = Vec::new();
    for row in raw_rows {
        // Vec<(Option<Vec<u8>>, &ColumnInfo)>
        let row_parsed = row
            .values
            .into_iter()
            .zip(&columns)
            .map(|(row, info)| {
                if let Some(row_val) = row {
                    match &*info.data_type {
                        "integer" => {
                            serde_json::to_value(i32::from_be_bytes(row_val.try_into().unwrap()))
                                .unwrap()
                        }
                        "uuid" => {
                            serde_json::to_value(Uuid::from_bytes(row_val.try_into().unwrap()))
                                .unwrap()
                        }
                        "text" => {
                            serde_json::to_value(String::from_utf8(row_val).unwrap()).unwrap()
                        }
                        "bigint" => {
                            serde_json::to_value(i64::from_be_bytes(row_val.try_into().unwrap()))
                                .unwrap()
                        }
                        "timestamp without time zone" => {
                            let us = i64::from_be_bytes(row_val.try_into().unwrap());
                            let postgres_epoch_datetime = NaiveDate::from_ymd_opt(2000, 1, 1)
                                .expect("expected 2000-01-01 to be a valid NaiveDate")
                                .and_hms_opt(0, 0, 0)
                                .expect("expected 2000-01-01T00:00:00 to be a valid NaiveDateTime");

                            let datetime = postgres_epoch_datetime + Duration::microseconds(us);
                            serde_json::to_value(datetime).unwrap()
                        }
                        _ => serde_json::to_value("unexpected type").unwrap(),
                    }
                } else {
                    serde_json::Value::Null
                }
            })
            .collect();
        values_parsed.push(row_parsed);
    }
    values_parsed
}

#[actix_web::test]
async fn test_query_table() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info,sqlx=debug"));

    let res = query_table("camlytics_event").await.unwrap();
    dbg!(res);
}
