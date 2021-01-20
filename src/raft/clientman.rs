
use tonic::{Request};
use std::sync::Arc;
use raftservice::raft_service_client::RaftServiceClient;
use actix::prelude::*;
use uuid::Uuid;
use std::str::FromStr;
use hashbrown::{HashMap, HashSet};
use crate::raft::state::Raft;

pub mod raftservice {
    tonic::include_proto!("raftservice");
}

pub struct ClientManager {
    raft: Arc<Addr<Raft>>,
    nodes: HashMap<Uuid, RaftServiceClient<tonic::transport::Channel>>,
}

impl ClientManager {
    pub async fn new(raft: Arc<Addr<Raft>>, node_addrs: Vec<String>) -> ClientManager {
        let mut nodes = HashMap::new();
        let mut set: HashSet<String> = HashSet::new();

        for addr in node_addrs {
            match set.get(&addr) {
                Some(e) => e,
                None => continue
            };

            set.insert(addr.to_owned());

            let mut client = RaftServiceClient::connect(addr).await.unwrap();
            let id = client.get_uuid(Request::new({})).await.unwrap().into_inner();
            let id = Uuid::from_str(&id.data).unwrap();
            
            // try and fix partitions on startup
            let addrs = client.get_addrs(Request::new({})).await.unwrap().into_inner().addrs;
            for addr in addrs {
                match set.get(&addr) {
                    Some(_) => {},
                    None => continue
                };
                let mut client = RaftServiceClient::connect(addr.to_owned()).await.unwrap();
                let id = client.get_uuid(Request::new({})).await.unwrap().into_inner();
                let id = Uuid::from_str(&id.data).unwrap();
                nodes.insert(id, client);
                set.insert(addr);
            }
            nodes.insert(id, client);
        }
        
        ClientManager{raft, nodes}
    }
}

impl Actor for ClientManager {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {

    }
}