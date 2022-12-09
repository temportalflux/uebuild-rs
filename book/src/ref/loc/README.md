# Localization

Subcommands for managing localization.

The localization lifecycle starts with adding a localized field in either the editor or in code via `FText`. These entries are refered to as the `Game` in this tool's documentation.
Localizations in the game are transformed into and stored in `.archive` files, refered to here as the `Archive` state. Archive files contain the engine-readable format where localizations are read from when they are built later.
Localizations which live in `.archive` files are transformed into binary data and saved in `.locres` files (the `LocRes` state). This binary data is what gets packaged with the game binary and is read at runtime, but they are impractical to use within the development cycle because they are binaries.
Localizations also live in PO files (`.po`), which are human-readable and editable transformations of the `.archive` files (refered to as the `PO` state). These files are exported from the game and sent to localizers, and then imported back into the `.archive` files when they have been filled with localized text.

The commands below perform these various transformations, and include additional commands which run a sequence of commands at once or handle the exporting/importing of PO files for localizers.

- [`gather`](gather.md): `Game` ➔ `Archive` - Searches through compiled code and assets for localized text, saving detected entries to `.archive` text files.
- [`compile`](compile.md): `Archive` ➔ `LocRes` - Compiles localization archive into binary files for application bundling.
- [`export`](export.md): `Archive` ➔ `PO` - Exports gathered archives as human-readable PO files. Updates the `Game_Conflicts.txt` file.
- [`import`](import.md): `PO` ➔ `Archive` - Imports external PO files into the localization archive.
- [`export-zip`](export-zip.md): `PO` ➔ `Zip` - Zips the existing PO files for localizers.
- [`import-zip`](import-zip.md): `Zip` ➔ `PO` - Extracts the contents of a PO zip and imports them into localization archives.
- [`update`](update.md): Gather & Compile all supported locales, and then export and zip the PO files for localizers.
