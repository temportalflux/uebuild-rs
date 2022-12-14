use crate::utility::AsUnrealStr;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(
	Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Hash, Serialize, Deserialize,
)]
pub enum Target {
	Editor,
	Client,
	Server,
}

impl AsUnrealStr for Target {
	fn as_ue(&self) -> &'static str {
		match self {
			Self::Editor => "editor",
			Self::Client => "client",
			Self::Server => "server",
		}
	}
}
