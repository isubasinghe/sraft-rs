#[cfg(not(target_env = "msvc"))]
use jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

mod raft;

use tracing::{error};
use crate::raft::server::{start};
use actix_web::{get, web, App, HttpServer, Responder};
use std::sync::Arc;

#[get("/{id}/{name}/index.html")]
async fn index(web::Path((id, name)): web::Path<(u32, String)>) -> impl Responder {
    format!("Hello {}! id:{}", name, id)
}
fn main()  {
    tracing_subscriber::fmt()
        // enable everything
        .with_max_level(tracing::Level::INFO)
        // sets this to be the default, global collector for this application.
        .init();
    let out = start("127.0.0.1:3000".to_string(), vec!["127.0.0.1:3001".to_string()], 1);


}

