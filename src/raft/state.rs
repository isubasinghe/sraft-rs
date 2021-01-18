use actix::prelude::*;
use uuid::Uuid;
use std::collections::{HashSet, HashMap};
use std::iter::FromIterator;
use std::time::Duration;
use serde::{Serialize, Deserialize};
use std::rc::Rc;
use std::sync::Arc;

#[derive(Eq, PartialEq)]
pub enum Role {
    Follower,
    Candidate,
    Leader
}


struct StateData {
    current_term: u64,
    voted_for: Option<Uuid>,
    log: Vec<(Rc<Vec<u8>>, u64)>,
    commit_length: u64,
    current_role: Role,
    current_leader: Option<Uuid>,
    votes_received: HashSet<Uuid>,
    sent_length: HashMap<Uuid, u64>, 
    acked_length: HashMap<Uuid, u64>,
}

impl Default for StateData 
{
    fn default() -> Self { 
        StateData {
            current_term: 0, 
            voted_for: None, 
            log: Vec::new(),
            commit_length: 0,
            current_role: Role::Follower,
            current_leader: None,
            votes_received: HashSet::new(),
            sent_length: HashMap::new(),
            acked_length: HashMap::new(),
        }
    }
}

#[derive(Message)]
#[rtype(result="()")]
pub enum StateError {
    Crash,
    Timeout
}

#[derive(Message)]
#[rtype(result="()")]
pub struct ReplicateLog {
    leader_id: Uuid, 
    follower_id: Uuid
}

#[derive(Message)]
#[rtype(result="()")]
pub struct BroadcastMsg {
    data: Rc<Vec<u8>>
}

#[derive(MessageResponse)]
#[derive(Message)]
#[rtype(result="VoteResponse")]
pub struct VoteRequest(Uuid, u64, u64, u64);

#[derive(MessageResponse)]
#[derive(Message)]
#[rtype(result="()")]
pub struct VoteResponse(Uuid, u64, bool);

#[derive(Message)]
#[rtype(result="()")]
pub struct ReplicateLogAllExcept;

#[derive(Message)]
#[rtype(result="()")]
pub struct LogRequest {
    leader_id: Uuid,
    term: u64, 
    log_length: u64,
    log_term: u64, 
    leader_commit: u64,
    entries: Vec<(Arc<Vec<u8>>, u64)>,
}

#[derive(Message)]
#[rtype(result="()")]
pub struct LogResponse {
    node_id: Uuid,
    current_term: u64, 
    ack: u64, 
    succes: bool
}

pub struct Raft
{
    state_data: StateData,
    node_id: Uuid,
    election_handle: Option<SpawnHandle>,
    nodes: HashMap<Uuid, Addr<Raft>>,
    replicator_handle: Option<SpawnHandle>
}

impl Raft {

    pub fn default(node_id: Uuid) -> Raft {
        let state_data = StateData::default();

        Raft {state_data, node_id, election_handle: None, nodes: HashMap::new(), replicator_handle: None}
    }
    fn append_entries(&mut self, log_length: u64, leader_commit: u64, entries: Vec<(Rc<Vec<u8>>,u64)>) {
        if entries.len() > 0 && self.state_data.log.len() > log_length as usize{

        }        
    }

    fn commit_log_entries(&mut self) {

    }
}

impl Actor for Raft {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {

    }

    fn stopped(&mut self, ctx: &mut Context<Self>) {

    }
}

impl Handler<StateError> for Raft {
    type Result = ();

    fn handle(&mut self, _msg: StateError, ctx: &mut Context<Self>) -> Self::Result {
        self.state_data.current_term += 1;
        self.state_data.current_role = Role::Candidate;
        self.state_data.voted_for = Some(self.node_id);
        self.state_data.votes_received = HashSet::from_iter(vec![self.node_id]);
        let mut last_term = 0;

        if self.state_data.log.len() > 0 {
            last_term = self.state_data.log[self.state_data.log.len() - 1].1;
        }

        VoteRequest(self.node_id, 
                    self.state_data.current_term, 
                    self.state_data.log.len() as u64, 
                    last_term
                    );
        // Cancel the old election timer
        self.election_handle.map(|x| {
            ctx.cancel_future(x);
        });

        // Start a new election timer
        self.election_handle = Some(ctx.run_later(Duration::from_secs(1), |act, ctx| {
            ctx.address().do_send(StateError::Timeout);
        }));
        ()
    }
}

impl Handler<VoteRequest> for Raft {
    
    type Result = VoteResponse;

    fn handle(&mut self, msg: VoteRequest, ctx: &mut Context<Self>) -> Self::Result {
        let mut my_log_term = 0;

        if self.state_data.log.len() <= 0 {
            return VoteResponse(self.node_id, self.state_data.current_term, false);
        }else {
            my_log_term = self.state_data.log[self.state_data.log.len() -1].1;
        }
        let log_ok = msg.3 > my_log_term || (msg.3 == my_log_term && msg.2 >= self.state_data.log.len() as u64);
        let term_ok = (msg.1 > self.state_data.current_term) || 
                        (msg.1 == self.state_data.current_term && 
                            (self.state_data.voted_for == Some(msg.0) || self.state_data.voted_for == None));
        
        
        if term_ok && log_ok {
            self.state_data.current_term = msg.1;
            self.state_data.current_role = Role::Follower;
            self.state_data.voted_for = Some(msg.0);
            // true
        }else {
            // false
        }
        unimplemented!()
    }
}

impl Handler<VoteResponse> for Raft {
    type Result = ();

    fn handle(&mut self, msg: VoteResponse, ctx: &mut Context<Self>) -> Self::Result {
        match self.replicator_handle {
            Some(handle) => {ctx.cancel_future(handle);},
            None => {}
        };

        if (self.state_data.current_role == Role::Candidate) && 
            self.state_data.current_term == msg.1 && msg.2 {
            

            self.replicator_handle = Some(
                ctx.run_interval(Duration::from_secs(1), |_, ctx| {
                    ctx.address().do_send(ReplicateLogAllExcept);
                })
            );
            self.state_data.votes_received.insert(msg.0);
            if self.state_data.votes_received.len() >= ((self.nodes.len() + 1) / 2) {

                self.state_data.current_role = Role::Leader;
                self.state_data.current_leader = Some(self.node_id);
                
                match self.election_handle {
                    Some(handle) => {ctx.cancel_future(handle);}
                    None => {}
                }
                self.election_handle = None;

                for (uuid, _) in &self.nodes {
                    if *uuid != self.node_id {
                        self.state_data.sent_length.insert(*uuid, self.state_data.log.len() as u64);
                        self.state_data.acked_length.insert(*uuid, 0);

                        // REPLICATELOG(nodeId, follower)
                    }
                }


            }


        }else {
            match self.election_handle {
                Some(handle) => {ctx.cancel_future(handle);}
                None => {}
            }
            self.election_handle = None;
        }
        ()
    }
}

impl Handler<BroadcastMsg> for Raft {
    type Result = ();

    fn handle(&mut self, msg: BroadcastMsg, ctx: &mut Context<Self>) -> Self::Result {
        if self.state_data.current_role == Role::Leader {
            self.state_data.log.push((msg.data.clone(), self.state_data.current_term));
            self.state_data.acked_length.insert(self.node_id, self.state_data.log.len() as u64);

            for (uuid, _) in &self.nodes {
                if *uuid != self.node_id {
                    // REPLICATELOG(nodeId, follower)
                }
            }
        }else {
            // forward the request to the current leader
        }
        ()
    }
}

impl Handler<ReplicateLog> for Raft {
    type Result = ();
    #[inline(always)]
    fn handle(&mut self, msg: ReplicateLog, ctx: &mut Context<Self>) -> Self::Result {
        let i = self.state_data.sent_length.get(&msg.follower_id).unwrap_or(&0);
        let entries: Vec<(Vec<u8>, u64)> = Vec::new();
        let mut prev_log_term: u64 = 0;
        if *i > 0 {
            prev_log_term = self.state_data.log[(i+1) as usize].1;
        }
        // send logrequest to followerid
        ()
    }
}

impl Handler<ReplicateLogAllExcept> for Raft {
    type Result = ();

    #[inline(always)]
    fn handle(&mut self, msg: ReplicateLogAllExcept, ctx: &mut Context<Self>) -> Self::Result {


        ()
    }
}

impl Handler<LogRequest> for Raft {
    type Result = ();

    fn handle(&mut self, msg: LogRequest, ctx: &mut Context<Self>) -> Self::Result {
        
        if msg.term > self.state_data.current_term {
            self.state_data.current_term = msg.term;
            self.state_data.voted_for = None;
        }

        let mut log_ok = self.state_data.log.len() >= msg.log_length as usize;
        if msg.log_length as usize + msg.entries.len() > self.state_data.log.len() {
            log_ok = self.state_data.log.len() > 0 && 
                (msg.log_term == self.state_data.log[self.state_data.log.len() -1].1);
        }

        if msg.term == self.state_data.current_term && log_ok {
            self.state_data.current_role = Role::Follower;
            self.state_data.current_leader = Some(msg.leader_id);
            // APPENDENTRIES(log_length, leader_commit, entries)

            // send LogResponse(node_id, current_term, ack, true) to leader_id
        }else {
            // send LogResponse(node_id, current_term, 0, false) to leader_id
        }

        ()
    }
}

impl Handler<LogResponse> for Raft {
    type Result = ();

    fn handle(&mut self, msg: LogResponse, ctx: &mut Context<Self>) -> Self::Result {

        ()
    }
}