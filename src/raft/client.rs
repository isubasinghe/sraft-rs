use actix::prelude::*;
use futures::executor::block_on;
use tokio::time::{sleep, Duration};
use crate::raft::state::Raft;
use crate::raft::messages::*;
use crate::raft::transformers::*;
use crate::raft::raftservice::raft_service_client::RaftServiceClient;
use crate::raft::raftservice::{
    BroadCastMsgData,
    Uuid as Uuid_
};

pub struct Client{
    addr: String,
    client: RaftServiceClient<tonic::transport::Channel>,
    raft: Addr<Raft>
}

impl Client {
    pub fn new(addr: String, raft: Addr<Raft>) -> Client {
        let addr = addr.to_owned();
        let addr1 = addr.to_owned();
        let fut = async move {
            loop {
                match RaftServiceClient::connect(addr.clone()).await {
                    Ok(client) => {
                        return client;
                    },
                    Err(e) => {
                        sleep(Duration::from_secs(10)).await;
                    }
                }

            }
        };
        let client = block_on(fut);

        Client{addr: addr1, client, raft}
    }

    pub async fn async_new(addr: String, raft: Addr<Raft>) -> Client {
        let addr = addr.to_owned();
        let addr1 = addr.to_owned();
        let client = loop {
            match RaftServiceClient::connect(addr.clone()).await {
                Ok(client) => {
                    break client;
                },
                Err(e) => {
                    sleep(Duration::from_secs(10)).await;
                }
            };
        };

        Client{addr: addr1, client, raft}
        
    }
}

impl Actor for Client {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {

    }
}

impl Handler<NodeMsgs> for Client {
    type Result = ();

    fn handle(&mut self, msg: NodeMsgs, ctx: &mut Context<Self>) {
        match msg {
            NodeMsgs::BroadcastMsg(msg) => {
                let msg = BroadCastMsgData{data: (*msg.data).clone()};
                let fut = async move {
                    self.client.broad_cast_msg(msg).await
                };

                let res = block_on(fut);
            },
            NodeMsgs::LogRequest(msg) => {
                let msg = lreq_to_lreq_(msg);
                let fut = async move {
                    self.client.do_log_request(msg).await
                };

                let res = block_on(fut);
            },
            NodeMsgs::VoteRequest(msg) => {
                
            }
        }
        

        ()
    }

}