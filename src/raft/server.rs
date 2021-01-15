use actix::prelude::*;

use actix_rt::net::{TcpStream};
use actix_server::Server;
use actix_service::pipeline_factory;
use std::time::Duration;
use uuid::Uuid;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use bytes::BytesMut;
use tokio_io::AsyncRead;
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

#[derive(Clone)]
pub struct Passable {
    addr: Arc<Addr<state::Raft>>
}

async fn start() {
    let addr = ("127.0.0.1", 8080);
    let raft =  Arc::new(state::Raft::default(Uuid::new_v4()).start());
    let x = Server::build()
        .bind("raft", addr, move || {
            let raft = Arc::clone(&raft);
            pipeline_factory( move |mut stream: TcpStream| {
                let raft = Arc::clone(&raft);

                async move {
                    let tcp_client = TcpClient::new(stream, raft).start();

                    let mut size: u64 = 0;
                    let mut buf = BytesMut::new();
                    
                    loop {
                        
                    }
                    // send data down service pipeline
                    let res: Result<(), ()> = Ok(());

                    res
                }
            })
        });
}