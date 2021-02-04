#[cfg(not(target_env = "msvc"))]
use jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

mod raft;

use clap::Clap;
use tracing::{error};
use std::str::FromStr;
use crate::raft::server::{start};
use std::sync::Arc;



#[derive(Clap, Debug)]
pub enum Logging {
    Trace,
    Info,
    Warn,
    Error
}

impl FromStr for Logging {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "trace" => Ok(Logging::Trace),
            "info" => Ok(Logging::Info),
            "warn" => Ok(Logging::Warn),
            "error" => Ok(Logging::Error),
            _ => Err("{trace, info, warn, error} expected")
        }
    }
}


#[derive(Clap, Debug)]
#[clap(version = "1.0", author = "Isitha S <{subasingheisitha} at {gmail.com}>")]
pub struct Opts {
    #[clap(short, long)]
    id: u128,
    /// Host to start the server
    #[clap(short, long)]
    host: String,
    /// Other servers
    #[clap(short, long)]
    nodes: Vec<String>,

    #[clap(short, long, default_value="info")]
    logging: Logging
}

fn main()  {

    let opts = Opts::parse();

    let trace_level = match opts.logging {
        Logging::Trace => tracing::Level::TRACE,
        Logging::Info => tracing::Level::INFO,
        Logging::Warn => tracing::Level::WARN,
        Logging::Error => tracing::Level::ERROR
    };
    tracing_subscriber::fmt()
        // enable everything
        .with_max_level(trace_level)
        // sets this to be the default, global collector for this application.
        .init();
    let out = start(opts.host, opts.nodes, opts.id);
    println!("{:?}", out);

}

