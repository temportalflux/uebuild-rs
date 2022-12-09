# cook

Cooks the project to run as a standalone application.

Usage: `uebuild cook [OPTIONS]`

Options:
* `-h/--help`: Print help information
* `-t/--target <TARGET>`
	* default: `client`
	* possible values: `client`, `server`
* `-p/--platform <PLATFORM>`
	* default: `windows`
	* possible values: `windows`, `ps4`, `switch`, `xbox-one`, `linux`
* `-c/--configuration <CONFIGURATION>`
	* default: `development`
	* possible values: `debug-game`, `development`, `test`, `shipping`
* `-d/--dest <PATH>`: Relative path in the project root to output the cooked build to
	* default: "DeploymentBuilds"

This delegates down to the engine's `RunUAT` batch file (`<Engine>/Build/BatchFiles/RunUAT.bat`).

All variants end up with a subprocess call which looks like:

```sh
C:/Program Files/Epic Games/UE_5.0/Engine/Build/BatchFiles/RunUAT.bat -ScriptsForProject="./Game.uproject" BuildCookRun -project="./Game.uproject" -target=<target> -installed -nop4 -build -cook -stage -archive -archivedirectory="<dest>" -ddc=InstalledDerivedDataBackendGraph -pak -prereqs -nodebuginfo -utf8output
```

And depending on the target, there are additional parameters for:
* client: `-targetplatform=<platform> -clientconfig=<configuration>`
* server: `-server -noclient -serverplatform=<platform> -platform=<platform> -serverconfig=<configuration> -Target="<target> <platform> <configuration>"`

The `editor` target will throw an error if you try to cook it.

## Examples

Context: The target files are `Game.Target.cs`, `GameEditor.Target.cs`, and `GameServer.Target.cs`

### Default Client

Usage: `uebuild cook`

Becomes:

```sh
C:/Program Files/Epic Games/UE_5.0/Engine/Build/BatchFiles/RunUAT.bat -ScriptsForProject="./Game.uproject" BuildCookRun -project="./Game.uproject" -target=Game -installed -nop4 -build -cook -stage -archive -archivedirectory="DeploymentBuilds" -ddc=InstalledDerivedDataBackendGraph -pak -prereqs -nodebuginfo -utf8output -targetplatform=Win64 -clientconfig=Development
```

### Linux Test Server

Usage: `uebuild cook --target server --platform linux --configuration test -d ServerBuilds`

Becomes:

```sh
C:/Program Files/Epic Games/UE_5.0/Engine/Build/BatchFiles/RunUAT.bat -ScriptsForProject="./Game.uproject" BuildCookRun -project="./Game.uproject" -target=GameServer -installed -nop4 -build -cook -stage -archive -archivedirectory="ServerBuilds" -ddc=InstalledDerivedDataBackendGraph -pak -prereqs -nodebuginfo -utf8output -server -noclient -serverplatform=Linux -platform=Linux -serverconfig=Test -Target="GameServer Linux Test"
```
