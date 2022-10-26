use crate::utility::AsUnrealStr;
use clap::ValueEnum;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Configuration {
	DebugGame,
	Development,
	Test,
	Shipping,
}

impl AsUnrealStr for Configuration {
	fn as_ue(&self) -> &'static str {
		match self {
			Self::DebugGame => "DebugGame",
			Self::Development => "Development",
			Self::Test => "Test",
			Self::Shipping => "Shipping",
		}
	}
}
