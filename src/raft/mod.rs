pub mod state;
pub mod messages;
pub mod server;
pub mod application;
pub mod client;
pub mod transformers;

pub mod raftservice {
    tonic::include_proto!("raftservice");
}