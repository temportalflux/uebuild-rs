use crate::{config::Config, utility::spawn_command};
use clap::Parser;
use tokio::process::Command;

/// Generate the project files (e.g. ".sln")
#[derive(Parser, Debug)]
pub struct UpdateProjectFiles;

impl super::Operation for UpdateProjectFiles {
	fn run(self, config: Config) -> crate::utility::PinFuture<anyhow::Result<()>> {
		Box::pin(async move {
			let root = config.project_root()?;
			let batch = {
				let mut path = root.clone();
				path.push("GenerateProjectFiles.bat");
				path
			};
			spawn_command(Command::new(batch).current_dir(root)).await?;
			Ok(())
		})
	}
}
