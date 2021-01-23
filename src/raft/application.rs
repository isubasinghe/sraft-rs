use actix::prelude::*;
use crate::raft::messages::*;
// Application layer
pub struct Application<A>  
    where
        A: Handler<AppMsg>
{
    actor: A
}

impl<A> Application<A> 
    where
        A: Handler<AppMsg>
{
    pub async fn start() {

    }
}