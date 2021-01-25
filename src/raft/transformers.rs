use uuid::Uuid;
use std::str::FromStr;
use std::sync::Arc;
use tonic::{Status};
use crate::raft::messages::*;
use crate::raft::raftservice::raft_service_server::{RaftService, RaftServiceServer};
use crate::raft::raftservice::{
    LogResponse as LogResponse_, 
    LogRequest as LogRequest_, 
    VoteRequest as VoteRequest_, 
    VoteResponse as VoteResponse_,
    Uuid as Uuid_,
    CommitEntry,
    Addrs
};

pub fn opt_uuid__to_uuid(uuid: Option<Uuid_>) -> Result<Uuid, Status> {
    let uuid = match uuid {
        Some(uuid) => Uuid::from_str(&uuid.data), 
        None => return Err(Status::invalid_argument(""))
    };
    match uuid {
        Ok(uuid) => Ok(uuid),
        Err(_) => Err(Status::invalid_argument(""))
    }
}

pub fn uuid_to_uuid_(uuid: Uuid) -> Uuid_ {
    Uuid_ {data: uuid.to_simple().encode_lower(&mut Uuid::encode_buffer()).to_string()}
}

pub fn log_resp_to_log_resp_(resp: LogResponse) -> LogResponse_ {
    LogResponse_{ 
                node_id: Some(uuid_to_uuid_(resp.node_id)), 
                current_term: resp.current_term,
                ack: resp.ack,
                success: resp.success }
}

pub fn log_resp__to_log_resp(resp: LogResponse_) -> Result<(LogResponse), ()> {
    let uuid = match opt_uuid__to_uuid(resp.node_id) {
        Ok(uuid) => uuid,
        Err(_) => return Err(())
    };
    Ok(LogResponse::new(uuid, resp.current_term, resp.ack, resp.success))
}

pub fn lreq_to_lreq_(req: LogRequest)  -> LogRequest_ {
    let uuid_ = Uuid_ {data: req.leader_id.to_simple().encode_lower(&mut Uuid::encode_buffer()).to_string()};
    let entries: Vec<CommitEntry> = req.entries.iter().cloned().map(|e| {
        let data = (*e.0).clone();
        CommitEntry{data, term: e.1}
    }).collect();

    LogRequest_{
        leader_id: Some(uuid_), 
        term: req.term, 
        log_length: req.log_length,
        log_term: req.log_term, 
        log_commit: req.leader_commit, 
        entries }
}

pub fn vote_req_to_vote_req_(vr: VoteRequest) -> VoteRequest_ {
    let uuid = uuid_to_uuid_(vr.0);
    VoteRequest_{candidate_id: Some(uuid), candidate_term: vr.1, candidate_log_length: vr.2, candidate_log_term: vr.3}
}

pub fn vote_resp__to_vote_resp(vr: VoteResponse_) -> Result<VoteResponse, ()> {
    let uuid = match opt_uuid__to_uuid(vr.voter_id) {
        Ok(uuid) => uuid, 
        Err(_) => return Err(())
    };

    Ok(VoteResponse(uuid, vr.term, vr.granted))
}