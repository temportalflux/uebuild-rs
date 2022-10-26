use serde::{Deserialize, Serialize};
use std::{
	collections::HashMap,
	path::{Path, PathBuf},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct BinaryModule {
	#[serde(rename = "BuildId")]
	pub build_id: String,
	#[serde(rename = "Modules")]
	pub modules: HashMap<String, String>,
}

impl BinaryModule {
	pub async fn read(path: &Path) -> anyhow::Result<Self> {
		let json = tokio::fs::read_to_string(path).await?;
		Ok(serde_json::from_str::<Self>(&json)?)
	}

	pub async fn write(&self, path: &Path) -> anyhow::Result<()> {
		let json = serde_json::to_string_pretty(&self)?;
		let json = json.replace("  ", "\t");
		tokio::fs::write(&path, json).await?;
		Ok(())
	}

	pub fn dealias(&mut self) -> Vec<(PathBuf, PathBuf)> {
		let mut redirectors = Vec::new();
		for (module_name, binary_pathname) in self.modules.iter_mut() {
			let desired_name = format!("UE4Editor-{}.dll", module_name);
			if *binary_pathname != desired_name {
				let mut existing_path = PathBuf::from(binary_pathname.clone());
				let mut desired_path = PathBuf::from(desired_name.clone());
				redirectors.push((existing_path.clone(), desired_path.clone()));

				existing_path.set_extension("pdb");
				desired_path.set_extension("pdb");
				redirectors.push((existing_path, desired_path));

				*binary_pathname = desired_name;
			}
		}
		redirectors
	}
}
