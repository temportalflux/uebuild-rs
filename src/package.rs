#[macro_export]
macro_rules! package {
	() => {
		$crate::Package {
			name: env!("CARGO_PKG_NAME"),
			version: env!("CARGO_PKG_VERSION"),
			authors: env!("CARGO_PKG_AUTHORS"),
			description: env!("CARGO_PKG_DESCRIPTION"),
		}
	};
}

#[derive(Debug)]
pub struct Package {
	pub name: &'static str,
	pub version: &'static str,
	pub authors: &'static str,
	pub description: &'static str,
}

impl Package {
	pub(crate) fn new_command(&self) -> clap::Command {
		clap::Command::new(self.name)
			.version(self.version)
			.author(self.authors)
			.about(self.description)
	}
}
