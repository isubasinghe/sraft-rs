#[cfg(not(target_env = "msvc"))]
use jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

mod raft;

use clap::Clap;
use tracing::{error};
use crate::raft::server::{start};
use std::sync::Arc;





#[derive(Clap, Debug)]
#[clap(version = "1.0", author = "Kevin K. <kbknapp@gmail.com>")]
pub struct Opts {
    #[clap(short, long)]
    id: u128,
    /// Host to start the server
    #[clap(short, long)]
    host: String,
    /// Other servers
    #[clap(short, long)]
    nodes: Vec<String>
}

fn main()  {

    let opts = Opts::parse();

    println!("{:?}", opts);
    tracing_subscriber::fmt()
        // enable everything
        .with_max_level(tracing::Level::INFO)
        // sets this to be the default, global collector for this application.
        .init();
    let out = start(opts.host, opts.nodes, opts.id);


}

