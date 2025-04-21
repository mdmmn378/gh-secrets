use base64::{engine::general_purpose, Engine as _};
use clap::Parser;
use serde::{Deserialize, Serialize};
use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::sealedbox;
use std::env;

#[derive(Parser, Debug)]
#[command(author, version, about = "Push a .env file to GitHub Actions secrets")]
struct Args {
    #[arg(long)]
    repo: String,

    #[arg(long, default_value = ".env")]
    env_file: String,

    #[arg(long)]
    prefix: Option<String>,

    #[arg(long)]
    environment: Option<String>,

    #[arg(long)]
    token: Option<String>,
}

#[derive(Deserialize)]
struct PublicKeyResponse {
    key_id: String,
    key: String,
}

#[derive(Serialize)]
struct SecretRequest<'a> {
    encrypted_value: &'a str,
    key_id: &'a str,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let token = args
        .token
        .or_else(|| env::var("GITHUB_TOKEN").ok())
        .or_else(|| env::var("GH_TOKEN").ok())
        .expect("GitHub token must be provided via --token or GITHUB_TOKEN/GH_TOKEN env var");

    sodiumoxide::init().expect("sodiumoxide init failed");

    let kvs = dotenvy::from_path_iter(&args.env_file)?
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    let client = reqwest::Client::new();
    let (owner, repo) = {
        let mut parts = args.repo.splitn(2, '/');
        (parts.next().unwrap(), parts.next().unwrap())
    };

    let base = format!("https://api.github.com/repos/{owner}/{repo}");
    let (pk_path, secret_base_path) = if let Some(env_name) = &args.environment {
        (
            format!("{base}/environments/{env_name}/secrets/public-key"),
            format!("{base}/environments/{env_name}/secrets"),
        )
    } else {
        (
            format!("{base}/actions/secrets/public-key"),
            format!("{base}/actions/secrets"),
        )
    };

    let pk_res: PublicKeyResponse = client
        .get(&pk_path)
        .header("Authorization", format!("Bearer {token}"))
        .header("User-Agent", "rust-env-secrets-cli")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let decoded_key = general_purpose::STANDARD
        .decode(pk_res.key.as_bytes())?
        .try_into()
        .expect("public key is wrong length");
    let public_key = box_::PublicKey(decoded_key);

    for (key, value) in kvs {
        let secret_name = match &args.prefix {
            Some(p) => format!("{p}{}", key.to_uppercase()),
            None => key.to_uppercase(),
        };

        let cipher = sealedbox::seal(value.as_bytes(), &public_key);
        let encrypted = general_purpose::STANDARD.encode(&cipher);

        let req_body = SecretRequest {
            encrypted_value: &encrypted,
            key_id: &pk_res.key_id,
        };

        let put_url = format!("{}/{}", secret_base_path, secret_name);
        let resp = client
            .put(&put_url)
            .header("Authorization", format!("Bearer {token}"))
            .header("User-Agent", "rust-env-secrets-cli")
            .header("Accept", "application/vnd.github+json")
            .json(&req_body)
            .send()
            .await?;

        if resp.status().is_success() {
            println!("✅ Pushed secret {}", secret_name);
        } else {
            eprintln!("❌ Failed to push {}: {}", secret_name, resp.text().await?);
        }
    }

    Ok(())
}
