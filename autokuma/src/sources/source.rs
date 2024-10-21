use async_trait::async_trait;

use crate::{entity::Entity, error::Result};

#[async_trait]
pub trait Source {
    fn name(&self) -> &'static str;
    async fn init(&mut self) -> Result<()>;
    async fn get_entities(&mut self) -> Result<Vec<(String, Entity)>>;
    async fn shutdown(&mut self) -> Result<()>;
}
