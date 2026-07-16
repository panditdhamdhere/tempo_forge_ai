use super::auditor::AuditorAgent;
use super::chat::ChatAgent;
use super::codegen::CodeGeneratorAgent;
use super::debugger::DebuggerAgent;
use super::docs::DocumentationAgent;
use super::planner::PlannerAgent;
use super::tests_agent::TestGeneratorAgent;
use super::traits::Agent;
use crate::providers::LlmProvider;
use crate::rag::{InMemoryRag, RagStore};
use crate::types::{AgentKind, AgentRequest, AgentResponse};
use std::sync::Arc;
use tempoforge_common::{AppError, AppResult};

pub struct AgentOrchestrator {
    planner: PlannerAgent,
    codegen: CodeGeneratorAgent,
    auditor: AuditorAgent,
    debugger: DebuggerAgent,
    docs: DocumentationAgent,
    tests: TestGeneratorAgent,
    chat: ChatAgent,
}

impl AgentOrchestrator {
    pub fn new(provider: Arc<dyn LlmProvider>, rag: Arc<dyn RagStore>) -> Self {
        Self {
            planner: PlannerAgent::new(provider.clone()),
            codegen: CodeGeneratorAgent::new(provider.clone(), rag.clone()),
            auditor: AuditorAgent::new(provider.clone(), rag.clone()),
            debugger: DebuggerAgent::new(provider.clone()),
            docs: DocumentationAgent::new(provider.clone()),
            tests: TestGeneratorAgent::new(provider.clone()),
            chat: ChatAgent::new(provider, rag),
        }
    }

    pub fn with_defaults(provider: Arc<dyn LlmProvider>) -> Self {
        Self::new(provider, Arc::new(InMemoryRag::with_seed_docs()))
    }

    pub async fn run(&self, request: AgentRequest) -> AppResult<AgentResponse> {
        match request.agent {
            AgentKind::Planner => self.planner.run(request).await,
            AgentKind::CodeGenerator => self.codegen.run(request).await,
            AgentKind::Auditor => self.auditor.run(request).await,
            AgentKind::Debugger => self.debugger.run(request).await,
            AgentKind::DocumentationWriter => self.docs.run(request).await,
            AgentKind::TestGenerator => self.tests.run(request).await,
            AgentKind::Chat | AgentKind::Architect | AgentKind::DeploymentAssistant => {
                self.chat.run(request).await
            }
        }
    }

    pub async fn route_by_name(&self, agent: &str, mut request: AgentRequest) -> AppResult<AgentResponse> {
        request.agent = parse_agent(agent)?;
        self.run(request).await
    }
}

fn parse_agent(name: &str) -> AppResult<AgentKind> {
    match name {
        "planner" => Ok(AgentKind::Planner),
        "codegen" | "code_generator" => Ok(AgentKind::CodeGenerator),
        "auditor" => Ok(AgentKind::Auditor),
        "debugger" => Ok(AgentKind::Debugger),
        "architect" => Ok(AgentKind::Architect),
        "docs" | "documentation" => Ok(AgentKind::DocumentationWriter),
        "tests" | "test_generator" => Ok(AgentKind::TestGenerator),
        "deploy" | "deployment" => Ok(AgentKind::DeploymentAssistant),
        "chat" => Ok(AgentKind::Chat),
        other => Err(AppError::BadRequest(format!("unknown agent: {other}"))),
    }
}
