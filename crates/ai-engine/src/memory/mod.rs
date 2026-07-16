use crate::types::ChatMessage;
use std::collections::HashMap;
use std::sync::Arc;
use tempoforge_common::ConversationId;
use tokio::sync::RwLock;

/// In-process conversation memory. Production API persists to PostgreSQL;
/// this cache accelerates multi-turn agent loops within a request.
#[derive(Default, Clone)]
pub struct ConversationMemory {
    inner: Arc<RwLock<HashMap<ConversationId, Vec<ChatMessage>>>>,
}

impl ConversationMemory {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn get(&self, id: &ConversationId) -> Vec<ChatMessage> {
        self.inner
            .read()
            .await
            .get(id)
            .cloned()
            .unwrap_or_default()
    }

    pub async fn append(&self, id: ConversationId, messages: impl IntoIterator<Item = ChatMessage>) {
        let mut guard = self.inner.write().await;
        guard.entry(id).or_default().extend(messages);
    }

    pub async fn replace(&self, id: ConversationId, messages: Vec<ChatMessage>) {
        self.inner.write().await.insert(id, messages);
    }
}
