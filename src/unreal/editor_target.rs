use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct EditorTarget {
	#[serde(rename = "Launch")]
	pub binary_path: String,
}

impl EditorTarget {
	pub async fn read(path: &Path) -> anyhow::Result<Self> {
		let json = tokio::fs::read_to_string(path).await?;
		Ok(serde_json::from_str::<Self>(&json)?)
	}

	pub fn binary_path(&self) -> PathBuf {
		PathBuf::from(self.binary_path.strip_prefix("$(EngineDir)/").unwrap())
	}
}
