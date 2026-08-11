# Usage

```help
PowerToys Run Plugin Manager

Usage: ptr.exe [OPTIONS] <COMMAND>

Commands:
  init         Try to find and add existing plugins to config
  add          Add a plugin [alias: a]
  update       Update plugins [alias: u]
  remove       Remove plugins [alias: r]
  list         List all installed plugins [alias: l]
  pin          Pin plugins so it's not updated with `update --all` [alias: p]
  import       Import plugins from configuration file [alias: i]
  restart      Restart PowerToys
  edit         Open config file in default editor
  self-update  Self update to latest
  completion   Generate shell completion (PowerShell)
  help         Print this message or the help of the given subcommand(s)

Options:
      --no-restart  Do not restart PowerToys after plugin modification
  -h, --help        Print help
  -V, --version     Print version
```

## Init

```init --help
Try to find and add existing plugins to config

Usage: ptr.exe init

Options:
  -h, --help  Print help
```

## Add

```add --help
Add a plugin

Usage: ptr.exe add [OPTIONS] <NAME> <REPO>

Arguments:
  <NAME>  Plugin name, can be anything
  <REPO>  GitHub repository identifier or URL of the plugin

Options:
  -v, --version <VERSION>  Target version
  -p, --pattern <PATTERN>  Asset match pattern (rust regex)
  -h, --help               Print help
```

e.g.

```
ptr a GitHubRepo 8LWXpg/PowerToysRun-GitHubRepo
```

## Update

```update --help
Update plugins

Usage: ptr.exe update [OPTIONS] [NAME]...

Arguments:
  [NAME]...  Name of the plugins to update

Options:
  -a, --all                Update all plugins
  -v, --version <VERSION>  Version to update
  -h, --help               Print help
```

e.g.

```
ptr u -a
```

```
ptr u Plugin1 Plugin2 -v v1.1.0 -v 1.2.0
```

## Remove

```remove --help
Remove plugins

Usage: ptr.exe remove [NAME]...

Arguments:
  [NAME]...  Name of the plugins to remove

Options:
  -h, --help  Print help
```

e.g.

```
ptr r GitHubRepo ProcessKiller
```

## List

```list --help
List all installed plugins

Usage: ptr.exe list

Options:
  -h, --help  Print help
```

## Pin

```pin --help
Pin plugins so it's not updated with `update --all`

Usage: ptr.exe pin <COMMAND>

Commands:
  add     Add pins [alias: a]
  remove  Remove pins [alias: r]
  list    List pins [alias: l]
  reset   Clear all pins
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

## Import

This reads the configuration file at `%LOCALAPPDATA%\Microsoft\PowerToys\PowerToys Run\Plugins\version.toml`.

```import --help
Import plugins from configuration file

Usage: ptr.exe import [OPTIONS]

Options:
  -d, --dry-run  Update the configuration file without downloading the plugin
  -h, --help     Print help
```

## Restart

```restart --help
Restart PowerToys

Usage: ptr.exe restart

Options:
  -h, --help  Print help
```

## Edit

```edit --help
Open config file in default editor

Usage: ptr.exe edit [OPTIONS]

Options:
  -p, --path  Prints path instead
  -h, --help  Print help
```

## Self Update

```self-update --help
Self update to latest

Usage: ptr.exe self-update

Options:
  -h, --help  Print help
```

## Completion

```completion --help
Generate shell completion (PowerShell)

Usage: ptr.exe completion

Options:
  -h, --help  Print help
```

Add this line in your PowerShell `$PROFILE`:

```pwsh
(ptr completion) -join "`n" | iex
```
