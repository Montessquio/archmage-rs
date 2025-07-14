use std::sync::Arc;

use serenity::async_trait;
use thiserror::Error;

use crate::db::ArchmageDatabase;

mod pbp;
pub use pbp::PpbService;

#[derive(Debug, Error)]
pub enum ServiceError {

}

#[async_trait]
pub trait Service {
    async fn start(db: Arc<ArchmageDatabase>) -> Result<(), ServiceError>;
}