use uebuild::{cli::Cli, package, unreal::Unreal, Runtime};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	Runtime::new(package!())
		.with_plugin(Cli)
		.with_plugin(Unreal)
		.run()
		.await
}
