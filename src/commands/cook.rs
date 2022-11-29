use crate::{
	config::Config,
	types::{Configuration, Platform, Target},
	utility::{spawn_command, AsUnrealStr},
};
use clap::Parser;
use tokio::process::Command;

/// Cooks the project to run standalone.
#[derive(Parser, Debug)]
pub struct Cook {
	#[clap(short, long, value_enum, default_value_t = Target::Client)]
	target: Target,
	#[clap(short, long, value_enum, default_value_t = Platform::Windows)]
	platform: Platform,
	#[clap(short, long, value_enum, default_value_t = Configuration::Development)]
	configuration: Configuration,
	/// Relative path in the project root to output the cooked build to.
	#[clap(short, long, default_value = "DeploymentBuilds")]
	dest: String,
}

impl super::Operation for Cook {
	fn run(self, config: Config) -> crate::utility::PinFuture<anyhow::Result<()>> {
		Box::pin(async move {
			let uat_batch = config.engine_path().join("Build/BatchFiles/RunUAT.bat");
			let deploy_dir = config.project_root().join(self.dest);
			let uproject = config.uproject_path();
			let project_target_name = config.get_project_target(self.target).unwrap();
			let mut cmd = Command::new(uat_batch);
			cmd.current_dir(config.project_root())
				.arg(format!("-ScriptsForProject=\"{}\"", uproject.display()))
				.arg("BuildCookRun")
				.arg(format!("-project=\"{}\"", uproject.display()))
				.arg(format!("-target={}", project_target_name))
				.args(&["-installed", "-nop4"])
				.args(&["-build", "-cook", "-stage"])
				.arg("-archive")
				.arg(format!("-archivedirectory=\"{}\"", deploy_dir.display()))
				.arg("-ddc=InstalledDerivedDataBackendGraph")
				.args(&["-pak", "-prereqs", "-nodebuginfo", "-utf8output"]);
			match self.target {
				Target::Client => {
					cmd.arg(format!("-targetplatform={}", self.platform.as_ue()));
					cmd.arg(format!("-clientconfig={}", self.configuration.as_ue()));
				}
				Target::Server => {
					cmd.args(&["-server", "-noclient"]);
					cmd.arg(format!("-serverplatform={}", self.platform.as_ue()));
					cmd.arg(format!("-platform={}", self.platform.as_ue()));
					cmd.arg(format!("-serverconfig={}", self.configuration.as_ue()));
					cmd.arg(format!(
						"-Target=\"{} {} {}\"",
						project_target_name,
						self.platform.as_ue(),
						self.configuration.as_ue()
					));
				}
				Target::Editor => {
					return Err(InvalidCookTarget)?;
				}
			}
			spawn_command(&mut cmd).await?;
			Ok(())
		})
	}
}

#[derive(thiserror::Error, Debug)]
pub struct InvalidCookTarget;
impl std::fmt::Display for InvalidCookTarget {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"Cooking does not support the {:?} target",
			Target::Editor
		)
	}
}
