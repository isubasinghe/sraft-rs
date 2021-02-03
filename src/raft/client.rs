use actix::prelude::*;
use futures::executor::block_on;
use std::time::Duration;
use tracing::{info, warn, error};
use std::sync::Arc;
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
    raft: Addr<Raft>,
    handle: Arc<tokio::runtime::Runtime>,
}

impl Client {
    pub fn new(addr: String, raft: Addr<Raft>, handle: Arc<tokio::runtime::Runtime>) -> Client {
        Client{addr, client: None, raft, handle}
    }
}

#[derive(Message)]
#[rtype(result="()")]
struct AddClient {
    client: RaftServiceClient<tonic::transport::Channel>,
    notify_msg: NotifyUUID,
}

impl Actor for Client {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        let addr = self.addr.clone();
        let me = ctx.address();
        let rt = self.handle.clone();
        let recipient = me.clone().recipient();
        let fut = async move {
            loop {
                info!("CLIENT: Attempting connect");
                match rt.block_on(RaftServiceClient::connect(format!("http://{0}", addr.clone()))) {
                    Ok(mut client) => {
                        info!("CLIENT: ESTABLISHED CONNECTION");
                        let uuid = match client.get_uuid(()).await {
                            Ok(uuid) => uuid, 
                            Err(e) => {
                                error!("CLIENT: UNABLE TO RETRIEVE err:{0}", e);
                                continue;
                            }
                        };
                        let uuid = uuid__to_uuid(uuid.into_inner()).unwrap();
                        me.do_send(AddClient{client, notify_msg: NotifyUUID{node_id: uuid, addr: recipient}});
                        break;
                    },
                    Err(e) => {
                        error!("CLIENT: COULD NOT ESTABLISH CONNECTION err:{0}", e);
                    }
                }
                warn!("CLIENT: SLEEPING FOR 10 seconds");
                actix::clock::delay_for(Duration::from_secs(10)).await;
            }
        };
        actix::spawn(fut);

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

                let _ = self.handle.block_on(fut);
            },
            NodeMsgs::LogRequest(msg) => {

                let msg = lreq_to_lreq_(msg);
                let raft = self.raft.clone();
                let client = self.client.clone();
                let fut = async move {
                    client.unwrap().do_log_request(msg).await
                };

                let res = match self.handle.block_on(fut) {
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
                let res = match self.handle.block_on(fut) {
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

impl Handler<AddClient> for Client {

    type Result = ();

    fn handle(&mut self, msg: AddClient, ctx: &mut Context<Self>) -> Self::Result {
        self.client = Some(msg.client);
        self.raft.do_send(msg.notify_msg);
        ()
    }
}