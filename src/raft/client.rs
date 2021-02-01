use actix::prelude::*;
use futures::executor::block_on;
use std::time::Duration;
use tracing::{info, warn, error};
use crate::raft::state::Raft;
use crate::raft::messages::*;
use crate::raft::transformers::*;
use crate::raft::raftservice::raft_service_client::RaftServiceClient;
use crate::raft::raftservice::{
    BroadCastMsgData,
};

pub struct Client{
    addr: String,
    client: Option<RaftServiceClient<tonic::transport::Channel>>,
    raft: Addr<Raft>
}

impl Client {
    pub fn new(addr: String, raft: Addr<Raft>) -> Client {
        Client{addr, client: None, raft}
    }
}

impl Actor for Client {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        let addr = self.addr.clone();
        let raft = self.raft.clone();
        let fut = async move {
            loop {
                info!("CLIENT: Attempting connect");
                match RaftServiceClient::connect(addr.clone()).await {
                    Ok(mut client) => {
                        info!("CLIENT: ESTABLISHED CONNECTION");
                        let uuid = client.get_uuid(()).await.unwrap();
                        let uuid = uuid__to_uuid(uuid.into_inner()).unwrap();
                        raft.do_send(NotifyUUID{node_id: uuid, addr: ctx.address().recipient()});
                        return client;
                    },
                    Err(e) => {
                        error!("CLIENT: COULD NOT ESTABLISH CONNECTION err:{0}", e);
                    }
                }
                warn!("CLIENT: SLEEPING FOR 10 seconds");
                actix::clock::sleep(Duration::from_secs(10)).await;
            }
        };
        self.client = Some(block_on(fut));
    }
}

impl Handler<NodeMsgs> for Client {
    type Result = ();

    fn handle(&mut self, msg: NodeMsgs, _: &mut Context<Self>) {
        match msg {
            NodeMsgs::BroadcastMsg(msg) => {
                let msg = BroadCastMsgData{data: (*msg.data).clone()};
                let client = self.client.clone();
                let fut = async move {
                    client.unwrap().broad_cast_msg(msg).await
                };

                let _ = block_on(fut);
            },
            NodeMsgs::LogRequest(msg) => {

                let msg = lreq_to_lreq_(msg);
                let raft = self.raft.clone();
                let client = self.client.clone();
                let fut = async move {
                    client.unwrap().do_log_request(msg).await
                };

                let res = match block_on(fut) {
                    Ok(res) => res, 
                    Err(_) => return
                };
                let res = match log_resp__to_log_resp(res.into_inner()) {
                    Ok(res) => res, 
                    Err(_)  => return
                };
                raft.do_send(res);
            },
            NodeMsgs::VoteRequest(msg) => {
                let msg = vote_req_to_vote_req_(msg);
                let raft = self.raft.clone();
                let client = self.client.clone();
                let fut = async move {
                    client.unwrap().do_vote_request(msg).await
                };
                let res = match block_on(fut) {
                    Ok(res) => res.into_inner(),
                    Err(_) => return
                };
                let res = match vote_resp__to_vote_resp(res) {
                    Ok(res) => res, 
                    Err(_) => return
                };
                raft.do_send(res);
            }
        }
        

        ()
    }

}