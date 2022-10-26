use crate::{
	types::Target,
	unreal::{self, EditorTarget},
};
use clap::{ValueEnum, Parser};
use serde::{Deserialize, Serialize};
use std::{
	collections::BTreeMap,
	hash::Hash,
	path::{Path, PathBuf},
};

/// An identifier for all supported per-user settings/preferences.
#[derive(
	Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Serialize, Deserialize, Hash,
)]
pub enum Key {
	/// The aboslute path to the engine.
	/// Should be equivalent to "/c/Program Files/Epic Games/UE4.26/".
	/// Directory at path is expected to contain the "Build/BatchFiles/" directory.
	EnginePath,
	EditorBinaryPath,
	/// The absolute path to the directory containing the `.uproject` file that commands will refer to.
	/// Settings are meant to be configured on a per-project basis.
	ProjectRoot,
	/// The name of the `.uproject` file (without the suffix).
	ProjectName,
	/// The unreal asset path for the default level of a dedicated server.
	DefaultServerLevel,
	/// The name of the `.target.cs` file associated with building the editor.
	ProjectEditorTarget,
	/// The name of the `.target.cs` file associated with building a client.
	ProjectClientTarget,
	/// The name of the `.target.cs` file associated with building a dedicated server.
	ProjectServerTarget,
}

pub struct Config(Data);
type Data = BTreeMap<Key, String>;

impl Default for Config {
	fn default() -> Self {
		let mut data = Data::new();
		data.insert(Key::EnginePath, "".to_owned());
		data.insert(Key::EditorBinaryPath, "".to_owned());
		data.insert(Key::ProjectRoot, "".to_owned());
		data.insert(Key::ProjectName, "".to_owned());
		data.insert(Key::DefaultServerLevel, "".to_owned());
		data.insert(Key::ProjectEditorTarget, "".to_owned());
		data.insert(Key::ProjectClientTarget, "".to_owned());
		data.insert(Key::ProjectServerTarget, "".to_owned());
		Self(data)
	}
}

impl Config {
	fn cfg_path() -> anyhow::Result<PathBuf> {
		let mut path = std::env::current_dir()?;
		path.push("ubuild-cfg.json");
		Ok(path)
	}

	pub async fn load() -> anyhow::Result<Self> {
		let cfg_path = Self::cfg_path()?;
		match cfg_path.exists() {
			false => {
				println!("No ubuild-cfg.json found, generating one now.");
				let config = Self::generate_from_disk().await?;
				config.save().await?;
				Ok(config)
			}
			true => {
				let raw_file = tokio::fs::read_to_string(&cfg_path).await?;
				let data = serde_json::from_str::<Data>(&raw_file)?;
				Ok(Self(data))
			}
		}
	}

	async fn generate_from_disk() -> anyhow::Result<Self> {
		let cwd = std::env::current_dir()?;

		let uproject_path = match find_uproject(&cwd)? {
			Some(path) => {
				println!("Found uproject file at {}", path.display());
				path
			}
			None => {
				println!(
					"Failed to find uproject file in {cwd}, using empty config.",
					cwd = cwd.display()
				);
				return Ok(Self::default());
			}
		};
		let project_root = uproject_path.parent().unwrap().to_owned();
		let project_name = uproject_path
			.file_stem()
			.unwrap()
			.to_str()
			.unwrap()
			.to_owned();
		let default_map = find_default_map(&project_root).await?;

		let uproject = unreal::UProject::read(&uproject_path).await?;
		let engine_path = find_engine_path(&project_root, uproject.get_engine_association());

		let mut config = Self::default();
		if let Some(path) = engine_path {
			config.insert(Key::EnginePath, &path);
		}

		let editor_target = EditorTarget::read(&project_root.join(format!(
			"Binaries/Win64/{project_name}Editor-Win64-DebugGame.target"
		)))
		.await?;
		config.insert(Key::EditorBinaryPath, &editor_target.binary_path());

		config.insert(Key::ProjectRoot, &project_root);
		config.0.insert(Key::ProjectName, project_name.clone());
		config
			.0
			.insert(Key::ProjectClientTarget, project_name.clone());
		if project_root
			.join(format!("Source/{project_name}Editor.Target.cs"))
			.exists()
		{
			config
				.0
				.insert(Key::ProjectEditorTarget, format!("{project_name}Editor"));
		}
		if project_root
			.join(format!("Source/{project_name}Server.Target.cs"))
			.exists()
		{
			config
				.0
				.insert(Key::ProjectServerTarget, format!("{project_name}Server"));
		}
		if let Some(asset_path) = default_map {
			config.0.insert(Key::DefaultServerLevel, asset_path);
		}
		Ok(config)
	}

	fn insert(&mut self, key: Key, path: &Path) {
		self.0.insert(key, format!("{}", path.display()));
	}

	async fn save(&self) -> anyhow::Result<()> {
		let cfg_path = Self::cfg_path()?;
		let content = serde_json::to_string_pretty(&self.0)?;
		tokio::fs::write(&cfg_path, content).await?;
		Ok(())
	}

	pub fn get(&self, key: Key) -> Result<&String, MissingValue> {
		self.0.get(&key).ok_or(MissingValue(key))
	}

	pub fn engine_path(&self) -> Result<PathBuf, MissingValue> {
		Ok(PathBuf::from(self.get(Key::EnginePath)?))
	}

	pub fn editor_binary(&self) -> Result<PathBuf, MissingValue> {
		let mut path = self.engine_path()?;
		path.push(self.get(Key::EditorBinaryPath)?);
		Ok(path)
	}

	pub fn project_root(&self) -> Result<PathBuf, MissingValue> {
		Ok(PathBuf::from(self.get(Key::ProjectRoot)?))
	}

	pub fn uproject_path(&self) -> Result<PathBuf, MissingValue> {
		Ok({
			let mut path = self.project_root()?;
			path.push(self.get(Key::ProjectName)?);
			path.set_extension("uproject");
			path
		})
	}

	pub fn get_project_target(&self, target: Target) -> Result<&String, MissingValue> {
		self.get(match target {
			Target::Editor => Key::ProjectEditorTarget,
			Target::Client => Key::ProjectClientTarget,
			Target::Server => Key::ProjectServerTarget,
		})
	}
}

#[derive(thiserror::Error, Debug)]
pub struct MissingValue(Key);
impl std::fmt::Display for MissingValue {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Missing config value for key {:?}", self.0)
	}
}

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

impl crate::commands::Operation for Configure {
	fn run(self, mut config: Config) -> crate::utility::PinFuture<anyhow::Result<()>> {
		Box::pin(async move {
			match (self.key, self.value) {
				(None, _) => {
					for (key, value) in config.0.iter() {
						println!("{:?} => {:?}", key, value);
					}
				}
				(Some(key), None) => {
					println!("{:?} => {:?}", key, config.get(key).ok());
				}
				(Some(key), Some(value)) => {
					config.0.insert(key, value);
					config.save().await?;
				}
			}
			Ok(())
		})
	}
}

fn find_uproject(cwd: &Path) -> anyhow::Result<Option<PathBuf>> {
	if let Some(path) = glob_path_exists(&cwd, "/*.uproject")? {
		return Ok(Some(path));
	}
	if let Some(path) = glob_path_exists(&cwd, "/Game/*.uproject")? {
		return Ok(Some(path));
	}
	Ok(None)
}

fn glob_path_exists(root: &Path, glob_fmt: &str) -> Result<Option<PathBuf>, glob::PatternError> {
	for entry in glob::glob(&format!("{dir}{glob_fmt}", dir = root.display()))? {
		if let Ok(path) = entry {
			return Ok(Some(path));
		}
	}
	Ok(None)
}

fn find_engine_path(project_root: &Path, engine_association: Option<&String>) -> Option<PathBuf> {
	match engine_association {
		Some(version) => {
			// Using a pre-installed engine. Check program files for installment.
			let path = PathBuf::from(format!(
				"C:\\Program Files\\Epic Games\\UE_{version}\\Engine"
			));
			if path.exists() {
				Some(path)
			} else {
				println!(
					"Failed to find pre-installed engine for v{version} at {}",
					path.display()
				);
				None
			}
		}
		None => {
			// Must be using a custom engine. We should check the parent directory for an "Engine" dir and contents.
			let mut path = project_root.parent().unwrap().to_owned();
			path.push("Engine");
			if path.exists() {
				Some(path)
			} else {
				println!("Failed to find custom engine at {}", path.display());
				None
			}
		}
	}
}

async fn find_default_map(project_root: &Path) -> anyhow::Result<Option<String>> {
	let engine_ini = project_root.join("Config/DefaultEngine.ini");
	let content = tokio::fs::read_to_string(&engine_ini).await?;
	if let Some(default_map) = read_ini_prop(&content, "ServerDefaultMap") {
		return Ok(Some(default_map.to_owned()));
	}
	if let Some(default_map) = read_ini_prop(&content, "GameDefaultMap") {
		return Ok(Some(default_map.to_owned()));
	}
	Ok(None)
}

fn read_ini_prop<'a>(ini_content: &'a str, prop_name: &str) -> Option<&'a str> {
	let game_default_map = regex::Regex::new(&format!("{prop_name}=(?P<value>.*)")).unwrap();
	let captures = match game_default_map.captures(&ini_content) {
		Some(matches) => matches,
		None => return None,
	};
	captures.name("value").map(|v| v.as_str().trim())
}
