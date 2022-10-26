use crate::utility::AsUnrealStr;
use clap::ValueEnum;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
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
