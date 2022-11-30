use crate::{utility::PinFuture, Config, Runtime};
use clap::{FromArgMatches, Parser};

pub mod commands;
mod editor_modules;
pub use editor_modules::*;
mod uproject;
pub use uproject::*;
mod editor_target;
pub use editor_target::*;

pub struct Unreal;
impl crate::Plugin for Unreal {
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
	GenProjectFiles(commands::UpdateProjectFiles),
	FixupBinaries(commands::FixupBinaries),
	Compile(commands::Compile),
	Cook(commands::Cook),
	Editor(commands::RunEditor),
	Pisep(commands::RunPisep),
	#[command(subcommand)]
	Loc(commands::localization::Localization),
}

impl crate::Operation for Commands {
	fn run(self, config: Config) -> PinFuture<anyhow::Result<()>> {
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
