use crate::utility::{spawn_command, PinFuture};
use clap::Parser;
use tokio::process::Command;

/// [DEBUG ONLY] Compile uebuild as a binary and copy it to the project's root directory.
#[derive(Parser, Debug)]
pub struct ReleaseBinary {
	#[clap(short, long, default_value = "uebuild")]
	name: String,
}

impl crate::Operation for ReleaseBinary {
	fn run(self, config: crate::config::Config) -> PinFuture<anyhow::Result<()>> {
		Box::pin(async move {
			let out_path = config.project_root().join(self.name).with_extension("exe");

			let cwd = std::env::current_dir()?;
			spawn_command(
				Command::new("cargo")
					.args(&["build", "--release"])
					.current_dir(cwd.clone()),
			)
			.await?;

			spawn_command(
				Command::new("cp")
					.arg("./target/release/uebuild.exe")
					.arg(format!("{}", out_path.display()))
					.current_dir(cwd.clone()),
			)
			.await?;

			Ok(())
		})
	}
}
