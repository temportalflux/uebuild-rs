use crate::utility::AsUnrealStr;
use clap::ValueEnum;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Platform {
	Windows,
	PS4,
	Switch,
	XboxOne,
	Linux,
}

impl AsUnrealStr for Platform {
	fn as_ue(&self) -> &'static str {
		match self {
			Self::Windows => "Win64",
			Self::PS4 => "PS4",
			Self::Switch => "Switch",
			Self::XboxOne => "XboxOne",
			Self::Linux => "Linux",
		}
	}
}
