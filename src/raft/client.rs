use actix::prelude::*;
use crate::raft::messages::*;

pub struct Client {
    addr: String
}

impl Actor for Client {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {

    }

    fn stopped(&mut self, ctx: &mut Context<Self>) {

    }
}

impl Handler<NodeMsgs> for Client {
    type Result = ();

    fn handle(&mut self, msg: NodeMsgs, ctx: &mut Context<Self>) {

        ()
    }

}


impl Client {

}