use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use std::error::Error;
use actix::prelude::*;


#[derive(Message)]
#[rtype(result="()")]
pub struct NewHost(String);

pub struct Client {
    client: TcpStream,
    manager: Addr<ClientManager>
}

impl Client {
    fn new(client: TcpStream, manager: Addr<ClientManager>) -> Client {
        Client{client, manager}
    }
}

impl Actor for Client {
    type Context = Context<Self>;
}



pub struct ClientManager {

}


impl Actor for ClientManager {
    type Context = Context<Self>;
}

impl Handler<NewHost> for ClientManager {
    type Result = ();

    fn handle(&mut self, msg: NewHost, ctx: &mut Context<Self>) -> Self::Result {
        let addr = ctx.address();
        tokio::spawn(async move {
            let stream = TcpStream::connect("").await;
            match stream {
                Ok(stream) => {
                    let client = Client::new(stream, addr).start();
                    
                },
                Err(e) => {

                }
            }
        });

        ()
    }
}

