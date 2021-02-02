
use tonic::{transport::Server, Request, Response, Status};
use uuid::Uuid;
use actix::prelude::*;
use std::sync::Arc;
use tracing::{info};
use std::collections::{BTreeMap};
use tokio::runtime::Runtime;
use crate::raft::client::*;
use crate::raft::messages::*;
use crate::raft::state::{Raft, StateData};
use crate::raft::raftservice::raft_service_server::{RaftService, RaftServiceServer};
use crate::raft::raftservice::{
    LogResponse as LogResponse_, 
    LogRequest as LogRequest_, 
    VoteRequest as VoteRequest_, 
    VoteResponse as VoteResponse_,
    Uuid as Uuid_,
    Addrs,
    BroadCastMsgData
};
use crate::raft::transformers::*;
use crate::raft::application::*;

pub struct RaftServiceImpl {
    raft: Arc<Addr<Raft>>,
    uuid: Uuid_,
}

impl Actor for RaftServiceImpl {
    type Context = Context<Self>;
}

#[tonic::async_trait]
impl RaftService for RaftServiceImpl {
    async fn do_log_request(&self, request: Request<LogRequest_>) -> Result<Response<LogResponse_>, Status> {
        let x = request.into_inner();
        let leader_id = match opt_uuid__to_uuid(x.leader_id) {
            Ok(uuid) => uuid, 
            Err(e) => return Err(e)
        };
        let entries: Vec<(Arc<Vec<u8>>, u64)> = x.entries.iter().cloned().map(|e| {
            (Arc::new(e.data), e.term)
        }).collect();

        let req = LogRequest::new(leader_id, x.term, x.log_length, x.log_term, x.log_commit, entries);
        let x = match self.raft.send(req).await {
            Ok(e) => {
                let uuid_ = Uuid_ {data: e.node_id.to_simple().encode_lower(&mut Uuid::encode_buffer()).to_string()}; 
                LogResponse_ { node_id: Some(uuid_), current_term: e.current_term, ack: e.ack, success: e.success}
            },
            Err(e) => return Err(Status::internal(""))
        };
        Ok(Response::new(x))
    }

    async fn do_vote_request(&self, request: Request<VoteRequest_>) -> Result<Response<VoteResponse_>, Status> {
        let request = request.into_inner();
        let uuid = match opt_uuid__to_uuid(request.candidate_id) {
            Ok(uuid) => uuid, 
            Err(e) => return Err(e)
        };
        let vote_request = VoteRequest(
                                uuid, 
                                request.candidate_term, 
                                request.candidate_log_length, 
                                request.candidate_log_term);
        let resp = match self.raft.send(vote_request).await {
            Ok(resp) => resp,
            Err(_) => return Err(Status::invalid_argument(""))
        };
        
        let uuid = Uuid_ {data: resp.0.to_simple().encode_lower(&mut Uuid::encode_buffer()).to_string()}; 
        Ok(Response::new(
            VoteResponse_{voter_id: Some(uuid), term: resp.1, granted: resp.2}
        ))
    }

    async fn broad_cast_msg(&self, request: Request<BroadCastMsgData>) -> Result<Response<()>, Status> {
        let data = Arc::new(request.into_inner().data);
        let msg: BroadcastMsg = BroadcastMsg{data};
        self.raft.do_send(msg);
        Ok(Response::new(()))
    }

    async fn get_addrs(&self, request: Request<()>) -> Result<Response<Addrs>, Status> {
        
        unimplemented!();
    }

    async fn get_uuid(&self, request: Request<()>) -> Result<Response<Uuid_>, Status> {
        Ok(Response::new(self.uuid.clone()))
    }
}

pub struct ExecutionContext {

}

pub fn start(addr: String, addrs: Vec<String>, id: u128) -> Result<i32, Box<dyn std::error::Error>> {
    info!("SERVER: Starting server");

    let mut system = actix::System::new("test");
    
    let addr_server = addr.parse()?;
    let mut app = None;

    let uuid = uuid_to_uuid_(Uuid::from_u128(id));
    
    let raft = system.block_on(async move {
        let raft = Raft::create(|ctx| {
            let raft = ctx.address();
            let app_ = Application::new(raft.clone().recipient()).start();
            app = Some(app_.clone());
            for i in 0..(addrs.len()) {
                Client::new(addrs[i].clone(), raft.clone()).start();
            }
            let state_data = StateData::default();
            Raft{
                state_data,
                node_id: Uuid::from_u128(id), 
                timer_handle: None,
                nodes: BTreeMap::new(),
                replicator_handle: None,
                app: app_.recipient()
    
            }
        });
        raft
    });
    let greeter = RaftServiceImpl{raft: Arc::new(raft), uuid};

    println!("RUNNING");
    let rt = Runtime::new().unwrap();

    rt.spawn(async move {
        // tonic server
        let _ = Server::builder()
                .add_service(RaftServiceServer::new(greeter))
                .serve(addr_server)
                .await;
    });

    info!("SERVER: Started server");
    
    system.run();

    Ok(1)
}