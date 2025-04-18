# PowerToys Run Plugin Manager

![preview](https://github.com/user-attachments/assets/94489f6f-0301-4427-8c44-2f801201c64f)

Install and manage any PowerToys Run plugin released on GitHub with single command line interface.

## Installation

Download binary from [releases](https://github.com/8LWXpg/ptr/releases) page.

using `cargo-binstall`:

```
cargo binstall --git https://github.com/8LWXpg/ptr ptr
```

build from source:

```
cargo install --git https://github.com/8LWXpg/ptr.git
```

## Features

- Easy to use command line interface with informative help messages.
- Automatically download and install plugins from GitHub.
- Restart PowerToys after installing or removing plugins.
- Update all plugins with a single command.
- Restore plugins from a configuration file.

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

Check result with `list`:

```
ptr list
```

Use `help`, `-h` or `--help` to quickly check for usage:

```
ptr pin -h
ptr pin add -h
```

Use alias to type commands faster:

```
ptr u -a
```

## Limitations

If you have any suggestions for these limitations, feel free to open an issue.

- This tool only supports plugins hosted on GitHub.
- The plugin release must be a zip file with either `x64` or `arm64` in the file name, or a pattern from `--pattern` is required.

For more general pattern matching and downloading, check another tool I wrote: [gpm](https://github.com/8LWXpg/gpm).

## Config

The following config needs to modify manually:

```toml
admin = true    # Whether start and kill as admin
token = 'token' # Token used when making request to GitHub.
```

For the generated config structure, refer to [`test.toml`](./test/test.toml).

## Usage

Check [usage.md](./usage.md)

## Why Rust?

The `clap` crate in Rust is very powerful and easy to use for building command line applications, so I chose Rust to build this tool.
