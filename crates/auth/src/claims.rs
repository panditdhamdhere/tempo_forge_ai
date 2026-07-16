use serde::{Deserialize, Serialize};
use tempoforge_common::{OrgId, UserId};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Owner,
    Admin,
    Member,
    Viewer,
}

impl Role {
    pub fn at_least(self, required: Role) -> bool {
        self.rank() >= required.rank()
    }

    fn rank(self) -> u8 {
        match self {
            Self::Viewer => 1,
            Self::Member => 2,
            Self::Admin => 3,
            Self::Owner => 4,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContext {
    pub user_id: UserId,
    pub clerk_user_id: String,
    pub org_id: Option<OrgId>,
    pub role: Role,
    pub email: Option<String>,
    pub session_id: Option<String>,
}

impl AuthContext {
    pub fn from_clerk(clerk_user_id: String, email: Option<String>, session_id: Option<String>) -> Self {
        Self {
            user_id: UserId(Uuid::nil()),
            clerk_user_id,
            org_id: None,
            role: Role::Member,
            email,
            session_id,
        }
    }
}
