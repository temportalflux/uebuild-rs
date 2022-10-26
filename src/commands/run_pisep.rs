use crate::{
	config::{self, Config},
	types::Configuration,
	utility::{spawn_command, AsUnrealStr},
};
use clap::Parser;
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
	#[clap(long)]
	level: Option<String>,
	/// The game mode alias to run in the level.
	/// Ignored if level is not provided.
	#[clap(long)]
	mode: Option<String>,
}

impl RunPisep {
	fn get_level_arg(&self, config: &Config) -> anyhow::Result<Option<String>> {
		let mut level_arg = Vec::with_capacity(3);
		if let Some(level) = self.level.clone() {
			level_arg.push(level);
		} else if self.server {
			level_arg.push(config.get(config::Key::DefaultServerLevel)?.to_owned());
		}
		if let Some(mode) = self.mode.clone() {
			level_arg.push(format!("?game={mode}"));
		}
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
			let mut cmd = Command::new(config.editor_binary()?);
			cmd.current_dir(config.project_root()?)
				.arg(config.uproject_path()?);

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
