use actix::prelude::*;
use uuid::Uuid;
use std::collections::{HashSet, HashMap, BTreeMap};
use std::iter::FromIterator;
use std::time::{Duration, Instant};
use std::sync::Arc;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use tracing::{info};
use rand::prelude::*;

use crate::raft::messages::*;

#[derive(Eq, PartialEq, Debug)]
pub enum Role {
    Follower,
    Candidate,
    Leader
}
#[derive(Eq, PartialEq, Debug)]
pub struct StateData {
    current_term: u64,
    voted_for: Option<Uuid>,
    log: Vec<(Arc<Vec<u8>>, u64)>,
    commit_length: u64,
    current_role: Role,
    current_leader: Option<Uuid>,
    votes_received: HashSet<Uuid>,
    sent_length: HashMap<Uuid, u64>, 
    acked_length: HashMap<Uuid, u64>,
    last_msg_time: Instant
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
            last_msg_time: Instant::now()
        }
    }
}
#[derive(Eq, PartialEq, Debug)]
pub struct Raft
{
    pub state_data: StateData,
    pub node_id: Uuid,
    pub timer_handle: Option<SpawnHandle>,
    pub nodes: BTreeMap<Uuid, Recipient<NodeMsgs>>,
    pub replicator_handle: Option<SpawnHandle>,
    pub app: Recipient<AppMsg>,
}

impl Raft {
    fn acks(&mut self, length: usize) -> usize {
        let mut num = 0;
        self.state_data.acked_length.iter().for_each(|(_, val)| {
            if (*val) as usize >= length {
                num += 1;
            }
        });
        num
    }
    fn append_entries(&mut self, log_length: u64, leader_commit: u64, entries: Vec<(Arc<Vec<u8>>, u64)>) {
        info!("RAFT: APPENDING ENTRIES");
        if entries.len() > 0 && self.state_data.log.len() > log_length as usize {
            if self.state_data.log[log_length as usize].1 != entries[0].1 {
                self.state_data.log.truncate(log_length as usize -1);
            }
        }

        if log_length as usize + entries.len() > self.state_data.log.len() {
            for i in (self.state_data.log.len() - log_length as usize)..(entries.len() -1) {
                self.state_data.log.push(entries[i].clone());
            }
        }
        if leader_commit > self.state_data.commit_length {
            for i in (self.state_data.commit_length)..(leader_commit -1) {
                info!("RAFT: SENDING ENTRY TO APPLICATION LAYER");
                self.app.do_send(AppMsg{data: entries[i as usize].0.clone()}).unwrap();
            }
            self.state_data.commit_length = leader_commit;
        }
    }
    fn commit_log_entries(&mut self) {
        info!("RAFT: COMMITING LOG ENTRIES");
        let min_acks = (self.nodes.len() + 1)/2;
        let mut max_ready = 0;
        for len in 1..(self.state_data.log.len()) {
            if self.acks(len) >= min_acks {
                max_ready = len;
            }
        }

        if max_ready > 0 && max_ready > self.state_data.commit_length as usize
            && self.state_data.log[max_ready - 1].1 == self.state_data.current_term {

            for i in (self.state_data.commit_length as usize)..(max_ready - 1) {
                info!("RAFT: SENDING ENTRY TO APPLICATION LAYER");
                self.app.do_send(AppMsg{data: self.state_data.log[i].0.clone()}).unwrap();
            }
            self.state_data.commit_length = max_ready as u64;
        }

    }
    fn simulate_crash(&mut self, addr: Addr<Raft>) {
        info!("RAFT: SIMULATING CRASH");
        addr.do_send(Crash);
    }
    fn reset_timers(&mut self, ctx: &mut Context<Self>) {
        info!("RAFT: RESETTING TIMERS");
    }
}

impl Actor for Raft {
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Context<Self>) {
        info!("RAFT: Raft started");
        let mut rng = thread_rng();

        self.state_data.last_msg_time = Instant::now();
        self.replicator_handle = Some(ctx.run_interval(Duration::from_secs(2), |act, ctx| {
            if act.state_data.current_role == Role::Leader {
                ctx.address().do_send(ReplicateLogAllExcept);
            }
        }));
        self.timer_handle = Some(ctx.run_interval(Duration::from_secs(1), move |act, ctx| {
            let new_time = Instant::now();
            let duration = new_time.duration_since(act.state_data.last_msg_time);
            
            let time_ms: u64 = rng.gen_range(150..350);
            if act.state_data.current_role != Role::Leader && duration > Duration::from_millis(time_ms){
                ctx.address().do_send(Timeout);
            }

            act.state_data.last_msg_time = new_time;
        }));
    }
    fn stopped(&mut self, ctx: &mut Context<Self>) {
        info!("RAFT: Stopping raft");
    }
}

impl Handler<Crash> for Raft {
    type Result = ();
    fn handle(&mut self, _msg: Crash, ctx: &mut Context<Self>) -> Self::Result {
        info!("RAFT: Handling crash");
        self.state_data.current_role = Role::Follower;
        self.state_data.current_leader = None;
        self.state_data.votes_received = HashSet::new();
        self.state_data.sent_length = HashMap::new();
        self.state_data.acked_length = HashMap::new();
        ()
    }
}

impl Handler<Timeout> for Raft {
    type Result = ();
    fn handle(&mut self, _msg: Timeout, ctx: &mut Context<Self>) -> Self::Result {
        info!("RAFT: Handling timeout");
        self.state_data.current_term += 1;
        self.state_data.current_role = Role::Candidate;
        self.state_data.voted_for = Some(self.node_id);
        self.state_data.votes_received = HashSet::from_iter(vec![self.node_id]);
        let mut last_term = 0;

        if self.state_data.log.len() > 0 {
            last_term = self.state_data.log[self.state_data.log.len() - 1].1;
        }

        let msg = NodeMsgs::VoteRequest(VoteRequest(self.node_id, 
            self.state_data.current_term, 
            self.state_data.log.len() as u64, 
            last_term
            ));
        
        for (_, v) in &self.nodes {
            v.do_send(msg.clone()).unwrap();
        }
        ()
    }
}

impl Handler<VoteRequest> for Raft {
    
    type Result = VoteResponse;
    fn handle(&mut self, msg: VoteRequest, ctx: &mut Context<Self>) -> Self::Result {
        info!("RAFT: RECEIVED VOTEREQUEST");
        let mut my_log_term = 0;
        
        if self.state_data.log.len() > 0 {
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
            info!("RAFT: I AM FOLLOWER");
            return VoteResponse(self.node_id, self.state_data.current_term, true);
        }
        VoteResponse(self.node_id, self.state_data.current_term, false)
    }
}

impl Handler<VoteResponse> for Raft {
    type Result = ();
    fn handle(&mut self, msg: VoteResponse, ctx: &mut Context<Self>) -> Self::Result {
        info!("RAFT: RECEIVED VOTERESPONSE");

        if (self.state_data.current_role == Role::Candidate) && 
            self.state_data.current_term == msg.1 && msg.2 {
            
            self.state_data.votes_received.insert(msg.0);
            if self.state_data.votes_received.len() >= ((self.nodes.len() + 1) / 2) {
                info!("RAFT: I AM LEADER");
                self.state_data.current_role = Role::Leader;
                self.state_data.current_leader = Some(self.node_id);

                for (uuid, _) in &self.nodes {
                    if *uuid != self.node_id {
                        self.state_data.sent_length.insert(*uuid, self.state_data.log.len() as u64);
                        self.state_data.acked_length.insert(*uuid, 0);
                        info!("RAFT: ISSUING REPLICATELOG FROM VOTERESPONSE");
                        ctx.address().do_send(ReplicateLog{leader_id: self.node_id, follower_id: *uuid});
                    }
                }


            }


        }else if msg.1 > self.state_data.current_term {
            self.state_data.current_term = msg.1;
            info!("RAFT: I AM FOLLOWER");
            self.state_data.current_role = Role::Follower;
            self.state_data.voted_for = None;
        }
        ()
    }
}

impl Handler<BroadcastMsg> for Raft {
    type Result = ();
    fn handle(&mut self, msg: BroadcastMsg, ctx: &mut Context<Self>) -> Self::Result {
        info!("RAFT: HANDLING TOTAL ORDER BROADCAST");
        if self.state_data.current_role == Role::Leader {
            self.state_data.log.push((msg.data.clone(), self.state_data.current_term));
            self.state_data.acked_length.insert(self.node_id, self.state_data.log.len() as u64);

            for (uuid, _) in &self.nodes {
                if *uuid != self.node_id {
                    info!("RAFT: ISSUING REPLICATELOG FROM BROADCASTMSG");
                    ctx.address().do_send(ReplicateLog{leader_id: self.node_id, follower_id: *uuid});
                }
            }
        }else {
            match self.state_data.current_leader {
                Some(node_id) => {
                    match self.nodes.get(&node_id) {
                        Some(addr) => {
                            info!("RAFT: SENDING TOTAL ORDER BROADCAST TO LEADER");
                            addr.do_send(NodeMsgs::BroadcastMsg(msg)).unwrap();
                        },
                        None => {
                            self.reset_timers(ctx);
                            self.simulate_crash(ctx.address());
                        }
                    };
                }
                None => {
                    // this is weird, this isn't a state we are 
                    // meant to be in, let's simulate a crash
                    self.reset_timers(ctx); 
                    self.simulate_crash(ctx.address());
                }
            };
        }
        ()
    }
}

impl Handler<ReplicateLog> for Raft {
    type Result = ();
    #[inline(always)]
    fn handle(&mut self, msg: ReplicateLog, ctx: &mut Context<Self>) -> Self::Result {
        info!("RAFT: RECEIVED REPLICATELOG");
        let i = *self.state_data.sent_length.get(&msg.follower_id).unwrap_or(&0);
        info!("RAFT: SENT LENGTH {:?} = {1}", &msg.follower_id, i);
        let mut entries: Vec<(Arc<Vec<u8>>, u64)> = Vec::new();
        self.state_data.log.iter().skip(i as usize).cloned().for_each(|entry| {
            entries.push(entry);
        });
        info!("RAFT: BROADCASTING {:?}", entries);
        let mut prev_log_term: u64 = 0;
        if i > 0 {
            prev_log_term = self.state_data.log[(i-1) as usize].1;
        }
        match self.nodes.get(&msg.follower_id) {
            Some(addr) => {
                info!("RAFT: SENDING LOGREQUEST TO {:?}", &msg.follower_id);
                addr.do_send(
                    NodeMsgs::LogRequest(
                        LogRequest::new(
                            msg.leader_id, 
                            self.state_data.current_term,
                            i, 
                            prev_log_term,
                            self.state_data.commit_length,
                            entries
                        )
                    )
                ).unwrap();
            },  
            None => {
                // This is weird, I guess we are simulating a crash again
                self.reset_timers(ctx); 
                self.simulate_crash(ctx.address());
            }
        }
        ()
    }
}

impl Handler<ReplicateLogAllExcept> for Raft {
    type Result = ();
    #[inline(always)]
    fn handle(&mut self, _: ReplicateLogAllExcept, ctx: &mut Context<Self>) -> Self::Result {

        if self.state_data.current_role == Role::Leader {
            for (uuid, _) in &self.nodes {
                if *uuid != self.node_id {
                    info!("RAFT: ISSUING REPLICATELOG FROM REPLICATELOGALLEXCEPT");
                    ctx.address().do_send(ReplicateLog{leader_id: self.node_id, follower_id: *uuid});
                }
            }
        }

        ()
    }
}

impl Handler<LogRequest> for Raft {
    type Result = LogResponse;
    fn handle(&mut self, msg: LogRequest, ctx: &mut Context<Self>) -> Self::Result {
        info!("RAFT: RECEIVED LOGREQUEST, {:?}", msg.entries);
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
            info!("RAFT: I AM FOLLOWER");
            self.state_data.current_role = Role::Follower;
            self.state_data.current_leader = Some(msg.leader_id);
            let elen = msg.entries.len();
            self.append_entries(msg.log_length, msg.leader_commit, msg.entries);
            let ack = (self.state_data.log.len() + elen) as u64;
            return LogResponse::new(self.node_id, self.state_data.current_term, ack, true);
        }

        LogResponse::new(self.node_id, self.state_data.current_term, 0, false)
    }
}

impl Handler<LogResponse> for Raft {
    type Result = ();
    fn handle(&mut self, msg: LogResponse, ctx: &mut Context<Self>) -> Self::Result {
        info!("RAFT: RECEIVED LOGRESPONSE");
        if msg.current_term == self.state_data.current_term && self.state_data.current_role == Role::Leader {
            if msg.success == true && msg.ack >= *self.state_data.acked_length.get(&msg.node_id).unwrap_or(&0) {
                self.state_data.sent_length.insert(msg.node_id, msg.ack);
                self.state_data.acked_length.insert(msg.node_id, msg.ack);
                self.commit_log_entries();
            }else if  *self.state_data.sent_length.get(&msg.node_id).unwrap_or(&0) > 0 {
                *self.state_data.sent_length.get_mut(&msg.node_id).unwrap() -= 1;
                ctx.address().do_send(ReplicateLog{leader_id: self.node_id, follower_id: msg.node_id});
            }
        }else if msg.current_term > self.state_data.current_term {
            info!("RAFT: I AM FOLLOWER");
            self.state_data.current_term = msg.current_term;
            self.state_data.current_role = Role::Follower;
            self.state_data.voted_for = None;
        }
        ()
    }
}


impl Handler<GetNodesHash> for Raft {
    type Result = NodesHash;
    fn handle(&mut self, _: GetNodesHash, _: &mut Context<Self>) -> Self::Result {
        let mut hasher = DefaultHasher::new();
        self.nodes.hash(&mut hasher);
        NodesHash{id: hasher.finish()}
    }
}

impl Handler<NotifyUUID> for Raft {
    type Result = ();
    fn handle(&mut self, msg: NotifyUUID, ctx: &mut Context<Self>) -> Self::Result {
        info!("RAFT: RECEIVED NOTIFYUUID");
        self.nodes.insert(msg.node_id, msg.addr);
        ()
    }
}
