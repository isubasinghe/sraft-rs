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

fn lreq_to_lreq_(req: LogRequest_) -> Result<LogRequest, ()> {
    let leader_id = match req.leader_id {
        Some(leader_id) => {
            Uuid::from_str(&leader_id.data)
        },
        None => return Err(())
    };

    let req = match leader_id {
        Ok(leader_id) => {
            let entries: Vec<(Arc<Vec<u8>>, u64)> = req.entries.iter().cloned().map(|e| {
                (Arc::new(e.data), e.term)
            }).collect();

            LogRequest::new(leader_id, req.term, req.log_length, req.log_term, req.log_commit, entries)
        },
        Err(e) => return Err(())
    };

    Ok(req)
}