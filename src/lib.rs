pub mod cli;
mod package;
pub use package::*;
mod runtime;
pub use runtime::*;
mod plugin;
pub use plugin::*;

pub mod config;
pub use config::Config;
pub mod types;
pub mod unreal;
pub mod utility;

pub trait Operation {
	fn run(self, config: crate::config::Config) -> crate::utility::PinFuture<anyhow::Result<()>>;
}
