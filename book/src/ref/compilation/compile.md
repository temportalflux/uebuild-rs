# compile

Compiles the project (and the engine/editor if its built from source).

Usage: `uebuild compile [OPTIONS]`

Options:
* `-h/--help`: Print help information
* `-t/--target <TARGET>`
	* default: `editor`
	* possible values: `editor`, `client`, `server`
* `-p/--platform <PLATFORM>`
	* default: `windows`
	* possible values: `windows`, `ps4`, `switch`, `xbox-one`, `linux`
* `-c/--configuration <CONFIGURATION>`
	* default: `debug-game`
	* possible values: `debug-game`, `development`, `test`, `shipping`

This delegates down to the build batch file of the engine found out: `<Engine>/Build/BatchFiles/Build.bat`.

## Examples

Context: The target files are `Game.Target.cs`, `GameEditor.Target.cs`, and `GameServer.Target.cs`

### Default: Windows DebugGame Editor

Usage: `uebuild compile`

Becomes:

```sh
C:/Program Files/Epic Games/UE_5.0/Engine/Build/BatchFiles/Build.bat Game DebugGame Win64
```

### Linux Release Server

Usage: `uebuild compile --target server --platform linux --configuration shipping`

Becomes:

```sh
C:/Program Files/Epic Games/UE_5.0/Engine/Build/BatchFiles/Build.bat GameServer Shipping Linux
```

### PlayStation4 Test Client

Usage: `uebuild compile --target client --platform ps4 --configuration test`

Becomes:

```sh
C:/Program Files/Epic Games/UE_5.0/Engine/Build/BatchFiles/Build.bat Game Test PS4
```
