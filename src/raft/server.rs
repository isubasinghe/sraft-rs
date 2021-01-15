use actix::prelude::*;

use actix_rt::net::TcpStream;
use actix_server::Server;
use actix_service::pipeline_factory;
use std::time::Duration;
use uuid::Uuid;

use crate::raft::state;

#[derive(Message)]
#[rtype(result="()")]
pub struct Ping;


#[derive(Message)]
#[rtype(result="()")]
pub struct Timeout;

pub enum Msg {
    Msg,
    HeartBeat
}

pub struct TcpClient {
    stream: TcpStream,
    timer: Option<SpawnHandle>
}

impl TcpClient {
    fn new(stream: TcpStream) -> TcpClient {
        TcpClient{stream, timer: None}
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

async fn start() {
    let addr = ("127.0.0.1", 8080);
    // let raft = state::Raft::default(Uuid::new())
    Server::build()
        .bind("raft", addr, move || {
            pipeline_factory(move |mut stream: TcpStream| {

                async move {
                    
                    let tcp_client = TcpClient::new(stream);

                    // send data down service pipeline
                    let res: Result<(), ()> = Ok(());

                    res
                }
            })
        }).map_err(|err| {

        });
}