use clap::{Parser, Subcommand};
use serde_json::json;

#[derive(Parser)]
#[command(name = "tempoforge", about = "TempoForge AI CLI", version)]
struct Cli {
    #[arg(long, env = "API_URL", default_value = "http://localhost:8080")]
    api_url: String,

    #[arg(long, env = "TEMPOFORGE_TOKEN", default_value = "dev")]
    token: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check API health
    Health,
    /// Run an AI agent
    Agent {
        #[arg(long, default_value = "chat")]
        name: String,
        prompt: String,
    },
    /// Static + AI audit of a Solidity file
    Audit {
        #[arg(long)]
        file: String,
        #[arg(long, default_value = "CLI Audit")]
        title: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let client = reqwest::Client::new();

    match cli.command {
        Commands::Health => {
            let url = format!("{}/api/v1/health", cli.api_url.trim_end_matches('/'));
            let body: serde_json::Value = client.get(url).send().await?.json().await?;
            println!("{}", serde_json::to_string_pretty(&body)?);
        }
        Commands::Agent { name, prompt } => {
            let url = format!(
                "{}/api/v1/ai/agents/{}",
                cli.api_url.trim_end_matches('/'),
                name
            );
            let body: serde_json::Value = client
                .post(url)
                .bearer_auth(&cli.token)
                .json(&json!({ "prompt": prompt }))
                .send()
                .await?
                .json()
                .await?;
            println!("{}", serde_json::to_string_pretty(&body)?);
        }
        Commands::Audit { file, title } => {
            let source = std::fs::read_to_string(&file)?;
            let url = format!("{}/api/v1/audit", cli.api_url.trim_end_matches('/'));
            let body: serde_json::Value = client
                .post(url)
                .bearer_auth(&cli.token)
                .json(&json!({
                    "title": title,
                    "source": source,
                    "use_ai": true
                }))
                .send()
                .await?
                .json()
                .await?;
            println!("{}", serde_json::to_string_pretty(&body)?);
        }
    }

    Ok(())
}
