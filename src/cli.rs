use crate::{utility::PinFuture, Config, Runtime};
use clap::{FromArgMatches, Parser};

mod release_binary;
use release_binary::*;
mod save_config;
use save_config::*;
mod update_config;
use update_config::*;

pub struct Cli;
impl crate::Plugin for Cli {
	fn add_subcommands(&self, runtime: &mut Runtime, _config: &Config) {
		runtime.augment_subcommands::<Commands>();
	}

	fn run(
		&self,
		matches: &clap::ArgMatches,
		config: &mut Option<Config>,
	) -> Option<PinFuture<anyhow::Result<()>>> {
		use crate::Operation;
		if let Ok(cmds) = Commands::from_arg_matches(matches) {
			return Some(cmds.run(config.take().unwrap()));
		}
		None
	}
}

#[derive(Parser, Debug)]
pub enum Commands {
	InitCfg(SaveToDisk),
	Cfg(Configure),
	#[cfg(debug_assertions)]
	Ship(ReleaseBinary),
}

impl crate::Operation for Commands {
	fn run(self, config: Config) -> PinFuture<anyhow::Result<()>> {
		match self {
			Self::InitCfg(cmd) => cmd.run(config),
			Self::Cfg(cmd) => cmd.run(config),
			#[cfg(debug_assertions)]
			Self::Ship(cmd) => cmd.run(config),
		}
	}
}
