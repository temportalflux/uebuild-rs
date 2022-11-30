use crate::{
	types::Target,
	unreal::{self, EditorTarget, UProject},
};
use anyhow::Context;
use clap::{Parser, ValueEnum};
use enumset::EnumSetType;
use serde::{Deserialize, Serialize};
use std::{
	collections::{HashMap, HashSet},
	path::{Path, PathBuf},
	str::FromStr,
};

/// An identifier for all supported per-user settings/preferences.
#[derive(Debug, PartialOrd, Ord, ValueEnum, Serialize, Deserialize, EnumSetType)]
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
	/// The name of the `.target.cs` file associated with building the editor.
	ProjectEditorTarget,
	/// The name of the `.target.cs` file associated with building a client.
	ProjectClientTarget,
	/// The name of the `.target.cs` file associated with building a dedicated server.
	ProjectServerTarget,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
	engine_path: PathBuf,
	editor_binary_path: PathBuf,
	project_root: PathBuf,
	project_name: String,
	project_targets: HashMap<Target, String>,
	#[serde(skip)]
	project: UProject,
	#[serde(skip)]
	engine: Engine,
	#[serde(skip)]
	game: Game,
}

impl Config {
	fn instance() -> &'static mut Option<Self> {
		static mut INSTANCE: Option<Config> = None;
		unsafe { &mut INSTANCE }
	}

	pub fn set_global(inst: Self) {
		*Self::instance() = Some(inst);
	}

	pub fn get_global() -> &'static Self {
		Self::instance().as_ref().unwrap()
	}

	pub fn take_global() -> Option<Self> {
		Self::instance().take()
	}
}

impl Config {
	fn exe_name_stem() -> String {
		let mut path = std::env::current_exe().ok().unwrap();
		path.set_extension("");
		path.file_name().unwrap().to_str().unwrap().to_owned()
	}

	fn runtime_root() -> anyhow::Result<PathBuf> {
		Ok(match std::env::var("PROJECT_ROOT") {
			Ok(root) => PathBuf::from_str(&root)?,
			_ => std::env::current_dir()?,
		})
	}

	fn cfg_path() -> anyhow::Result<PathBuf> {
		Ok(Self::runtime_root()?.join(format!("{}-cfg.json", Self::exe_name_stem())))
	}

	pub async fn load() -> anyhow::Result<Self> {
		let cfg_path = Self::cfg_path()?;
		let mut config = match cfg_path.exists() {
			false => Self::generate_from_disk().await?,
			true => {
				let raw_file = tokio::fs::read_to_string(&cfg_path).await?;
				serde_json::from_str::<Self>(&raw_file)?
			}
		};
		config.load_configs().await?;
		Ok(config)
	}

	async fn generate_from_disk() -> anyhow::Result<Self> {
		let cwd = Self::runtime_root()?;
		let mut config = Self::default();

		let uproject_path = match find_uproject(&cwd)? {
			Some(path) => path,
			None => {
				println!(
					"Failed to find uproject file in {cwd}, using empty config.",
					cwd = cwd.display()
				);
				return Ok(Self::default());
			}
		};
		config.project_root = uproject_path.parent().unwrap().to_owned();
		config.project_name = uproject_path
			.file_stem()
			.unwrap()
			.to_str()
			.unwrap()
			.to_owned();

		config.project = unreal::UProject::read(&uproject_path)
			.await
			.context("read uproject")?;
		let engine_path = find_engine_path(
			&config.project_root,
			config.project.get_engine_association(),
		);

		if let Some(path) = engine_path {
			config.engine_path = path.clone();

			let editor_target = EditorTarget::read(
				&path.join(format!("Binaries/Win64/UE4Editor-Win64-DebugGame.target")),
			)
			.await;
			config.editor_binary_path = match editor_target {
				Ok(editor_target) => {
					let binary_path = editor_target.binary_path();
					match binary_path.strip_prefix("$(EngineDir)/") {
						Ok(path) => path.to_owned(),
						_ => binary_path,
					}
				}
				_ => PathBuf::from_str("Binaries/Win64/UE4Editor-Win64-DebugGame.exe").unwrap(),
			};
		}

		config
			.project_targets
			.insert(Target::Client, config.project_name.clone());
		if config
			.project_root
			.join(format!("Source/{}Editor.Target.cs", config.project_name))
			.exists()
		{
			config
				.project_targets
				.insert(Target::Editor, format!("{}Editor", config.project_name));
		}
		if config
			.project_root
			.join(format!("Source/{}Server.Target.cs", config.project_name))
			.exists()
		{
			config
				.project_targets
				.insert(Target::Server, format!("{}Server", config.project_name));
		}

		Ok(config)
	}

	async fn load_configs(&mut self) -> anyhow::Result<()> {
		self.engine = Engine::load(&self.project_root).await?;
		self.game = Game::load(&self.project_root).await?;
		Ok(())
	}

	fn get(&self, key: &Key) -> Option<String> {
		match key {
			Key::EnginePath => Some(format!("{}", self.engine_path.display())),
			Key::EditorBinaryPath => Some(format!("{}", self.editor_binary_path.display())),
			Key::ProjectRoot => Some(format!("{}", self.project_root.display())),
			Key::ProjectName => Some(self.project_name.clone()),
			Key::ProjectEditorTarget => self.project_targets.get(&Target::Editor).cloned(),
			Key::ProjectClientTarget => self.project_targets.get(&Target::Client).cloned(),
			Key::ProjectServerTarget => self.project_targets.get(&Target::Server).cloned(),
		}
	}

	fn set(&mut self, key: &Key, value: String) {
		match key {
			Key::EnginePath => {
				self.engine_path = PathBuf::from_str(&value).unwrap();
			}
			Key::EditorBinaryPath => {
				self.editor_binary_path = PathBuf::from_str(&value).unwrap();
			}
			Key::ProjectRoot => {
				self.project_root = PathBuf::from_str(&value).unwrap();
			}
			Key::ProjectName => {
				self.project_name = value;
			}
			Key::ProjectEditorTarget => {
				self.project_targets.insert(Target::Editor, value);
			}
			Key::ProjectClientTarget => {
				self.project_targets.insert(Target::Client, value);
			}
			Key::ProjectServerTarget => {
				self.project_targets.insert(Target::Server, value);
			}
		}
	}

	async fn save(&self) -> anyhow::Result<()> {
		let cfg_path = Self::cfg_path()?;
		let content = serde_json::to_string_pretty(&self)?;
		println!("Saving current configuration to {:?}", cfg_path);
		tokio::fs::write(&cfg_path, content).await?;
		Ok(())
	}

	pub fn engine_path(&self) -> &PathBuf {
		&self.engine_path
	}

	pub fn editor_binary(&self) -> PathBuf {
		self.engine_path().join(&self.editor_binary_path)
	}

	pub fn project_root(&self) -> &PathBuf {
		&self.project_root
	}

	pub fn uproject_path(&self) -> PathBuf {
		self.project_root()
			.join(&self.project_name)
			.with_extension("uproject")
	}

	pub fn project(&self) -> &UProject {
		&self.project
	}

	pub fn get_project_target(&self, target: Target) -> Option<&String> {
		self.project_targets.get(&target)
	}

	pub fn engine(&self) -> &Engine {
		&self.engine
	}

	pub fn game(&self) -> &Game {
		&self.game
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
					println!("Project:");
					println!("  Root: {:?}", config.project_root());
					println!("  Name: {:?}", config.project_name);
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
					println!("    Server: {:?}", config.engine().default_map_server);
					println!("    Game: {:?}", config.engine().default_map_game);
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

/// Save the dynamically generated config to the current directory.
#[derive(Parser, Debug)]
pub struct SaveToDisk;

impl crate::commands::Operation for SaveToDisk {
	fn run(self, config: Config) -> crate::utility::PinFuture<anyhow::Result<()>> {
		Box::pin(async move {
			config.save().await?;
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

#[derive(Debug, Clone, Default)]
pub struct Engine {
	default_map_server: Option<String>,
	default_map_game: Option<String>,
	mode_aliases: Vec<String>,
}

impl Engine {
	async fn load(project_root: &Path) -> anyhow::Result<Self> {
		let path = project_root.join("Config/DefaultEngine.ini");
		let text = tokio::fs::read_to_string(&path)
			.await
			.context("read DefaultEngine.ini")?;
		let content = ini::Ini::load_from_str(&text)?;

		let mut engine = Self::default();
		if let Some(map_settings) = content.section(Some("/Script/EngineSettings.GameMapsSettings"))
		{
			engine.default_map_server = map_settings.get("ServerDefaultMap").map(str::to_owned);
			engine.default_map_game = map_settings.get("GameDefaultMap").map(str::to_owned);

			let mut all_aliases = HashMap::new();
			for value in map_settings.get_all("+GameModeClassAliases") {
				let value = value.strip_prefix("(Name=\"").unwrap_or(value);
				let value = value.split("\"").next().unwrap();
				all_aliases.insert(
					value.to_owned(),
					match value == value.to_lowercase() {
						true => None,
						false => Some(value.to_lowercase()),
					},
				);
			}
			let mut aliases = HashSet::new();
			for (alias, distinct_lowercase) in all_aliases.iter() {
				if let Some(lowercase) = distinct_lowercase {
					if all_aliases.contains_key(lowercase) {
						continue;
					}
				}
				aliases.insert(alias.clone());
			}
			engine.mode_aliases = aliases.into_iter().collect();
			engine.mode_aliases.sort();
		}

		Ok(engine)
	}

	pub fn default_map(&self) -> Option<PathBuf> {
		let map = self
			.default_map_server
			.as_ref()
			.or(self.default_map_game.as_ref());
		let Some(map) = map else { return None; };
		let Ok(mut path) = PathBuf::from_str(map) else { return None; };
		path.set_extension("");
		Some(path)
	}

	pub fn mode_aliases(&self) -> &Vec<String> {
		&self.mode_aliases
	}
}

#[derive(Debug, Clone, Default)]
pub struct Game {
	maps_to_cook: Vec<PathBuf>,
}

impl Game {
	async fn load(project_root: &Path) -> anyhow::Result<Self> {
		let path = project_root.join("Config/DefaultGame.ini");
		let text = tokio::fs::read_to_string(&path)
			.await
			.context("read DefaultGame.ini")?;
		let content = ini::Ini::load_from_str(&text)?;

		let mut maps_to_cook = Vec::new();
		if let Some(packaging) = content.section(Some("/Script/UnrealEd.ProjectPackagingSettings"))
		{
			for value in packaging.get_all("+MapsToCook") {
				let value = value.strip_prefix("(FilePath=\"").unwrap_or(value);
				let value = value.strip_suffix("\")").unwrap_or(value);
				if let Ok(path) = PathBuf::from_str(value) {
					maps_to_cook.push(path);
				}
			}
		}

		Ok(Self { maps_to_cook })
	}

	pub fn maps_by_name(&self) -> HashMap<String, &PathBuf> {
		self.maps_to_cook
			.iter()
			.map(PathBuf::as_path)
			.filter_map(Path::file_name)
			.filter_map(std::ffi::OsStr::to_str)
			.map(str::to_owned)
			.zip(self.maps_to_cook.iter())
			.collect()
	}
}

pub(crate) fn read_ini_prop<'a>(ini_content: &'a str, prop_name: &str) -> Option<&'a str> {
	let game_default_map = regex::Regex::new(&format!("{prop_name}=(?P<value>.*)")).unwrap();
	let captures = match game_default_map.captures(&ini_content) {
		Some(matches) => matches,
		None => return None,
	};
	captures.name("value").map(|v| v.as_str().trim())
}
