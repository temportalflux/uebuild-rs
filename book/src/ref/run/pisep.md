# pisep

Run a local play-in-editor instance of the project in a separate editor process (Play In Separate Editor Process). Supports both clients and dedicated servers.

Usage: `uebuild pisep [OPTIONS]`

Options:
* `-h/--help`: Print help information
* `-c/--configuration <CONFIGURATION>`: The configuration that the project should be run in. The DebugGame editor binary is always used.
	* default: `debug-game`
	* possible values: `debug-game`, `development`, `test`, `shipping`
	* effect: Drives the value of the `RunConfig` property provided to the editor executable.
* `-s/--server`: If provided, pisep is run as a dedicated server (windowless).
	* effect: If provided, `-server` is passed to the editor executable, otherwise `-game` is provided.
* `--level <LEVEL>`: The unreal map level to open when the game begins. Defaults to the level setting in user preferences based on if this is a server or not.
	* possible values: Determined by the `MapsToCook` properties in the `Config/DefaultGame.ini` file of the project. The file path values are distilled to just the name of the map.
	* effect: the value provided is expanded back to its full asset path and passed to the binary executable as the level to load at start (it also always includes the `?listen` param).
* `--mode <MODE>`: The game mode alias to run in the level. Ignored if level is not provided.
	* possible values: Determined by the `GameModeClassAliases` properties in the `Config/DefaultEngine.ini` file of the project. The aliases are used as inputs, and if there are duplicates, the lowercase versions are preferred.
	* effect: appended the level path argument as the value of `?game=<value>`

This delegates down to the editor binary executable. The flags provided to this command configure what arguments are provided to the editor binary. The arguments `-stdout`, `-AllowStdOutLogVerbosity`, `-NoEAC`, `-messaging`, and `-debug` are always provided.

## Examples

### Default Client

Usage: `uebuild pisep`

Becomes:

```sh
C:/Program Files/Epic Games/UE_5.0/Engine/Binaries/Win64/UE4Editor-Win64-DebugGame.exe ./Game.uproject -game -stdout -AllowStdOutLogVerbosity -NoEAC -messaging RunConfig=DebugGame -debug
```

### Development Server

Usage: `uebuild pisep --configuration development --server --level ServerStart --mode Survival`

Becomes:

```sh
C:/Program Files/Epic Games/UE_5.0/Engine/Binaries/Win64/UE4Editor-Win64-DebugGame.exe ./Game.uproject -server /Game/Maps/ServerStart.ServerStart?game=Survival?listen -stdout -AllowStdOutLogVerbosity -NoEAC -messaging RunConfig=Development -debug
```
