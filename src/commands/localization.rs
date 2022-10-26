use clap::Parser;

/* Run Localization
	Args:
		uproject| Absolute path to .uproject file
		loc_script| Name of a localization script ini file
	Command:
		Engine/Binaries/Win64/UE4Editor-Win64-DebugGame.exe
			{uproject}
			-run=GatherText -config=Config/Localization/{loc_script}
			EnableSCC DisableSCCSubmit
*/
#[derive(Parser, Debug)]
pub struct UpdateLocalization;

impl super::Operation for UpdateLocalization {
	fn run(self, _config: crate::config::Config) -> crate::utility::PinFuture<anyhow::Result<()>> {
		Box::pin(async move { Ok(()) })
	}
}
