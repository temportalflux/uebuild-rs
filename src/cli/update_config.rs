use crate::{config::Key, types::Target, Config};
use clap::Parser;

/// Handle changes to the user preferences/configuration for this project.
///
/// Without a key, will present the entire configuration.
///
/// With key but no value with present the value for that key.
///
/// With key and value will update the config with the value.
#[derive(Parser, Debug)]
pub struct Configure {
	#[clap(value_enum)]
	key: Option<Key>,
	value: Option<String>,
}

impl crate::Operation for Configure {
	fn run(self, mut config: Config) -> crate::utility::PinFuture<anyhow::Result<()>> {
		Box::pin(async move {
			match (self.key, self.value) {
				(None, _) => {
					println!("Project:");
					println!("  Root: {:?}", config.project_root());
					println!("  Name: {:?}", config.project_name());
					println!("  UProject: {:?}", config.uproject_path());
					println!("  Targets:");
					println!(
						"    Editor: {:?}",
						config.get_project_target(Target::Editor)
					);
					println!(
						"    Client: {:?}",
						config.get_project_target(Target::Client)
					);
					println!(
						"    Server: {:?}",
						config.get_project_target(Target::Server)
					);
					println!("Engine:");
					println!("  Path: {:?}", config.engine_path());
					println!("  Default Maps:");
					println!("    Server: {:?}", config.engine().default_map_server());
					println!("    Game: {:?}", config.engine().default_map_game());
					println!("  Modes: {}", config.engine().mode_aliases().join(", "));
					println!("Game:");
					println!("  Maps:");
					{
						let maps_by_name = config.game().maps_by_name();
						let maps = {
							let mut maps = maps_by_name.iter().collect::<Vec<_>>();
							maps.sort_by_key(|(name, _)| *name);
							maps
						};
						for (name, path) in maps.into_iter() {
							println!("    {:?} => {:?}", name, path);
						}
					}
					println!("Editor:");
					println!("  Binary Path: {:?}", config.editor_binary());
				}
				(Some(key), None) => {
					println!("{:?} => {:?}", key, config.get(&key));
				}
				(Some(key), Some(value)) => {
					config.set(&key, value);
					config.save().await?;
				}
			}
			Ok(())
		})
	}
}
