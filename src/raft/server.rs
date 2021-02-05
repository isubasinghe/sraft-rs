
use actix_web::{get, web, App, middleware, HttpServer, Responder, HttpRequest, HttpResponse};
use tonic::{transport::Server, Request, Response, Status};
use uuid::Uuid;
use actix::prelude::*;
use std::sync::Arc;
use tracing::{info, instrument};
use std::collections::{BTreeMap};
use tokio::runtime::Runtime;
use std::borrow::Borrow;
use crate::raft::client::*;
use crate::raft::messages::*;
use crate::raft::state::{Raft, StateData};
use crate::raft::raftservice::raft_service_server::{RaftService, RaftServiceServer};
use crate::raft::raftservice::{
    LogResponse as LogResponse_, 
    LogRequest as LogRequest_, 
    VoteRequest as VoteRequest_, 
    VoteResponse as VoteResponse_,
    Uuid as Uuid_,
    Addrs,
    BroadCastMsgData
};
use crate::raft::transformers::*;
use crate::raft::application::*;

pub struct RaftServiceImpl {
    raft: Arc<Addr<Raft>>,
    uuid: Uuid_,
}

impl Actor for RaftServiceImpl {
    type Context = Context<Self>;
}

#[tonic::async_trait]
impl RaftService for RaftServiceImpl {
    async fn do_log_request(&self, request: Request<LogRequest_>) -> Result<Response<LogResponse_>, Status> {
        let x = request.into_inner();
        let leader_id = match opt_uuid__to_uuid(x.leader_id) {
            Ok(uuid) => uuid, 
            Err(e) => return Err(e)
        };
        let entries: Vec<(Arc<Vec<u8>>, u64)> = x.entries.iter().cloned().map(|e| {
            (Arc::new(e.data), e.term)
        }).collect();

        let req = LogRequest::new(leader_id, x.term, x.log_length, x.log_term, x.log_commit, entries);
        let x = match self.raft.send(req).await {
            Ok(e) => {
                let uuid_ = Uuid_ {data: e.node_id.to_simple().encode_lower(&mut Uuid::encode_buffer()).to_string()}; 
                LogResponse_ { node_id: Some(uuid_), current_term: e.current_term, ack: e.ack, success: e.success}
            },
            Err(e) => return Err(Status::internal(""))
        };
        Ok(Response::new(x))
    }

    async fn do_vote_request(&self, request: Request<VoteRequest_>) -> Result<Response<VoteResponse_>, Status> {
        let request = request.into_inner();
        let uuid = match opt_uuid__to_uuid(request.candidate_id) {
            Ok(uuid) => uuid, 
            Err(e) => return Err(e)
        };
        let vote_request = VoteRequest(
                                uuid, 
                                request.candidate_term, 
                                request.candidate_log_length, 
                                request.candidate_log_term);
        let resp = match self.raft.send(vote_request).await {
            Ok(resp) => resp,
            Err(_) => return Err(Status::invalid_argument(""))
        };
        
        let uuid = Uuid_ {data: resp.0.to_simple().encode_lower(&mut Uuid::encode_buffer()).to_string()}; 
        Ok(Response::new(
            VoteResponse_{voter_id: Some(uuid), term: resp.1, granted: resp.2}
        ))
    }

    async fn broad_cast_msg(&self, request: Request<BroadCastMsgData>) -> Result<Response<()>, Status> {
        let data = Arc::new(request.into_inner().data);
        let msg: BroadcastMsg = BroadcastMsg{data};
        self.raft.do_send(msg);
        Ok(Response::new(()))
    }

    async fn get_addrs(&self, request: Request<()>) -> Result<Response<Addrs>, Status> {
        
        unimplemented!();
    }

    async fn get_uuid(&self, request: Request<()>) -> Result<Response<Uuid_>, Status> {
        Ok(Response::new(self.uuid.clone()))
    }
}

async fn index(
    item: web::Json<ApplicationActions>, 
    addr: web::Data<Addr<Application>>,
    req: HttpRequest,
) -> HttpResponse {
    let x = addr.borrow();
    x.do_send(item.0);
    HttpResponse::Ok().json("")
}

pub fn start(addr: String, addrs: Vec<String>, id: u128, start_http: bool) -> Result<i32, Box<dyn std::error::Error>> {
    info!("SERVER: Starting server");
    let rt = Arc::new(Runtime::new().unwrap());
    let mut system = actix::System::new("test");
    
    let addr_server = addr.parse()?;

    let uuid = uuid_to_uuid_(Uuid::from_u128(id));

    let rt_ = rt.clone();
    let (raft, app): (Addr<Raft>, Addr<Application>) = system.block_on(async move {
        let mut app_outer = None;
        let raft = Raft::create(|ctx| {
            let raft = ctx.address();
            let app = Application::new(raft.clone().recipient()).start();
            app_outer = Some(app.clone());
            let app_recipient = app.clone().recipient();
            for i in 0..(addrs.len()) {
                Client::new(addrs[i].clone(), raft.clone(), rt_.clone()).start();
            }
            let state_data = StateData::default();
            Raft{
                state_data,
                node_id: Uuid::from_u128(id), 
                timer_handle: None,
                nodes: BTreeMap::new(),
                replicator_handle: None,
                app: app_recipient
    
            }
        });
        (raft, app_outer.unwrap())
    });
    let greeter = RaftServiceImpl{raft: Arc::new(raft), uuid};
    

    rt.spawn(async move {
        // tonic server
        let _ = Server::builder()
                .add_service(RaftServiceServer::new(greeter))
                .serve(addr_server)
                .await;
    });

    info!("SERVER: Started server");
    let http_server_fut = async move {

        HttpServer::new(move || {
    
            App::new()
                .data(app.clone()) // add shared state
                // enable logger
                .wrap(middleware::Logger::default())
                // register simple handler
                .service(web::resource("/").to(index))
        })
        .bind("127.0.0.1:8080")?
        .run()
        .await
    };

    if start_http {
        system.block_on(http_server_fut);
    }
    system.run();

    Ok(1)
}