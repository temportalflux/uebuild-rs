use std::path::PathBuf;

use crate::{
	config::{self, Config},
	types::Configuration,
	utility::{spawn_command, AsUnrealStr},
};
use clap::{builder::StringValueParser, Parser};
use tokio::process::Command;

/// Run a local play-in-editor instance of the project in a separate editor process (Play In Separate Editor Process).
///
/// Supports both clients and dedicated servers.
#[derive(Parser, Debug)]
pub struct RunPisep {
	/// The configuration that the project should be run in.
	/// The DebugGame editor binary is always used.
	#[clap(short, long, value_enum, default_value_t = Configuration::DebugGame)]
	configuration: Configuration,
	/// Run a pisep dedicated server
	#[clap(short, long)]
	server: bool,

	/// The unreal map level to open when the game begins.
	/// Defaults to the level setting in user preferences based on if this is a server or not.
	#[clap(long, value_parser=MapValueParser)]
	level: Option<PathBuf>,
	/// The game mode alias to run in the level.
	/// Ignored if level is not provided.
	#[clap(long, value_parser=ModeValueParser)]
	mode: Option<String>,
}

#[derive(Clone, Debug)]
struct MapValueParser;
impl clap::builder::TypedValueParser for MapValueParser {
	type Value = PathBuf;

	fn parse_ref(
		&self,
		cmd: &clap::Command,
		arg: Option<&clap::Arg>,
		value: &std::ffi::OsStr,
	) -> Result<Self::Value, clap::Error> {
		let val = StringValueParser::new().parse_ref(cmd, arg, value)?;
		let cfg = config::Config::get_global();
		match cfg.game().maps_by_name().get(&val) {
			Some(&path) => {
				// By unreal convection, map names should be suffixed with `.name`
				// e.g. "/Game/Maps/Level1" => "/Game/Maps/Level1.Level1"
				let map_name = path.file_name().unwrap().to_str().unwrap();
				Ok(path.with_extension(map_name))
			}
			None => Err(clap::Error::new(clap::error::ErrorKind::InvalidValue)),
		}
	}

	fn possible_values(
		&self,
	) -> Option<Box<dyn Iterator<Item = clap::builder::PossibleValue> + '_>> {
		let cfg = config::Config::get_global();
		Some(Box::new(
			cfg.game()
				.maps_by_name()
				.into_iter()
				.map(|(name, _)| name)
				.map(clap::builder::PossibleValue::new),
		))
	}
}

#[derive(Clone, Debug)]
struct ModeValueParser;
impl clap::builder::TypedValueParser for ModeValueParser {
	type Value = String;

	fn parse_ref(
		&self,
		cmd: &clap::Command,
		arg: Option<&clap::Arg>,
		value: &std::ffi::OsStr,
	) -> Result<Self::Value, clap::Error> {
		StringValueParser::new().parse_ref(cmd, arg, value)
	}

	fn possible_values(
		&self,
	) -> Option<Box<dyn Iterator<Item = clap::builder::PossibleValue> + '_>> {
		let cfg = config::Config::get_global();
		Some(Box::new(
			cfg.engine()
				.mode_aliases()
				.iter()
				.map(clap::builder::PossibleValue::new),
		))
	}
}

impl RunPisep {
	fn get_level_arg(&self, config: &Config) -> anyhow::Result<Option<String>> {
		let mut level_arg = Vec::with_capacity(3);
		// Add the level to load
		let map = match (self.level.clone(), self.server) {
			(Some(path), _) => Some(path),
			(None, true) => config.engine().default_map(),
			(None, false) => None,
		};
		if let Some(map) = map {
			level_arg.push(format!("{}", map.display()));
		}
		// Specify the game mode
		if let Some(mode) = self.mode.clone() {
			level_arg.push(format!("?game={mode}"));
		}
		// And always listen for connections
		level_arg.push("?listen".to_owned());

		Ok(match level_arg.is_empty() {
			true => None,
			false => Some(level_arg.join("")),
		})
	}
}

impl super::Operation for RunPisep {
	fn run(self, config: Config) -> crate::utility::PinFuture<anyhow::Result<()>> {
		Box::pin(async move {
			let mut cmd = Command::new(config.editor_binary());
			cmd.current_dir(config.project_root())
				.arg(config.uproject_path());

			cmd.arg(if self.server { "-server" } else { "-game" });
			if let Some(arg) = self.get_level_arg(&config)? {
				cmd.arg(arg);
			}
			cmd.args(&["-stdout", "-AllowStdOutLogVerbosity"]);
			cmd.args(&["-NoEAC", "-messaging"]);
			cmd.arg(format!("RunConfig={}", self.configuration.as_ue()));
			cmd.arg("-debug");

			spawn_command(&mut cmd).await?;
			Ok(())
		})
	}
}
