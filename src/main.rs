mod cli;

use anyhow::anyhow;
use clap::{IntoApp, Parser};
use cli::Opts;
use jungle_fi_cli_utils::input_parsing::config::get_solana_cli_config;
use shadow_drive_cli::{WrappedSigner, GENESYSGO_RPC};
use solana_clap_v3_utils::keypair::signer_from_path;
use std::io::stdin;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // CLI Parse
    let opts = Opts::parse();

    // Get signer
    let app = Opts::into_app();
    let matches = app.get_matches();
    let config = get_solana_cli_config()?;
    let keypath = opts
        .cfg_override
        .keypair
        .unwrap_or(config.keypair_path.clone());
    let mut wallet_manager = None;
    let signer = signer_from_path(&matches, &keypath, "keypair", &mut wallet_manager)
        .map_err(|e| anyhow!("Could not resolve signer: {:?}", e))?;
    let signer = WrappedSigner::new(signer);
    let url = opts.cfg_override.url.unwrap_or(GENESYSGO_RPC.to_string());

    opts.command
        .process(signer, &url, opts.cfg_override.skip_confirm)
        .await?;
    Ok(())
}

/// Confirm from the user that they definitely want some irreversible
/// operation to occur.
fn wait_for_user_confirmation(skip: bool) -> anyhow::Result<()> {
    if skip {
        return Ok(());
    }
    println!("Press ENTER to continue, or CTRL+C to abort");
    let mut proceed = String::new();
    stdin().read_line(&mut proceed)?;
    Ok(())
}
