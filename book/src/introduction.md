# Introduction

Unreal Engine Build (`uebuild`) is a Command Line tool for interfacing with [Unreal Engine](docs.unrealengine.com) ([GitHub](https://github.com/EpicGames/UnrealEngine)). It is written in Rust and provides methods for interfacing with a project such as; compiling from source, running the editor or standalone instance, cooking the project, and managing localization.

Unreal Engine Build is _not_ owned or maintained by Epic Games. This is a community creation and does not reflect or represent Epic Games or the official Unreal Engine project.

In this book you will find a guide for using the tool & what the specific commands are and how they are used or interface with a project. It is rendered using [mdbook](https://rust-lang.github.io/mdBook), and much of it is based on the tennets and examples set forth therein, so huge thanks and credit to contributors to that amazing project.

## Contributing

`uebuild` is free and open source. You can find the source code on [Github](https://github.com/temportalflux/uebuild-rs), and issues or feature requests can be posted on the [Github issue tracker](https://github.com/temportalflux/uebuild-rs/issues).

## License

The uebuild source and documentation are released under the [MIT](https://github.com/temportalflux/uebuild-rs/blob/main/LICENSE-MIT) and [Apache 2.0](https://github.com/temportalflux/uebuild-rs/blob/main/LICENSE-APACHE) licenses.

TODO: Generate code docs and mdbook on commit to public to gh-pages.
`cargo doc --no-deps --document-private-items --release` and move `./target/doc` to `<gh-pages>/doc`
Refer to doc as [docs](doc/uebuild).
Build mdbook via `mdbook build`.
https://github.com/rust-lang/mdBook/issues/1803
https://github.com/rust-lang/mdBook/wiki/Automated-Deployment%3A-GitHub-Pages
