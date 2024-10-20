use async_trait::async_trait;

use crate::{app_state::AppState, entity::Entity, error::Result};

#[async_trait]
pub trait Source {
    async fn get_entities(&mut self, state: &AppState) -> Result<Vec<(String, Entity)>>;
}
