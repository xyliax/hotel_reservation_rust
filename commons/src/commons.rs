pub mod db_config;
pub mod interceptors;
pub mod tracing;
pub use colored::*;
pub use db_config::*;
pub use futures::stream::TryStreamExt;
pub use mongodb;
pub use mongodb::bson::document::ValueAccessError;
pub use mongodb::bson::{doc, Document};
pub use mongodb::{options::ClientOptions, Client, Collection};
pub use prost;
pub use std::collections::HashMap;
pub use std::sync::Arc;
pub use tokio;
pub use tokio::sync::Mutex;
pub use tokio::time::*;
pub use tonic;
pub use tonic::metadata::*;
pub use tonic::transport::{Channel, Error, Server};
pub use tonic::{Request, Response, Result, Status};
pub type Cache<T> = Arc<Mutex<HashMap<String, Vec<T>>>>;

pub mod user_svc {
    pub const NAME: &'static str = "srv-user";
    pub const ADDR: &'static str = "[::1]:50051";
    pub const PROT: &'static str = "http://[::1]:50051";
}

pub mod reserv_svc {
    pub const NAME: &'static str = "srv-reservation";
    pub const ADDR: &'static str = "[::1]:50052";
    pub const PROT: &'static str = "http://[::1]:50052";
}

pub mod profile_svc {
    pub const NAME: &'static str = "srv-profile";
    pub const ADDR: &'static str = "[::1]:50053";
    pub const PROT: &'static str = "http://[::1]:50053";
    pub const GET_COMMENTS_LOG: bool = true;
}

pub mod recomm_svc {
    pub const NAME: &'static str = "srv-recommendation";
    pub const ADDR: &'static str = "[::1]:50054";
    pub const PROT: &'static str = "http://[::1]:50054";
    pub const RECOMM_NUM: i64 = 100_i64;
    pub const POP_THRESHOLD: i64 = 0_i64;
    pub const GET_RECMD_LOG: bool = true;
}

pub mod resch_svc {
    pub const NAME: &'static str = "srv-research";
    pub const ADDR: &'static str = "[::1]:50055";
    pub const PROT: &'static str = "http://[::1]:50055";
    pub const NEARBY_NUM: i64 = 100_i64;
    pub const GET_RESCH_NEARBY_LOG: bool = true;
}

pub mod geo_svc {
    pub const NAME: &'static str = "srv-geo";
    pub const ADDR: &'static str = "[::1]:50056";
    pub const PROT: &'static str = "http://[::1]:50056";
    pub const NEARBY_LOG: bool = true;
}

pub mod rate_svc {
    pub const NAME: &'static str = "srv-rate";
    pub const ADDR: &'static str = "[::1]:50057";
    pub const PROT: &'static str = "http://[::1]:50057";
    pub const GET_RATE_PLAN_LOG: bool = true;
}

pub mod mono_svc {
    pub const NAME: &'static str = "srv-monolithic";
    pub const ADDR: &'static str = "[::1]:50080";
    pub const PROT: &'static str = "http://[::1]:50080";
}
