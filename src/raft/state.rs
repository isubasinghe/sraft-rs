use actix::prelude::*;
use uuid::Uuid;
use std::collections::{HashSet, HashMap};
use std::iter::FromIterator;
use std::time::Duration;
use serde::{Serialize, Deserialize};

#[derive(Eq, PartialEq)]
pub enum Role {
    Follower,
    Candidate,
    Leader
}


struct StateData {
    current_term: u64,
    voted_for: Option<Uuid>,
    log: Vec<(Vec<u8>, u64)>,
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

#[derive(MessageResponse)]
#[derive(Message)]
#[rtype(result="VoteResponse")]
pub struct VoteRequest(Uuid, u64, u64, u64);

#[derive(MessageResponse)]
#[derive(Message)]
#[rtype(result="()")]
pub struct VoteResponse(Uuid, u64, bool);

pub struct Raft
{
    state_data: StateData,
    node_id: Uuid,
    election_handle: Option<SpawnHandle>,
    nodes: HashMap<Uuid, Addr<Raft>>
}

impl Raft {
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
        self.election_handle = Some(ctx.run_later(Duration::from_secs(1), |_, ctx| {
            ctx.address().do_send(StateError::Timeout);
        }));
        ()
    }
}

impl Handler<VoteRequest> for Raft {
    
    type Result = VoteResponse;

    fn handle(&mut self, msg: VoteRequest, ctx: &mut Context<Self>) -> Self::Result {
        let mut my_log_term = 0;

        if self.state_data.log.len() < 0 {
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
        if (self.state_data.current_role == Role::Candidate) && 
            self.state_data.current_term == msg.1 && msg.2 {
            
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
