use actix::prelude::*;
use uuid::Uuid;
use std::sync::Arc;


#[derive(Message)]
#[rtype(result="()")]
pub struct Timeout;

#[derive(Message)]
#[rtype(result="()")]
pub struct Crash;

#[derive(Message)]
#[rtype(result="()")]
pub struct ReplicateLog {
    pub leader_id: Uuid, 
    pub follower_id: Uuid
}

#[derive(Message)]
#[rtype(result="()")]
pub struct BroadcastMsg {
    pub data: Arc<Vec<u8>>
}

#[derive(MessageResponse, Clone)]
#[derive(Message)]
#[rtype(result="VoteResponse")]
pub struct VoteRequest(pub Uuid, pub u64, pub u64, pub u64);

#[derive(MessageResponse)]
#[derive(Message)]
#[rtype(result="()")]
pub struct VoteResponse(pub Uuid, pub u64, pub bool);

#[derive(Message)]
#[rtype(result="()")]
pub struct ReplicateLogAllExcept;

#[derive(Message, Clone)]
#[rtype(result="LogResponse")]
pub struct LogRequest {
    pub leader_id: Uuid,
    pub term: u64, 
    pub log_length: u64,
    pub log_term: u64, 
    pub leader_commit: u64,
    pub entries: Vec<(Arc<Vec<u8>>, u64)>,
}

impl LogRequest {
    pub fn new(leader_id: Uuid, term: u64, log_length: u64, log_term: u64, leader_commit: u64, entries: Vec<(Arc<Vec<u8>>, u64)>)  -> LogRequest {
        LogRequest{leader_id, term, log_length, log_term, leader_commit, entries}
    }
}
#[derive(MessageResponse)]
#[derive(Message)]
#[rtype(result="()")]
pub struct LogResponse {
    pub node_id: Uuid,
    pub current_term: u64, 
    pub ack: u64, 
    pub success: bool
}

impl LogResponse {
    pub fn new(node_id: Uuid, current_term: u64, ack: u64, success: bool) -> LogResponse {
        LogResponse{node_id, current_term, ack, success}
    }
}

#[derive(Message, Clone)]
#[rtype(result="()")]
pub enum NodeMsgs {
    VoteRequest(VoteRequest),
    LogRequest(LogRequest),
}

#[derive(Message, Clone)]
#[rtype(result="()")]
pub struct AppMsg {
    data: Vec<u8>,
    term: u64
}