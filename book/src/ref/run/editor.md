# editor

Opens the project in the Unreal Engine editor by running the binary executable for the editor (derived from `.uproject` file).

Usage: `uebuild editor`

Thats all there is to it! Basically this just delegates to the editor exe to result in a subprocess like:

```sh
C:/Program Files/Epic Games/UE_5.0/Engine/Binaries/Win64/UE4Editor-Win64-DebugGame.exe ./Game.uproject -stdout -AllowStdOutLogVerbosity -debug
```
