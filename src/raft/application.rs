use actix::prelude::*;
use std::sync::Arc;
use kv::*;
use serde::{Serialize, Deserialize};
use crate::raft::messages::*;

#[derive(Message, Serialize, Deserialize, Debug)]
#[rtype(result="ReturnVal")]
pub enum ApplicationActions {
    Insert(String, String),
    Get(String),
    Delete(String),
    Update(String, String)
}

#[derive(MessageResponse, Debug)]
pub enum ReturnVal {
    GetValue(Option<Vec<u8>>),
    Deleted,
    Updated,
    Inserted
}

pub struct Application {
    raft: Recipient<BroadcastMsg>,
    store: Bucket<'static, Raw, Raw>
}

impl Application {
    pub fn new(raft: Recipient<BroadcastMsg>) -> Application {
        let cfg = Config::new("./test/example1");
        let store = Store::new(cfg).unwrap();
        let bucket = store.bucket::<Raw, Raw>(None).unwrap();
        Application{raft, store: bucket}
    }
}

impl Actor for Application {
    type Context = Context<Self>;
}

impl Handler<ApplicationActions> for Application {
    type Result = ReturnVal;

    fn handle(&mut self, msg: ApplicationActions, ctx: &mut Context<Self>) -> Self::Result {
        let data = Arc::new(bincode::serialize(&msg).unwrap());

        match msg {
            ApplicationActions::Insert(k, v) => {
                self.store.set(k.as_bytes(), v.as_bytes()).unwrap();
                self.raft.do_send(BroadcastMsg{data}).unwrap();
                ReturnVal::Inserted
            },
            ApplicationActions::Delete(k) => {
                ReturnVal::Deleted
            },
            ApplicationActions::Get(k) => {
                let data = self.store.get(k.as_bytes()).unwrap().map(|v| {
                    v.to_vec()
                });
                ReturnVal::GetValue(data)
            },
            ApplicationActions::Update(k, v) => {
                ReturnVal::Updated
            }
        }
    }
}

impl Handler<AppMsg> for Application {
    type Result = ();

    fn handle(&mut self, msg: AppMsg, ctx: &mut Context<Self>) -> Self::Result {
        let msg: ApplicationActions = bincode::deserialize(&*msg.data).unwrap();
        match msg {
            ApplicationActions::Insert(k, v) => {
                self.store.set(k.as_bytes(), v.as_bytes()).unwrap();
            },
            ApplicationActions::Get(_) => {

            }
            ApplicationActions::Delete(k) => {

            },
            ApplicationActions::Update(k, v) => {
                
            }
        };
    }
}