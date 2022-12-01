use std::{
	io::Write,
	path::{Path, PathBuf},
	str::FromStr,
};

use anyhow::Context;
use clap::{Parser, Subcommand};
use tokio::process::Command;

use crate::utility::spawn_command;

/// Subcommands to handle localization files.
#[derive(Subcommand, Debug)]
pub enum Localization {
	Gather(Gather),
	Export(Export),
	Compile(Compile),
	Import(Import),
	Update(Update),
	ExportZip(ExportPOZip),
	ImportZip(ImportPOZip),
}

impl crate::Operation for Localization {
	fn run(self, config: crate::config::Config) -> crate::utility::PinFuture<anyhow::Result<()>> {
		match self {
			Self::Gather(cmd) => cmd.run(config),
			Self::Export(cmd) => cmd.run(config),
			Self::Compile(cmd) => cmd.run(config),
			Self::Import(cmd) => cmd.run(config),
			Self::Update(cmd) => cmd.run(config),
			Self::ExportZip(cmd) => cmd.run(config),
			Self::ImportZip(cmd) => cmd.run(config),
		}
	}
}

impl Localization {
	fn make_command(
		config: &crate::config::Config,
		loc_config: &PathBuf,
	) -> anyhow::Result<Command> {
		let mut cmd = Command::new(config.editor_binary());
		cmd.current_dir(config.project_root());
		cmd.arg(config.uproject_path());
		cmd.arg("-run=GatherText");
		cmd.arg(format!("-config={}", loc_config.to_str().unwrap()));
		cmd.arg("-EnableSCC -DisableSCCSubmit");
		cmd.arg("-DisableSCCSubmit");
		Ok(cmd)
	}

	fn make_cfg_ini_path(project_root: Option<&Path>, cfg: &str) -> PathBuf {
		let parent = "Config/Localization";
		let dir = match project_root {
			Some(root) => root.join(parent),
			None => PathBuf::from_str(parent).unwrap(),
		};
		dir.join(cfg)
	}

	async fn make_loc_config(
		config: &crate::config::Config,
		base_name: &str,
		lang: Option<String>,
	) -> anyhow::Result<(PathBuf, Option<PathBuf>)> {
		let base_ini = format!("{base_name}.ini");

		let Some(lang) = lang else {
			return Ok((Self::make_cfg_ini_path(None, &base_ini), None));
		};

		let base_path = Self::make_cfg_ini_path(Some(config.project_root()), &base_ini);
		let base_ini_content = tokio::fs::read_to_string(&base_path)
			.await
			.context(format!("read {base_path:?}"))?;

		let mut mono_lang_content = Vec::new();
		let mut found_lang = false;
		let suffix = format!("={lang}");
		for line in base_ini_content.split('\n') {
			if !line.starts_with("CulturesToGenerate=") {
				mono_lang_content.push(line);
				continue;
			}
			if line.trim().ends_with(&suffix) {
				mono_lang_content.push(line);
				found_lang = true;
			}
		}
		if !found_lang {
			return Err(anyhow::Error::new(std::io::Error::new(
				std::io::ErrorKind::InvalidInput,
				format!("Culture \"{lang}\" is not supported."),
			)));
		}

		let mono_lang_content = mono_lang_content.join("\n");

		let mono_lang_name = format!("{base_name}_{lang}.ini");
		let path_abs = Self::make_cfg_ini_path(Some(&config.project_root()), &mono_lang_name);
		let path_rel = Self::make_cfg_ini_path(None, &mono_lang_name);
		tokio::fs::write(&path_abs, mono_lang_content)
			.await
			.context(format!("write temporary {:?}", path_abs))?;

		Ok((path_rel, Some(path_abs)))
	}

	async fn run_command(
		config: &crate::config::Config,
		base_name: &str,
		lang: Option<String>,
	) -> anyhow::Result<()> {
		let (loc_config, temporary_path) = Self::make_loc_config(&config, base_name, lang)
			.await
			.context("make temporary config")?;

		let mut cmd = Self::make_command(&config, &loc_config).context("make command")?;
		spawn_command(&mut cmd).await.context("run command")?;

		if let Some(temp) = temporary_path {
			tokio::fs::remove_file(temp)
				.await
				.context("remove temporary config")?;
		}

		Ok(())
	}
}

/// [Game -> Archive] Searches through compiled code and assets for localized text. saving detected entries to .archive text files.
#[derive(Parser, Debug)]
pub struct Gather {
	/// Optionally provide the specific language to gather.
	lang: Option<String>,
}

impl crate::Operation for Gather {
	fn run(self, config: crate::config::Config) -> crate::utility::PinFuture<anyhow::Result<()>> {
		Box::pin(async move {
			Localization::run_command(&config, "Game_Gather", self.lang)
				.await
				.context("gather localization")?;
			Ok(())
		})
	}
}

/// [Archive -> PO] Exports gathered archives to human-readable PO files. Updates the 'Game_Conflicts.txt' file.
#[derive(Parser, Debug)]
pub struct Export {
	/// Optionally provide the specific language to gather.
	lang: Option<String>,
}

impl crate::Operation for Export {
	fn run(self, config: crate::config::Config) -> crate::utility::PinFuture<anyhow::Result<()>> {
		Box::pin(async move {
			Localization::run_command(&config, "Game_Export", self.lang)
				.await
				.context("export localization")?;
			Ok(())
		})
	}
}

/// [Archive -> LocRes] Compiles localization archive into binary files for application bundling.
#[derive(Parser, Debug)]
pub struct Compile {
	/// Optionally provide the specific language to gather.
	lang: Option<String>,
}

impl crate::Operation for Compile {
	fn run(self, config: crate::config::Config) -> crate::utility::PinFuture<anyhow::Result<()>> {
		Box::pin(async move {
			Localization::run_command(&config, "Game_Compile", self.lang)
				.await
				.context("compile localization")?;
			Ok(())
		})
	}
}

/// [PO -> Archive] Imports external PO files into the localization archive.
#[derive(Parser, Debug)]
pub struct Import {
	/// Optionally provide the specific language to gather.
	lang: Option<String>,
}

impl Import {
	async fn get_source_path(config: &crate::config::Config) -> anyhow::Result<PathBuf> {
		let import_ini_path =
			Localization::make_cfg_ini_path(Some(&config.project_root()), "Game_Import.ini");
		let content = tokio::fs::read_to_string(&import_ini_path)
			.await
			.context("read Game_Import.ini")?;
		let entry = crate::config::read_ini_prop(&content, "SourcePath");
		Ok(PathBuf::from_str(entry.unwrap()).unwrap())
	}
}

impl crate::Operation for Import {
	fn run(self, config: crate::config::Config) -> crate::utility::PinFuture<anyhow::Result<()>> {
		Box::pin(async move {
			Localization::run_command(&config, "Game_Import", self.lang)
				.await
				.context("import localization")?;
			Ok(())
		})
	}
}

/// [Game -> Archive -> PO & LocRes] Gather, Export, and Compile all current localization.
#[derive(Parser, Debug)]
pub struct Update {
	/// Optionally provide the specific language to gather.
	lang: Option<String>,
}

impl crate::Operation for Update {
	fn run(self, config: crate::config::Config) -> crate::utility::PinFuture<anyhow::Result<()>> {
		Box::pin(async move {
			println!("Updating localization files...\n");
			let gather = Gather {
				lang: self.lang.clone(),
			};
			let export = Export {
				lang: self.lang.clone(),
			};
			let compile = Compile {
				lang: self.lang.clone(),
			};
			println!("Gather:");
			gather.run(config.clone()).await.context("gather")?;
			println!("Export:");
			export.run(config.clone()).await.context("export")?;
			println!("Compile:");
			compile.run(config.clone()).await.context("compile")?;
			println!("Zip PO Files:");
			ExportPOZip.run(config).await.context("zip")?;
			Ok(())
		})
	}
}

/// [Archive -> PO Zip] Exports localization archives and zips the PO fles.
#[derive(Parser, Debug)]
pub struct ExportPOZip;

impl crate::Operation for ExportPOZip {
	fn run(self, config: crate::config::Config) -> crate::utility::PinFuture<anyhow::Result<()>> {
		Box::pin(async move {
			let loc_root = config.project_root().join("Content/Localization/Game");
			let mut lang_paths = Vec::new();
			for entry in std::fs::read_dir(&loc_root)? {
				let entry = entry?;
				if entry.file_type()?.is_dir() {
					lang_paths.push(entry.path());
				}
			}

			let now = chrono::Utc::now().format("%Y-%m-%d").to_string();
			let archive_name = format!("Localization_{now}.zip");
			let archive_path = config
				.project_root()
				.join("Content/Localization")
				.join(archive_name);
			let out_file = std::fs::File::options()
				.create(true)
				.write(true)
				.truncate(true)
				.open(&archive_path)
				.context(format!("open file {archive_path:?}"))?;
			let mut archive = zip::ZipWriter::new(out_file);
			let options = zip::write::FileOptions::default();
			for lang_dir_abs in lang_paths.into_iter() {
				let lang_name = lang_dir_abs.strip_prefix(&loc_root)?.to_str().unwrap();
				archive.add_directory(lang_name, options)?;

				let po_path_abs = lang_dir_abs.join("Game.po");
				let po_bytes = tokio::fs::read(&po_path_abs).await?;
				archive.start_file(&format!("{lang_name}/Game.po"), options)?;
				archive.write_all(&po_bytes[..])?;
			}
			archive.finish()?;

			Ok(())
		})
	}
}

/// [PO Zip -> Archive] Extracts the contents of a PO zip and imports them into localization archive.
#[derive(Parser, Debug)]
pub struct ImportPOZip {
	/// The zip file to import.
	zip_path: PathBuf,
}

impl crate::Operation for ImportPOZip {
	fn run(self, config: crate::config::Config) -> crate::utility::PinFuture<anyhow::Result<()>> {
		Box::pin(async move {
			let import_source_dir = Import::get_source_path(&config)
				.await
				.context("get import source path")?;
			let import_source_dir = config.project_root().join(import_source_dir);
			std::fs::create_dir_all(&import_source_dir).context("create import dir")?;

			self.extract_zip_to(&import_source_dir)
				.context("extract zip")?;

			Import { lang: None }.run(config).await.context("import")?;

			std::fs::remove_dir_all(&import_source_dir)?;
			Ok(())
		})
	}
}
impl ImportPOZip {
	fn extract_zip_to(&self, target_dir: &std::path::Path) -> anyhow::Result<()> {
		let file = std::fs::File::open(&self.zip_path)?;
		let mut archive = zip::ZipArchive::new(file).unwrap();
		for i in 0..archive.len() {
			let mut entry = archive.by_index(i).unwrap();
			if !entry.is_file() {
				continue;
			}
			let path_rel = match entry.enclosed_name() {
				Some(path) => path.to_owned(),
				None => continue,
			};
			if !path_rel.ends_with("Game.po") {
				continue;
			}
			let lang_name = path_rel
				.parent()
				.unwrap()
				.file_name()
				.unwrap()
				.to_str()
				.unwrap();
			let lang_dir = target_dir.join(lang_name);
			std::fs::create_dir_all(&lang_dir)?;
			let target_file_path = lang_dir.join("Game.po");
			let mut target_file = std::fs::File::options()
				.create(true)
				.write(true)
				.truncate(true)
				.open(&target_file_path)
				.context(format!("open {target_file_path:?}"))?;
			std::io::copy(&mut entry, &mut target_file)
				.context(format!("write to {target_file_path:?}"))?;
		}
		Ok(())
	}
}
