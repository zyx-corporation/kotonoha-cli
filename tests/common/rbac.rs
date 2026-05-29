//! Set legacy default principal role for M6 RBAC (one role per principal in DB).

use kotonoha_core::store::principals::LegacyDefaults;

pub fn set_legacy_member_role(database_url: &str, role: &str) {
    let rt = tokio::runtime::Runtime::new().expect("runtime");
    rt.block_on(async {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect(database_url)
            .await
            .expect("connect");
        sqlx::query(
            r#"UPDATE project_members
               SET role = $1
               WHERE project_id = $2 AND principal_id = $3"#,
        )
        .bind(role)
        .bind(LegacyDefaults::PROJECT_ID)
        .bind(LegacyDefaults::PRINCIPAL_ID)
        .execute(&pool)
        .await
        .expect("set legacy role");
    });
}
