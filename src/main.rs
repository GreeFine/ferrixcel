#![feature(map_try_insert)]
#[warn(unused_extern_crates)]
#[macro_use]
extern crate lazy_static;
mod database;
mod grid;
mod introspection;
mod models;
mod websocket;

use actix_cors::Cors;
use actix_web::{
    error, get, middleware::Logger, web, App, Error, HttpRequest, HttpResponse, HttpServer,
    Responder, Result,
};
use actix_web_actors::ws;
use env_logger::Env;
use introspection::list_columns;
use mongodb::bson::Uuid;

use crate::{database::get_grid, introspection::list_tables, websocket::MyWs};

#[get("/ws/{username}")]
async fn ws_start(
    req: HttpRequest,
    path: web::Path<(String,)>,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    let ip = req
        .connection_info()
        .realip_remote_addr()
        .unwrap()
        .to_string();
    ws::start(
        MyWs {
            uuid: Uuid::new(),
            username: path.into_inner().0,
            ip,
            table: None,
        },
        &req,
        stream,
    )
}

#[get("/")]
async fn index() -> Result<impl Responder> {
    let resp = get_grid()
        .await
        .map_err(|_| error::ErrorInternalServerError("unable to get canvas"))?;
    Ok(web::Json(resp))
}

#[get("/tables")]
async fn get_tables() -> Result<impl Responder> {
    let tables = list_tables()
        .await
        .map_err(|_| error::ErrorInternalServerError("sqlx error unable to list tables"))?;
    Ok(web::Json(tables))
}

#[get("/table/{table_name}/columns")]
async fn get_columns(path: web::Path<(String,)>) -> Result<impl Responder> {
    let table_name = path.into_inner().0;
    let columns = list_columns(&table_name)
        .await
        .map_err(|_| error::ErrorInternalServerError("sqlx error unable to list columns"))?;
    Ok(web::Json(columns))
}

#[get("/ws/table/{table_name}/{username}")]
async fn ws_start_table(
    req: HttpRequest,
    stream: web::Payload,
    path: web::Path<(String, String)>,
) -> Result<impl Responder> {
    let (table_name, username) = path.into_inner();
    let ip = req
        .connection_info()
        .realip_remote_addr()
        .unwrap()
        .to_string();
    ws::start(
        MyWs {
            uuid: Uuid::new(),
            username,
            ip,
            table: Some(table_name),
        },
        &req,
        stream,
    )
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info,ferrixcel=debug,sqlx=debug"));

    HttpServer::new(move || {
        let cors = Cors::default().allowed_origin_fn(|_, _req_head| true);
        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            .service(ws_start)
            .service(index)
            .service(get_tables)
            .service(get_columns)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
