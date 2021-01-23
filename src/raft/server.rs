
use tonic::{transport::Server, Request, Response, Status};
use uuid::Uuid;
use actix::prelude::*;
use std::sync::Arc;
use crate::raft::messages::*;
use crate::raft::state::Raft;
use crate::raft::raftservice::raft_service_server::{RaftService, RaftServiceServer};
use crate::raft::raftservice::{
    LogResponse as LogResponse_, 
    LogRequest as LogRequest_, 
    VoteRequest as VoteRequest_, 
    VoteResponse as VoteResponse_,
    Uuid as Uuid_,
    Addrs
};
use crate::raft::transformers::opt_uuid__to_uuid;

pub struct RaftServiceImpl {
    raft: Arc<Addr<Raft>>,
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

    async fn get_addrs(&self, request: Request<()>) -> Result<Response<Addrs>, Status> {
        
        unimplemented!();
    }

    async fn get_uuid(&self, request: Request<()>) -> Result<Response<Uuid_>, Status> {

        unimplemented!();
    }
}
async fn start() -> Result<(), Box<dyn std::error::Error>> {
    // let addr = "[::1]:50051".parse()?;

    // let raft = Arc::new(Raft::default(Uuid::new_v4()).start());
    // let greeter = RaftServiceImpl{raft};


    // Server::builder()
    //     .add_service(RaftServiceServer::new(greeter))
    //     .serve(addr)
    //     .await?;

    Ok(())
}