// SELECT column_name, data_type
//   FROM information_schema.columns
//  WHERE table_name = 'camlytics_event';

// SELECT tablename
// FROM pg_catalog.pg_tables
// WHERE schemaname != 'pg_catalog' AND
//     schemaname != 'information_schema';

use serde::Serialize;
use sqlx::{postgres::PgPoolOptions, Postgres};

pub async fn list_tables() -> Result<Vec<String>, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect("postgres://postgres:secret@127.0.0.1:5432/pdv-lab")
        .await?;

    let table_names: Vec<String> = sqlx::query_as::<Postgres, (String,)>(
        "SELECT tablename
              FROM pg_catalog.pg_tables
              WHERE schemaname != 'pg_catalog' AND schemaname != 'information_schema';",
    )
    .fetch_all(&pool)
    .await?
    .into_iter()
    .map(|(val,)| val)
    // .map(|r: (String,)| r.0)
    .collect();

    Ok(table_names)
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ColumnInfo {
    ordinal_position: i32,
    column_name: String,
    data_type: String,
    column_default: Option<String>,
    is_primary_key: bool,
}

pub async fn list_columns(table_name: &str) -> Result<Vec<ColumnInfo>, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect("postgres://postgres:secret@127.0.0.1:5432/pdv-lab")
        .await?;

    let mut column_names: Vec<ColumnInfo> = sqlx::query_as::<_, ColumnInfo>(
        "SELECT ordinal_position, column_name, data_type, column_default, false as is_primary_key FROM information_schema.columns WHERE table_name = $1 ORDER BY ordinal_position;",
    )
    .bind(table_name)
    .fetch_all(&pool).await?;

    let primary_keys: (String, String) = sqlx::query_as(
        "SELECT a.attname, format_type(a.atttypid, a.atttypmod) AS data_type
      FROM   pg_index i
      JOIN   pg_attribute a ON a.attrelid = i.indrelid
      AND a.attnum = ANY(i.indkey)
      WHERE  i.indrelid = $1::regclass
      AND    i.indisprimary;",
    )
    .bind(table_name)
    .fetch_one(&pool)
    .await?;

    column_names
        .iter_mut()
        .find(|c| c.column_name == primary_keys.0)
        .expect("unable to find primary key in columns")
        .is_primary_key = true;

    Ok(column_names)
}

#[actix_web::test]
async fn test_table_column_listing() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info,sqlx=debug"));

    let tables_name = list_tables().await.unwrap();
    dbg!(&tables_name);
    assert!(!tables_name.is_empty());

    let table_name = &tables_name[1];
    dbg!(format!("using table_name : {table_name}"));
    let columns = list_columns(table_name).await.unwrap();
    dbg!(&columns);
    assert!(!columns.is_empty())
}
