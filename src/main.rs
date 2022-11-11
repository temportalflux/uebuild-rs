use std::sync::{
	atomic::{self, AtomicBool},
	Arc,
};

use anyhow::Context;
use clap::Parser;

pub mod commands;
pub mod config;
pub mod types;
pub mod unreal;
pub mod utility;

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
pub enum Cli {
	InitCfg(config::SaveToDisk),
	Cfg(config::Configure),

	#[cfg(debug_assertions)]
	Ship(commands::ReleaseBinary),

	GenProjectFiles(commands::UpdateProjectFiles),
	FixupBinaries(commands::FixupBinaries),

	Compile(commands::Compile),
	Cook(commands::Cook),

	Editor(commands::RunEditor),
	Pisep(commands::RunPisep),
	#[command(subcommand)]
	Loc(commands::localization::Localization),
}

/// Delegate execution to the various subcommands.
impl commands::Operation for Cli {
	fn run(self, config: config::Config) -> utility::PinFuture<anyhow::Result<()>> {
		match self {
			Self::InitCfg(cmd) => cmd.run(config),
			Self::Cfg(cmd) => cmd.run(config),
			#[cfg(debug_assertions)]
			Self::Ship(cmd) => cmd.run(config),
			Self::GenProjectFiles(cmd) => cmd.run(config),
			Self::FixupBinaries(cmd) => cmd.run(config),
			Self::Compile(cmd) => cmd.run(config),
			Self::Cook(cmd) => cmd.run(config),
			Self::Editor(cmd) => cmd.run(config),
			Self::Pisep(cmd) => cmd.run(config),
			Self::Loc(cmd) => cmd.run(config),
		}
	}
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let terminate_signal = Arc::new(AtomicBool::new(false));
	let _ = signal_hook::flag::register(signal_hook::consts::SIGINT, terminate_signal.clone());

	let term_handle = tokio::task::spawn(async move {
		while !terminate_signal.load(atomic::Ordering::Relaxed) {
			tokio::time::sleep(std::time::Duration::from_millis(100)).await;
		}
		println!("Encountered terminate signal, cli task will be aborted");
	});
	let cli_handle = tokio::task::spawn(async move {
		if let Err(err) = run_cli().await {
			eprintln!("{err:?}");
		}
	});

	futures::future::select(cli_handle, term_handle).await;

	Ok(())
}

async fn run_cli() -> anyhow::Result<()> {
	use commands::Operation;
	// Load the config from disk
	let config = config::Config::load().await?;
	// Parse the command line args as a cli operation
	let cli = Cli::parse();
	// Construct the error context because `run` takes ownership of `cli`
	let failed_context = format!("failed to run {cli:?}");
	// Actually run the desired commmand with the loaded configuration
	cli.run(config).await.context(failed_context)?;
	Ok(())
}
