#[cfg(not(target_env = "msvc"))]
use jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

mod raft;

use tracing::{error};
use crate::raft::server::{start};
use std::sync::Arc;

fn main()  {
    tracing_subscriber::fmt()
        // enable everything
        .with_max_level(tracing::Level::INFO)
        // sets this to be the default, global collector for this application.
        .init();
    let out = start("127.0.0.1:3000".to_string(), vec!["127.0.0.1:3001".to_string()], 1);


}

