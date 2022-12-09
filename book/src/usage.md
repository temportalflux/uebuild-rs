# Usage

After creating an unreal engine project, you can use this tool to run commands against the build script or engine/editor binaries. While the current working directory contains a project (i.e. there is a uproject file at either `./*.uproject` for pre-compiled editors or `./Game/*.uproject` for embedded editors), you can run the following commands:

- [`uebuild [help/--help/-h]`](../ref/help.md): Display the help menu
- [`uebuild gen-project-files`](../ref/compilation/gen-project-files.md): Generate the project files (e.g. `.sln`)
- [`uebuild compile`](../ref/compilation/compile.md): Compiles the code for the project (and the editor if it is embedded)
- [`uebuild editor`](../ref/run/editor.md): Opens the uproject in the unreal editor
- [`uebuild pisep`](../ref/run/pisep.md): Run a local play-in-editor instance of the project in a separate editor process (Play In Separate Editor Process)
- [`uebuild cook`](../ref/cook.md): Cooks the project to run standalone
- [`uebuild loc gather`](../ref/loc/gather.md): Searches through compiled code and assets for localized text, saving detected entries to `.archive` text files. Performs the `Game -> Archive` transformation.
- [`uebuild loc export`](../ref/loc/export.md): Exports gathered archives to human-readable PO files. Updates the 'Game_Conflicts.txt' file. Performs the `Archive -> PO` transformation.
- [`uebuild loc compile`](../ref/loc/compile.md): Compiles localization archive into binary files for application bundling. Performs the `Archive -> LocRes` transformation.
- [`uebuild loc import`](../ref/loc/import.md): Imports external PO files into the localization archive. Performs the `PO -> Archive` transformation.
- [`uebuild loc update`](../ref/loc/update.md): Gather, Export, and Compile all supported locales, and then Export-Zip the PO files.
- [`uebuild loc export-zip`](../ref/loc/export-zip.md): Exports localization archives and zips the PO fles.
- [`uebuild loc import-zip`](../ref/loc/import-zip.md): Extracts the contents of a PO zip and imports them into localization archives.

When any command is run, the tool will attempt to determine metadata about the project like:
* Path to the `.uproject` file
* Path to the engine directory (if version is specified in `.uproject`, then its a pre-installed engine, otherwise its at `<root>/.uproject/../../Engine`)
* Path to the editor binary executable (based on engine path)
* All of the project targets (`*.Target.cs`)

From those configuration details, the tool can further find information about:
* `.ini` file configurations (Engine, Game, etc)
