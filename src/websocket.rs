use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use actix::prelude::*;

use actix_web_actors::ws;
use log::{debug, info};
use mongodb::bson::Uuid;
use serde::Serialize;

use crate::{
    grid,
    models::{ActionKind, Broadcast, Position},
};

type Users = Arc<RwLock<HashMap<Uuid, (Addr<MyWs>, MyWs)>>>;
type Selections = Arc<RwLock<HashMap<Position, String>>>;

lazy_static! {
    pub static ref USERS: Users = Arc::new(RwLock::new(HashMap::new()));
    pub static ref SELECTIONS: Selections = Arc::new(RwLock::new(HashMap::new()));
}

#[derive(Clone)]
pub struct MyWs {
    pub uuid: Uuid,
    pub username: String,
    pub ip: String,
    pub table: Option<String>,
}

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        info!("User connection: [{}] -> {}", self.ip, self.username);
        let mut users = USERS.write().expect("unable to get lock on users");

        users.insert(self.uuid, (ctx.address(), self.clone()));
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        info!("User disconnecting: [{}] -> {}", self.ip, self.username);
        {
            let mut selections = SELECTIONS
                .write()
                .expect("unable to get lock on selections");
            selections.retain(|_pos, username| username == &self.username);
        }
        {
            let mut users = USERS.write().expect("unable to get lock on users");
            users.remove(&self.uuid);
        }
    }
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct SendMessage(pub String);

impl ActionKind {
    fn get_action_payload(&self) -> serde_json::Value {
        match self {
            ActionKind::NewGridValue(x) => serde_json::to_value(x).unwrap(),
            ActionKind::Select(x) => serde_json::to_value(x).unwrap(),
        }
    }
}

impl MyWs {
    fn broadcast(&self, action: ActionKind) {
        let payload = SendMessage(
            serde_json::to_string(&Broadcast {
                who: &self.username,
                kind: action.as_ref(),
                payload: action.get_action_payload(),
            })
            .unwrap(),
        );
        let users = USERS.write().expect("unable to get lock on users");
        for (_username, user) in users.iter() {
            user.0.do_send(payload.clone());
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
                info!("{username} -> {action:#?}");
                match action.clone() {
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
                        self.broadcast(action);
                    }
                    ActionKind::Select(position) => {
                        {
                            let mut selections =
                                dbg!(SELECTIONS.write().expect("write in selections"));
                            if let Some(position) = position {
                                if selections.contains_key(dbg!(&position)) {
                                    ctx.address().do_send(SendMessage(
                                        serde_json::to_string(&ErrorMessages {
                                            error_code: 400,
                                            error: "This grid position is already locked.",
                                        })
                                        .unwrap(),
                                    ));
                                    return;
                                }

                                selections.retain(|_pos, username| {
                                    dbg!(username) == dbg!(&self.username)
                                });
                                selections.insert(position.clone(), username);
                            } else {
                                selections.retain(|_pos, username| username == &self.username);
                            }
                        }
                        self.broadcast(action);
                    }
                }
            } else {
                debug!("Unable to parse action from user {}: {text}", self.username);
                ctx.text(
                    serde_json::to_string(&ErrorMessages {
                        error_code: 400,
                        error: "Unable to parse value.",
                    })
                    .unwrap(),
                );
            };
        } else {
            info!("Received unhandled query from stream: {msg:#?}")
        }
    }
}
