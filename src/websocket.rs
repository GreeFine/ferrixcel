use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use actix::prelude::*;

use actix_web_actors::ws::{self, CloseReason};
use log::info;
use serde::Serialize;

use crate::{
    grid,
    models::{ActionKind, Broadcast, Position},
};

type User = Addr<MyWs>;
type Users = Arc<RwLock<HashMap<String, User>>>;
type Selections = Arc<RwLock<HashMap<Position, String>>>;

lazy_static! {
    pub static ref USERS: Users = Arc::new(RwLock::new(HashMap::new()));
    pub static ref SELECTIONS: Selections = Arc::new(RwLock::new(HashMap::new()));
}

pub struct MyWs {
    pub username: String,
    pub ip: String,
    pub logged: bool,
}

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        info!("User connection: [{}] -> {}", self.ip, self.username);
        let mut users = USERS.write().expect("unable to get lock on users");
        if users.get(&self.username).is_some() {
            info!(
                "Disconnecting new connection, username already taken {}",
                self.username
            );
            ctx.close(Some(CloseReason {
                code: ws::CloseCode::Invalid,
                description: Some("username already taken".to_string()),
            }));
            ctx.stop();
            return;
        }

        users.insert(self.username.clone(), ctx.address());
        self.logged = true;
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        if !self.logged {
            return;
        }
        info!("User disconnecting: [{}] -> {}", self.ip, self.username);
        {
            let mut selections = SELECTIONS
                .write()
                .expect("unable to get lock on selections");
            selections.retain(|_pos, username| username == &self.username);
        }
        {
            let mut users = USERS.write().expect("unable to get lock on users");
            users.remove(&self.username);
        }
    }
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct SendMessage(pub String);

impl MyWs {
    fn broadcast<T: Serialize>(&self, action: T) {
        let payload = SendMessage(
            serde_json::to_string(&Broadcast {
                who: &self.username,
                action,
            })
            .unwrap(),
        );
        let users = USERS.write().expect("unable to get lock on users");
        for (_username, user) in users.iter() {
            user.do_send(payload.clone());
        }
    }
}

impl actix::Handler<SendMessage> for MyWs {
    type Result = ();
    fn handle(&mut self, msg: SendMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0)
    }
}

#[derive(Debug, Serialize)]
struct ErrorMessages<'a> {
    error_code: u16,
    error: &'a str,
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        if let Ok(ws::Message::Text(text)) = msg {
            if let Ok(action) = serde_json::from_str::<ActionKind>(&text) {
                let username = self.username.clone();
                match action {
                    ActionKind::NewGridValue(grid_value) => {
                        {
                            let selections = SELECTIONS.read().expect("read in selections");
                            if selections.get(&grid_value.position) != Some(&username) {
                                ctx.address().do_send(SendMessage(
                                    serde_json::to_string(&ErrorMessages {
                                        error_code: 400,
                                        error: "This grid position is not locked by you.",
                                    })
                                    .unwrap(),
                                ));
                                return;
                            }
                        }
                        let grid_value_copy = grid_value.clone();
                        let future = async move {
                            grid::create_value(grid_value_copy, username)
                                .await
                                .expect("Unable to create value in database");
                        };
                        future.into_actor(self).spawn(ctx);
                        self.broadcast(grid_value);
                    }
                    ActionKind::Select(position) => {
                        {
                            let mut selections = SELECTIONS.write().expect("write in selections");
                            selections.insert(position.clone(), username);
                        }

                        self.broadcast(position);
                    }
                }
            } else {
                ctx.text(r#"{"error":"unable to parse value"}"#);
            };
        } else {
            info!("Received unhandled query from stream: {msg:#?}")
        }
    }
}
