use anyhow::Context;
use clap::{FromArgMatches, Parser, Subcommand};
use commands::Operation;
use std::sync::{
	atomic::{self, AtomicBool},
	Arc,
};

pub mod commands;
pub mod config;
pub mod types;
pub mod unreal;
pub mod utility;

#[macro_export]
macro_rules! package {
	() => {
		$crate::Package {
			name: env!("CARGO_PKG_NAME"),
			version: env!("CARGO_PKG_VERSION"),
			authors: env!("CARGO_PKG_AUTHORS"),
			description: env!("CARGO_PKG_DESCRIPTION"),
		}
	};
}

#[derive(Debug)]
pub struct Package {
	pub name: &'static str,
	pub version: &'static str,
	pub authors: &'static str,
	pub description: &'static str,
}
impl Package {
	fn new_command(&self) -> clap::Command {
		clap::Command::new(self.name)
			.version(self.version)
			.author(self.authors)
			.about(self.description)
	}
}

pub struct Runtime {
	#[allow(dead_code)]
	package: Package,
	command: Option<clap::Command>,
	plugins: Vec<Arc<dyn RuntimePlugin + 'static + Send + Sync>>,
}

pub trait RuntimePlugin {
	fn add_subcommands(&self, runtime: &mut Runtime, config: &config::Config);
	fn run(
		&self,
		matches: &clap::ArgMatches,
		config: &mut Option<config::Config>,
	) -> Option<utility::PinFuture<anyhow::Result<()>>>;
}

pub struct UnrealPlugin;
impl RuntimePlugin for UnrealPlugin {
	fn add_subcommands(&self, runtime: &mut Runtime, _config: &config::Config) {
		runtime.augment_subcommands(CliCommands::augment_subcommands);
		runtime.augment_subcommands(UnrealCommands::augment_subcommands);
	}

	fn run(
		&self,
		matches: &clap::ArgMatches,
		config: &mut Option<config::Config>,
	) -> Option<utility::PinFuture<anyhow::Result<()>>> {
		if let Ok(cmds) = CliCommands::from_arg_matches(matches) {
			return Some(cmds.run(config.take().unwrap()));
		}
		if let Ok(cmds) = UnrealCommands::from_arg_matches(matches) {
			return Some(cmds.run(config.take().unwrap()));
		}
		None
	}
}

#[derive(Parser, Debug)]
pub enum CliCommands {
	InitCfg(config::SaveToDisk),
	Cfg(config::Configure),
	#[cfg(debug_assertions)]
	Ship(commands::ReleaseBinary),
}
impl commands::Operation for CliCommands {
	fn run(self, config: config::Config) -> utility::PinFuture<anyhow::Result<()>> {
		match self {
			Self::InitCfg(cmd) => cmd.run(config),
			Self::Cfg(cmd) => cmd.run(config),
			#[cfg(debug_assertions)]
			Self::Ship(cmd) => cmd.run(config),
		}
	}
}

#[derive(Parser, Debug)]
pub enum UnrealCommands {
	GenProjectFiles(commands::UpdateProjectFiles),
	FixupBinaries(commands::FixupBinaries),
	Compile(commands::Compile),
	Cook(commands::Cook),
	Editor(commands::RunEditor),
	Pisep(commands::RunPisep),
	#[command(subcommand)]
	Loc(commands::localization::Localization),
}
impl commands::Operation for UnrealCommands {
	fn run(self, config: config::Config) -> utility::PinFuture<anyhow::Result<()>> {
		match self {
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

impl Runtime {
	pub fn new(package: Package) -> Self {
		Self {
			command: Some(package.new_command()),
			package,
			plugins: Vec::new(),
		}
	}

	pub fn with_plugin(mut self, plugin: impl RuntimePlugin + 'static + Send + Sync) -> Self {
		self.plugins.push(Arc::new(plugin));
		self
	}

	pub fn command_mut(&mut self) -> &mut clap::Command {
		self.command.as_mut().unwrap()
	}

	pub fn augment_subcommands<F>(&mut self, augment: F)
	where
		F: FnOnce(clap::Command) -> clap::Command,
	{
		let command = self.command.take().unwrap();
		self.command = Some(augment(command));
	}

	pub async fn run(self) -> anyhow::Result<()> {
		let terminate_signal = Arc::new(AtomicBool::new(false));
		let _ = signal_hook::flag::register(signal_hook::consts::SIGINT, terminate_signal.clone());

		let term_handle = tokio::task::spawn(async move {
			while !terminate_signal.load(atomic::Ordering::Relaxed) {
				tokio::time::sleep(std::time::Duration::from_millis(100)).await;
			}
			println!("Encountered terminate signal, cli task will be aborted");
		});
		let cli_handle = tokio::task::spawn(async move {
			if let Err(err) = self.execute_cli().await {
				eprintln!("{err:?}");
			}
		});

		futures::future::select(cli_handle, term_handle).await;

		Ok(())
	}

	async fn execute_cli(mut self) -> anyhow::Result<()> {
		// Load any .env file that may or may not exist.
		let _ = dotenv::dotenv();

		let config = config::Config::load().await?;
		self.load_subcommands(&config);

		// Load the config from disk
		config::Config::set_global(config);
		// Parse the command line args as a cli operation
		let matches = self.command.take().unwrap().get_matches();

		// Config is kept globally until after parse so that value parsers can use it.
		let config = config::Config::take_global().unwrap();

		if let Some(future) = self.run_operation(&matches, config) {
			// Construct the error context because `run` takes ownership of `cli`
			let failed_context = format!("failed to run {matches:?}");
			// Actually run the desired commmand with the loaded configuration
			future.await.context(failed_context)?;
		}

		Ok(())
	}

	fn load_subcommands(&mut self, config: &config::Config) {
		for plugin in self.plugins.clone().into_iter() {
			plugin.add_subcommands(self, config);
		}
	}

	fn run_operation(
		&self,
		matches: &clap::ArgMatches,
		config: config::Config,
	) -> Option<utility::PinFuture<anyhow::Result<()>>> {
		let mut config = Some(config);
		for plugin in self.plugins.iter() {
			if let Some(future) = plugin.run(matches, &mut config) {
				return Some(future);
			}
		}
		None
	}
}
