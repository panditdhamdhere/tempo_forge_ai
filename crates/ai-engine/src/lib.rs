//! TempoForge AI engine: providers, agents, prompts, memory, and RAG.

pub mod agents;
pub mod memory;
pub mod prompts;
pub mod providers;
pub mod rag;
pub mod types;

pub use agents::orchestrator::AgentOrchestrator;
pub use providers::{LlmProvider, ProviderKind, create_provider};
pub use types::{AgentKind, AgentRequest, AgentResponse, ChatMessage, Role as ChatRole};
