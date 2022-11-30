use uebuild::{package, Runtime, UnrealPlugin};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	Runtime::new(package!())
		.with_plugin(UnrealPlugin)
		.run()
		.await
}
