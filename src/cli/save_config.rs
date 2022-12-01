use crate::Config;
use clap::Parser;

/// Save the dynamically generated config to the current directory.
#[derive(Parser, Debug)]
pub struct SaveToDisk;

impl crate::Operation for SaveToDisk {
	fn run(self, config: Config) -> crate::utility::PinFuture<anyhow::Result<()>> {
		Box::pin(async move {
			config.save().await?;
			Ok(())
		})
	}
}
