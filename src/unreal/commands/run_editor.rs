use crate::{config::Config, utility::spawn_command};
use clap::Parser;
use tokio::process::Command;

/// Opens the uproject in the unreal editor.
#[derive(Parser, Debug)]
pub struct RunEditor;

impl crate::Operation for RunEditor {
	fn run(self, config: Config) -> crate::utility::PinFuture<anyhow::Result<()>> {
		Box::pin(async move {
			spawn_command(
				Command::new(config.editor_binary())
					.current_dir(config.project_root())
					.arg(config.uproject_path())
					.arg("-debug")
					.args(&["-stdout", "-AllowStdOutLogVerbosity"]),
			)
			.await?;
			Ok(())
		})
	}
}
