# PowerToys Run Plugin Manager

![preview](https://github.com/user-attachments/assets/94489f6f-0301-4427-8c44-2f801201c64f)

Install and manage any PowerToys Run plugin released on GitHub with single command line interface.

## Installation

### Download

Download binary from [releases](https://github.com/8LWXpg/ptr/releases) page.

### Using `cargo-binstall`

```
cargo binstall --git https://github.com/8LWXpg/ptr ptr
```

### Build from source

```
cargo install --git https://github.com/8LWXpg/ptr.git
```

## Features

- Easy to use command line interface with informative help messages.
- Automatically download and install plugins from GitHub.
- Restart PowerToys after installing or removing plugins.
- Update all plugins with a single command.
- Restore plugins from configuration file.

## Quick Start

### For New Plugins

Install a plugin with `add`:

```
ptr add GitHubRepo 8LWXpg/PowerToysRun-GitHubRepo
```

### For Existing Plugins

Add existing plugins with `init`:

```
ptr init
```

> [!NOTE]
> This overrides existing config

Then update with

```
ptr update --all
```

### Useful tips

A config file will be created at `%LOCALAPPDATA%\Microsoft\PowerToys\PowerToys Run\Plugins\version.toml`. Check [config](#config) for more detail.

#### Open config file in default editor

```
ptr edit
```

#### Check installed plugins with `list`

```
ptr list
```

#### Use `help`, `-h` or `--help` to quickly check for usage

```
ptr pin -h
ptr pin add -h
```

#### Use alias to type commands faster

```
ptr u -a
```

#### Self update to latest

```
ptr self-update
```

## Config

The following config needs to modify manually at `%LOCALAPPDATA%\Microsoft\PowerToys\PowerToys Run\Plugins\version.toml`:

```toml
admin = true    # Whether start and kill as admin
token = 'token' # Token used when making request to GitHub.
no_restart = false  # Set true to not restart PowerToys after plugin modification
```

For the generated config structure, refer to struct `Config` in [`config.rs`](./src/config.rs).

## Usage

Check [usage.md](./usage.md)

## Limitations

If you have any suggestions for these limitations, feel free to open an issue.

- This tool only supports plugins hosted on GitHub.
- The plugin release must be a zip file with either `x64` or `arm64` in the filename, or a pattern from `--pattern` is required.

For more general pattern matching and downloading, check another tool I wrote: [gpm](https://github.com/8LWXpg/gpm).

## Why Rust?

The `clap` crate in Rust is very powerful and easy to use for building command line applications, so I chose Rust to build this tool.
