use tokio::net::TcpStream;
use serde::{Deserialize, Serialize};
use actix::prelude::*;
use tokio_util::codec::{FramedWrite, FramedRead, LengthDelimitedCodec};
use tokio_serde::formats::*;
use tokio_serde::*;
use serde_json::*;
use std::rc::Rc;
use futures::prelude::*;

#[derive(Deserialize, Serialize)]
pub struct Data {
    data: Vec<(Vec<u8>, u64)>
}

#[derive(Deserialize, Serialize)]
pub enum Msg {
    Msg(Data),
    Ping, 
    Pong
}


#[derive(Message)]
#[rtype(result="()")]
pub struct NewHost(String);

pub struct Client {
    client: Framed<FramedWrite<TcpStream, LengthDelimitedCodec>, Msg, Msg, Bincode<Msg, Msg>>,
    manager: Addr<ClientManager>
}

impl Client {
    fn new(client: Framed<FramedWrite<TcpStream, LengthDelimitedCodec>, Msg, Msg, Bincode<Msg, Msg>>, manager: Addr<ClientManager>) -> Client {
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
                    let stream1 = stream.into_std().unwrap();
                    let stream2 = stream1.try_clone().unwrap();
                    
                    let stream1_ = TcpStream::from_std(stream1).unwrap();
                    let stream2_ = TcpStream::from_std(stream2).unwrap();

                    let length_delimited = FramedWrite::new(stream1_, LengthDelimitedCodec::new());
                    let length_delimited_ = FramedRead::new(stream2_, LengthDelimitedCodec::new());
                    // Serialize frames with JSON
                    let mut serialized: Framed<FramedWrite<TcpStream, LengthDelimitedCodec>, Msg, Msg, Bincode<Msg, Msg>> =
                        SymmetricallyFramed::new(length_delimited, SymmetricalBincode::default());

                    tokio::spawn(async move {
                        let mut deserialized = tokio_serde::SymmetricallyFramed::new(
                            length_delimited_,
                            SymmetricalJson::<Msg>::default(),
                        );

                        while let Some(msg) = deserialized.try_next().await.unwrap() {
                            
                        }
                    });
                    
                },
                Err(e) => {

                }
            }
        });

        ()
    }
}

