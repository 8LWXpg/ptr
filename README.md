# PowerToys Run Plugin Manager

![preview](https://github.com/user-attachments/assets/94489f6f-0301-4427-8c44-2f801201c64f)

This is a cli tool to manage PowerToys Run plugins. It can be used to install, uninstall, update, list, and import plugins.

## Installation

Download binary from [releases](https://github.com/8LWXpg/ptr/releases) page.

or build from source:

```
cargo install --git https://github.com/8LWXpg/ptr.git
```

### Features

Currently it has 2 variants:

- `sudo` - calls `sudo.exe` to elevate the process.
- `winapi` - uses Windows API to elevate the process.

## Limitations

- This tool only supports plugins hosted on GitHub.
- The plugin release must be a zip file with either `x64` or `arm64` in the file name.
- The zip structure must be like this:
  ```
  something-x64.zip
  └── pluginName
      └── plugin files...
  ```

For more general pattern matching and downloading, check another tool I wrote: [gpm](https://github.com/8LWXpg/gpm).


## Usage

This tool will create a file at `%APPDATA%\Local\Microsoft\PowerToys\PowerToys Run\Plugins\version.toml` to store installed plugins.

```
Usage: ptr.exe <COMMAND>

Commands:
  add     Add a plugin [aliases: a]
  update  Update plugins [aliases: u]
  remove  Remove plugins [aliases: r]
  list    List all installed plugins [aliases: l]
  import  Import plugins from configuration file [aliases: i]
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### Add

```
Usage: ptr.exe add <NAME> <REPO>

Arguments:
  <NAME>  The name of the plugin, same as the folder name in zip file.
  <REPO>  The GitHub repository of the plugin

Options:
  -v, --version <VERSION>  The target version of the plugin
  -h, --help               Print help
```

e.g.

```
ptr a GitHubRepo 8LWXpg/PowerToysRun-GitHubRepo
```

### Update

```
Usage: ptr.exe update [OPTIONS] [NAME]...

Arguments:
  [NAME]...  The name of the plugins to update

Options:
  -a, --all   Update all plugins
  -h, --help  Print help
```

### Remove

```
Usage: ptr.exe remove [NAME]...

Arguments:
  [NAME]...  The name of the plugins to remove

Options:
  -h, --help  Print help
```

### List

```
Usage: ptr.exe list
```

### Import

This reads the configuration file at `%APPDATA%\Local\Microsoft\PowerToys\PowerToys Run\Plugins\version.toml`.

```
Usage: ptr.exe import
```

## Why Rust?

The `clap` crate in Rust is very powerful and easy to use for building command line applications, so I chose Rust to build this tool.
