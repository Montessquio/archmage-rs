use serenity::async_trait;
use crate::service::{Service, ServiceError};

pub struct PpbService;

#[async_trait]
impl Service for PpbService {
    async fn start(_db: std::sync::Arc<crate::db::ArchmageDatabase>) -> Result<(), ServiceError> {
        todo!()
    }
}
