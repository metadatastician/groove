// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// groove-provider — run the reference Groove provider.
//
//   groove-provider                     # serve groove-ref on its registry port
//   groove-provider --port 6470        # port override
//   groove-provider --manifest m.json  # serve an arbitrary JSON manifest

#![forbid(unsafe_code)]

use anyhow::{Context, Result};
use clap::Parser;

/// Reference Groove provider (SPEC §2, §4, §5).
#[derive(Parser)]
#[command(name = "groove-provider", version, about)]
struct Cli {
    /// Port to bind on both [::1] and 127.0.0.1 (default: groove-ref's
    /// registry assignment).
    #[arg(long)]
    port: Option<u16>,

    /// Path to a JSON manifest file to serve instead of the built-in one.
    #[arg(long)]
    manifest: Option<String>,

    /// Base64 32-byte Ed25519 seed: serve a signed manifest (SPEC §2.1.5).
    /// Falls back to the GROOVE_SIGNING_KEY environment variable.
    #[arg(long)]
    signing_key: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let manifest = match &cli.manifest {
        Some(path) => {
            let text = std::fs::read_to_string(path)
                .with_context(|| format!("read manifest {path}"))?;
            Some(serde_json::from_str(&text).with_context(|| format!("parse manifest {path}"))?)
        }
        None => None,
    };

    let seed_b64 = cli.signing_key.or_else(|| std::env::var("GROOVE_SIGNING_KEY").ok());
    let signing_seed = match seed_b64 {
        Some(b64) => Some(groove::sign::decode_seed(&b64).context("--signing-key / GROOVE_SIGNING_KEY")?),
        None => None,
    };
    let signed = signing_seed.is_some();

    let config = groove_provider::Config {
        port: cli.port.unwrap_or_else(groove_provider::default_port),
        manifest,
        log_attestations: true,
        signing_seed,
    };

    let server = groove_provider::serve(config).await?;
    println!(
        "groove-provider: serving {kind} /.well-known/groove on [::1]:{p} and 127.0.0.1:{p}",
        kind = if signed { "SIGNED" } else { "unsigned" },
        p = server.port()
    );

    // Serve until interrupted.
    tokio::signal::ctrl_c().await?;
    println!("groove-provider: shutting down");
    Ok(())
}
