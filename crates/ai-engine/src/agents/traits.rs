use crate::types::{AgentKind, AgentRequest, AgentResponse};
use async_trait::async_trait;
use tempoforge_common::AppResult;

#[async_trait]
pub trait Agent: Send + Sync {
    fn kind(&self) -> AgentKind;
    async fn run(&self, request: AgentRequest) -> AppResult<AgentResponse>;
}
