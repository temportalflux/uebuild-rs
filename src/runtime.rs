use anyhow::Context;

use crate::{utility::PinFuture, Config, Package, Plugin};
use std::sync::{
	atomic::{self, AtomicBool},
	Arc,
};

pub struct Runtime {
	#[allow(dead_code)]
	package: Package,
	command: Option<clap::Command>,
	plugins: Vec<Arc<dyn Plugin + 'static + Send + Sync>>,
}

impl Runtime {
	pub fn new(package: Package) -> Self {
		Self {
			command: Some(package.new_command()),
			package,
			plugins: Vec::new(),
		}
	}

	pub fn with_plugin(mut self, plugin: impl Plugin + 'static + Send + Sync) -> Self {
		self.plugins.push(Arc::new(plugin));
		self
	}

	pub fn command_mut(&mut self) -> &mut clap::Command {
		self.command.as_mut().unwrap()
	}

	pub fn augment_subcommands<T>(&mut self)
	where
		T: clap::Subcommand,
	{
		self.augment_cmd(T::augment_subcommands);
	}

	pub fn augment_cmd<F>(&mut self, augment: F)
	where
		F: FnOnce(clap::Command) -> clap::Command,
	{
		let command = self.command.take().unwrap();
		self.command = Some(augment(command));
	}

	pub async fn run(self) -> anyhow::Result<()> {
		let terminate_signal = Arc::new(AtomicBool::new(false));
		let _ = signal_hook::flag::register(signal_hook::consts::SIGINT, terminate_signal.clone());

		let term_handle = tokio::task::spawn(async move {
			while !terminate_signal.load(atomic::Ordering::Relaxed) {
				tokio::time::sleep(std::time::Duration::from_millis(100)).await;
			}
			println!("Encountered terminate signal, cli task will be aborted");
		});
		let cli_handle = tokio::task::spawn(async move {
			if let Err(err) = self.execute_cli().await {
				eprintln!("{err:?}");
			}
		});

		futures::future::select(cli_handle, term_handle).await;

		Ok(())
	}

	async fn execute_cli(mut self) -> anyhow::Result<()> {
		// Load any .env file that may or may not exist.
		let _ = dotenv::dotenv();

		let config = Config::load().await?;
		self.load_subcommands(&config);

		// Load the config from disk
		Config::set_global(config);
		// Parse the command line args as a cli operation
		let matches = self.command.take().unwrap().get_matches();

		// Config is kept globally until after parse so that value parsers can use it.
		let config = Config::take_global().unwrap();

		if let Some(future) = self.run_operation(&matches, config) {
			// Construct the error context because `run` takes ownership of `cli`
			let failed_context = format!("failed to run {matches:?}");
			// Actually run the desired commmand with the loaded configuration
			future.await.context(failed_context)?;
		}

		Ok(())
	}

	fn load_subcommands(&mut self, config: &Config) {
		for plugin in self.plugins.clone().into_iter() {
			plugin.add_subcommands(self, config);
		}
	}

	fn run_operation(
		&self,
		matches: &clap::ArgMatches,
		config: Config,
	) -> Option<PinFuture<anyhow::Result<()>>> {
		let mut config = Some(config);
		for plugin in self.plugins.iter() {
			if let Some(future) = plugin.run(matches, &mut config) {
				return Some(future);
			}
		}
		None
	}
}
