#![feature(map_try_insert)]
#[warn(unused_extern_crates)]
#[macro_use]
extern crate lazy_static;
mod database;
mod grid;
mod models;
mod websocket;

use actix_cors::Cors;
use actix_web::{
    error, get, middleware::Logger, web, App, Error, HttpRequest, HttpResponse, HttpServer,
    Responder, Result,
};
use actix_web_actors::ws;
use env_logger::Env;

use crate::{database::get_grid, websocket::MyWs};

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
            username: path.into_inner().0,
            ip,
            logged: false,
        },
        &req,
        stream,
    )
}

async fn index() -> Result<impl Responder> {
    let resp = get_grid()
        .await
        .map_err(|_| error::ErrorInternalServerError("unable to get canvas"))?;
    Ok(web::Json(resp))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    HttpServer::new(move || {
        let cors = Cors::default().allowed_origin_fn(|_, _req_head| true);
        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            .service(ws_start)
            .route("/", web::get().to(index))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
