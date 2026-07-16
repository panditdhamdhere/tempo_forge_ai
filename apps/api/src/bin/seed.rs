use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://tempoforge:tempoforge@localhost:5432/tempoforge".into());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    sqlx::migrate!("../../migrations").run(&pool).await?;

    sqlx::query(
        r#"
        INSERT INTO templates (slug, name, description, category, payload)
        VALUES
          ('erc20-staking', 'ERC20 + Staking', 'TIP-20 compatible token with staking vault', 'tokens', '{"prompt":"Create an ERC20 with staking"}'::jsonb),
          ('payment-splitter', 'Payment Splitter', 'Split TIP-20 payments across recipients', 'payments', '{"prompt":"Create a payment splitter for TIP-20"}'::jsonb),
          ('nft-collection', 'NFT Collection', 'ERC721 collection with mint roles', 'nft', '{"prompt":"Create an NFT collection with allowlist mint"}'::jsonb)
        ON CONFLICT (slug) DO NOTHING
        "#,
    )
    .execute(&pool)
    .await?;

    let user_id = sqlx::query_scalar::<_, uuid::Uuid>(
        r#"
        INSERT INTO users (id, clerk_user_id, email, name)
        VALUES ('00000000-0000-4000-8000-000000000001', 'user_dev', 'dev@tempoforge.ai', 'Dev User')
        ON CONFLICT (clerk_user_id) DO UPDATE SET email = EXCLUDED.email
        RETURNING id
        "#,
    )
    .fetch_one(&pool)
    .await?;

    let org_id = sqlx::query_scalar::<_, uuid::Uuid>(
        r#"
        INSERT INTO organizations (name, slug, plan)
        VALUES ('TempoForge Labs', 'tempoforge-labs', 'pro')
        ON CONFLICT (slug) DO UPDATE SET updated_at = NOW()
        RETURNING id
        "#,
    )
    .fetch_one(&pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO organization_members (org_id, user_id, role)
        VALUES ($1, $2, 'owner')
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(org_id)
    .bind(user_id)
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO projects (org_id, name, slug, description, created_by)
        VALUES ($1, 'Demo Payments', 'demo-payments', 'Sample Tempo payments project', $2)
        ON CONFLICT (org_id, slug) DO NOTHING
        "#,
    )
    .bind(org_id)
    .bind(user_id)
    .execute(&pool)
    .await?;

    tracing::info!("seed complete");
    Ok(())
}
