use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{types::Platform, utility::AsUnrealStr};

#[derive(Debug, Serialize, Deserialize)]
pub struct UProject {
	#[serde(rename = "EngineAssociation")]
	pub engine_association: String,
	#[serde(rename = "Plugins")]
	pub plugins: Vec<Plugin>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Plugin {
	#[serde(rename = "Name")]
	pub name: String,
}

impl UProject {
	pub async fn read(path: &Path) -> anyhow::Result<Self> {
		let json = tokio::fs::read_to_string(path).await?;
		Ok(serde_json::from_str::<Self>(&json)?)
	}

	pub fn get_engine_association(&self) -> Option<&String> {
		match self.engine_association.is_empty() {
			true => None,
			false => Some(&self.engine_association),
		}
	}

	pub fn get_module_paths(&self, platform: Platform) -> Vec<PathBuf> {
		let mut subpaths = vec![PathBuf::new()];
		for plugin in &self.plugins {
			subpaths.push(PathBuf::from(format!("Plugins/{name}", name = plugin.name)));
		}
		let platform_str = platform.as_ue();
		subpaths
			.into_iter()
			.map(|mut subpath| {
				subpath.push(format!("Binaries/{platform_str}/UE4Editor.modules"));
				subpath
			})
			.collect()
	}
}
