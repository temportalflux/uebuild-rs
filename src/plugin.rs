use crate::{utility::PinFuture, Config, Runtime};

pub trait Plugin {
	fn add_subcommands(&self, runtime: &mut Runtime, config: &Config);
	fn run(
		&self,
		matches: &clap::ArgMatches,
		config: &mut Option<Config>,
	) -> Option<PinFuture<anyhow::Result<()>>>;
}
