//! M6 principal / project resolution from environment ([#138](https://github.com/zyx-corporation/kotonoha-management/issues/138) M6-c).

use uuid::Uuid;

/// Optional M6 scope from `KOTONOHA_PRINCIPAL_ID` / `KOTONOHA_PROJECT_ID`.
#[derive(Debug, Clone, Copy, Default)]
pub struct M6EnvContext {
    pub principal_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
}

impl M6EnvContext {
    /// Reads `KOTONOHA_PRINCIPAL_ID` and `KOTONOHA_PROJECT_ID` (invalid UUID → ignored).
    pub fn from_env() -> Self {
        Self {
            principal_id: parse_uuid_env("KOTONOHA_PRINCIPAL_ID"),
            project_id: parse_uuid_env("KOTONOHA_PROJECT_ID"),
        }
    }
}

fn parse_uuid_env(key: &str) -> Option<Uuid> {
    std::env::var(key)
        .ok()
        .and_then(|s| Uuid::parse_str(s.trim()).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_uuid_env() {
        let id = "00000000-0000-4000-8000-000000000001";
        std::env::set_var("KOTONOHA_PRINCIPAL_ID", id);
        let ctx = M6EnvContext::from_env();
        std::env::remove_var("KOTONOHA_PRINCIPAL_ID");
        assert_eq!(ctx.principal_id.unwrap().to_string(), id);
    }
}
