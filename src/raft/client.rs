use actix::prelude::*;
use futures::executor::block_on;
use crate::raft::messages::*;
use crate::raft::raftservice::raft_service_client::RaftServiceClient;
pub struct Client {
    addr: String,
    client: Option<RaftServiceClient<tonic::transport::Channel>>,
}

impl Actor for Client {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        let addr = self.addr.to_owned();
        let fut = async move {
            // TODO: FIX THIS ASAP
            let client = RaftServiceClient::connect(addr).await.unwrap();
            client
        };
        
        let client = block_on(fut);
        self.client = Some(client);
    }
}

impl Handler<NodeMsgs> for Client {
    type Result = ();

    fn handle(&mut self, msg: NodeMsgs, ctx: &mut Context<Self>) {
        
        match msg {
            NodeMsgs::BroadcastMsg(msg) => {

            },
            NodeMsgs::LogRequest(log_request) => {

            },
            NodeMsgs::VoteRequest(vote_req) => {
                
            }
        };

        ()
    }

}


impl Client {

}