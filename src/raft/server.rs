use std::time::Duration;
use std::error::Error;
use std::sync::Arc;
use uuid::Uuid;
use futures::SinkExt;
use tokio_stream::StreamExt;
use tokio::net::{TcpStream, TcpListener};
use serde::{Deserialize, Serialize};
use actix::prelude::*;

use crate::raft::state;


#[derive(Message)]
#[rtype(result="()")]
pub struct Ping;


#[derive(Message)]
#[rtype(result="()")]
pub struct Timeout;

#[derive(Serialize, Deserialize)]
pub enum Msg {
    Msg,
    HeartBeat
}

pub struct TcpClient {
    stream: TcpStream,
    timer: Option<SpawnHandle>,
    addr: Arc<Addr<state::Raft>>
}

impl TcpClient {
    fn new(stream: TcpStream, addr: Arc<Addr<state::Raft>>) -> TcpClient {
        TcpClient{stream, timer: None, addr}
    }
}


impl Actor for TcpClient {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        self.timer = Some(ctx.run_later(Duration::from_secs(1), |_, ctx| {
            ctx.address().do_send(Timeout);
        }));
    }
}

impl Handler<Ping> for TcpClient {
    type Result = ();

    fn handle(&mut self, msg: Ping, ctx: &mut Context<Self>) -> Self::Result {
        match self.timer {
            Some(handle) => {ctx.cancel_future(handle);}
            None => {}
        }
        self.timer = Some(ctx.run_later(Duration::from_secs(1), |_, ctx| {
            ctx.address().do_send(Timeout);
        }));
        ()
    }
}

impl Handler<Timeout> for TcpClient {
    type Result = ();
    fn handle(&mut self, msg: Timeout, ctx: &mut Context<Self>) -> Self::Result {

        ()
    }
}

pub struct TcpServer {

}

async fn start() -> Result<(), Box<dyn Error>>{
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    
    let raft = Arc::new(state::Raft::default(Uuid::new_v4()).start());
    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                let raft = raft.clone();
                tokio::spawn(async move {
                    let client = TcpClient::new(socket, raft).start();
                    
                });
            }
            Err(e) => {

            }
        }
    }

    Ok(())
}