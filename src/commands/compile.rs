use crate::{
	config::Config,
	types::{Configuration, Platform, Target},
	unreal::{BinaryModule, UProject},
	utility::{spawn_command, AsUnrealStr},
};
use anyhow::Context;
use clap::Parser;
use tokio::process::Command;

/// Compiles the code for the project.
#[derive(Parser, Debug)]
pub struct Compile {
	#[clap(short, long, value_enum, default_value_t = Target::Editor)]
	target: Target,
	#[clap(short, long, value_enum, default_value_t = Platform::Windows)]
	platform: Platform,
	#[clap(short, long, value_enum, default_value_t = Configuration::DebugGame)]
	configuration: Configuration,
}

impl super::Operation for Compile {
	fn run(self, config: Config) -> crate::utility::PinFuture<anyhow::Result<()>> {
		Box::pin(async move {
			let build_batch = {
				let mut path = config.engine_path()?;
				path.push("Build/BatchFiles/Build.bat");
				path
			};
			let project_target_name = config.get_project_target(self.target)?;
			spawn_command(
				Command::new(build_batch)
					.current_dir(config.project_root()?)
					.arg(project_target_name)
					.arg(self.configuration.as_ue())
					.arg(self.platform.as_ue())
			)
			.await?;
			Ok(())
		})
	}
}

/// Dealiases the binaries for the project and its plugins.
#[derive(Parser, Debug)]
pub struct FixupBinaries;
impl super::Operation for FixupBinaries {
	fn run(self, config: Config) -> crate::utility::PinFuture<anyhow::Result<()>> {
		Box::pin(async move {
			let project_root = config.project_root()?;
			let uproject = UProject::read(&config.uproject_path()?).await?;
			let module_paths = uproject.get_module_paths(Platform::Windows);

			let mut files_to_checkout = Vec::new();
			let mut modules_to_update = Vec::new();
			for module_path in module_paths.into_iter() {
				let module_path = project_root.join(&module_path);
				let module_dir = module_path.parent().unwrap();
				if !module_path.exists() {
					continue;
				}
				let mut module = BinaryModule::read(&module_path).await?;
				let binary_redirectors = module.dealias();
				if !binary_redirectors.is_empty() {
					files_to_checkout.push(module_path.clone());
					let mut binaries = Vec::with_capacity(binary_redirectors.len());
					for (prev, next) in binary_redirectors.into_iter() {
						files_to_checkout.push(module_dir.join(&next));
						binaries.push((module_dir.join(&prev), module_dir.join(&next)));
					}
					modules_to_update.push((module_path.clone(), module, binaries));
				}
			}

			if !files_to_checkout.is_empty() {
				let files_to_checkout = files_to_checkout
					.into_iter()
					.map(|path| format!("{}", path.display()).replace("/", "\\"))
					.collect::<Vec<_>>();
				spawn_command(
					Command::new("p4.exe")
						.current_dir(project_root)
						.arg("edit")
						.args(&files_to_checkout),
				)
				.await?;
			}

			for (module_path, module, binaries_to_move) in modules_to_update.into_iter() {
				for (prev, next) in binaries_to_move.into_iter() {
					if prev != next {
						spawn_command(
							Command::new("chmod")
								.arg("777")
								.arg(format!("{}", prev.display())),
						)
						.await?;

						spawn_command(
							Command::new("chmod")
								.arg("777")
								.arg(format!("{}", next.display())),
						)
						.await?;

						println!("Moving {prev:?} to {next:?}");
						tokio::fs::copy(&prev, &next)
							.await
							.context("failed to copy file contents")?;
						tokio::fs::remove_file(&prev)
							.await
							.context("failed to remove source file")?;
					}
				}

				println!("Writing updates to {module_path:?}");
				module.write(&module_path).await?;
			}

			Ok(())
		})
	}
}
