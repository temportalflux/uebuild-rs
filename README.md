# uebuild-rs (Unreal Engine Build)

[<img alt="github" src="https://img.shields.io/badge/github-temportalflux/uebuild-rs-8da0cb?logo=github" height="20">](https://github.com/temportalflux/uebuild-rs)
[![Latest version](https://img.shields.io/crates/v/uebuild.svg)](https://crates.io/crates/uebuild)
[![Documentation](https://docs.rs/uebuild/badge.svg)](https://docs.rs/uebuild)
[![Build Status](https://github.com/temportalflux/uebuild-rs/workflows/CI/badge.svg)](https://github.com/temportalflux/uebuild-rs/actions?workflow=CI)
[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/temportalflux/uebuild-rs/blob/master/LICENSE-MIT)
[![Apache](https://img.shields.io/badge/license-Apache-blue.svg)](https://github.com/temportalflux/uebuild-rs/blob/master/LICENSE-APACHE)

Unreal Engine Build is a Command Line Interface written in Rust which wraps the CLI provided by Unreal Engine.

Supported UE commands:
- `gen-project-files`: Generate the project files (e.g. ".sln")
- `compile`: Compiles the code for the project
- `cook`: Cooks the project to run standalone
- `editor`: Opens the uproject in the unreal editor
- `pisep`: Run a local play-in-editor instance of the project in a separate editor process (Play In Separate Editor Process)
- `loc gather` : [Game -> Archive] Searches through compiled code and assets for localized text. saving detected entries to .archive text files
- `loc export` : [Archive -> PO] Exports gathered archives to human-readable PO files. Updates the 'Game_Conflicts.txt' file
- `loc compile` : [Archive -> LocRes] Compiles localization archive into binary files for application bundling
- `loc import` : [PO -> Archive] Imports external PO files into the localization archive
- `loc update` : [Game -> Archive -> PO & LocRes] Gather, Export, and Compile all current localization
- `loc export-zip` : [Archive -> PO Zip] Exports localization archives and zips the PO fles
- `loc import-zip` : [PO Zip -> Archive] Extracts the contents of a PO zip and imports them into localization archive

Additional Subcommands:
- `init-cfg`: Save the dynamically generated config as a static config
- `cfg`: Apply changes to the current config and save it as a static config
- `fixup-binaries`: Dealiases the binaries for the project and its plugins (I forget what this is used for...)
