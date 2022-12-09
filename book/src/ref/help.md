# help

Command: `uebuild [help/--help/-h]` or `uebuild <subcommand> --help/-h`

Displays information about the tool or a subcommand; what its purpose is, how to use it, what arguments to supply. Also displays a list of subcommands relevant to the provided command.

Example:
```text
Rust CLI for interfacing with Unreal Engine's command line api.

Usage: uebuild.exe [COMMAND]

Commands:
  init-cfg           Save the dynamically generated config to the current directory
  cfg                Handle changes to the user preferences/configuration for this project
  gen-project-files  Generate the project files (e.g. ".sln")
  compile            Compiles the code for the project
  cook               Cooks the project to run standalone
  editor             Opens the uproject in the unreal editor
  pisep              Run a local play-in-editor instance of the project in a separate editor process (Play In Separate Editor Process)
  loc                Subcommands to handle localization files
  help               Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help information
  -V, --version  Print version information
```
