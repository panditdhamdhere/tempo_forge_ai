use crate::claims::{AuthContext, Role};
use tempoforge_common::{AppError, AppResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    ProjectRead,
    ProjectWrite,
    ContractDeploy,
    AuditRun,
    ApiKeyManage,
    BillingManage,
    OrgAdmin,
}

impl Permission {
    pub fn required_role(self) -> Role {
        match self {
            Self::ProjectRead => Role::Viewer,
            Self::ProjectWrite | Self::AuditRun => Role::Member,
            Self::ContractDeploy | Self::ApiKeyManage => Role::Admin,
            Self::BillingManage | Self::OrgAdmin => Role::Owner,
        }
    }
}

pub fn authorize(ctx: &AuthContext, permission: Permission) -> AppResult<()> {
    if ctx.role.at_least(permission.required_role()) {
        Ok(())
    } else {
        Err(AppError::Forbidden(format!(
            "role {:?} lacks permission {:?}",
            ctx.role, permission
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempoforge_common::UserId;
    use uuid::Uuid;

    fn ctx(role: Role) -> AuthContext {
        AuthContext {
            user_id: UserId(Uuid::nil()),
            clerk_user_id: "user_1".into(),
            org_id: None,
            role,
            email: None,
            session_id: None,
        }
    }

    #[test]
    fn viewer_can_read_not_write() {
        assert!(authorize(&ctx(Role::Viewer), Permission::ProjectRead).is_ok());
        assert!(authorize(&ctx(Role::Viewer), Permission::ProjectWrite).is_err());
    }
}
