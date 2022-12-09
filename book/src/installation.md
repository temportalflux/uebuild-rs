# Installation

The only presently-available way to install `uebuild` is through the pre-compiled released binaries.

## Pre-compiled Binaries

Executable binaries are available for download on the [Github Releases page](https://github.com/temportalflux/uebuild-rs/releases). Download the binary for you platform (only Windows is supported) which youcan run to build your project.

To make it easier to run, put the path to the binary into your `PATH`.

## Build from source

If this project is published to [crates.io], it will be able to be installed via cargo.

To build the `uebuild` executable from source, you'll need to install Rust and Cargo. Follow the instructions on the [Rust installation page](https://www.rust-lang.org/tools/install). `uebuild` is currently built using Rust version `1.65`.

Once you have installed Rust, the following command can be used to build and install `uebuild`:

```sh
cargo install uebuild
```

This will automatically download `uebuild` from [crates.io], build it, and install it in Cargo's global binary directory (`~/.cargo/bin/` by default).

To uninstall, run the command `cargo uninstall uebuild`.

### Installing the latest main version

The version published to [crates.io] will ever so slightly be behind the version hosted on GitHub. If you need the latest version you can build the git version of `uebuild` yourself. Cargo makes this ***super easy***!

```sh
cargo install --git https://github.com/temportalflux/uebuild-rs.git uebuild
```

Again, make sure to add the Cargo bin directory to your `PATH`.
