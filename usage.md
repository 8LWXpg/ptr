# Usage

```help
PowerToys Run Plugin Manager

Usage: ptr.exe [OPTIONS] <COMMAND>

Commands:
  init         Try to find and add existing plugins to config
  add          Add a plugin [aliases: a]
  update       Update plugins [aliases: u]
  remove       Remove plugins [aliases: r]
  list         List all installed plugins [aliases: l]
  pin          Pin plugins so it's not updated with `update --all` [aliases: p]
  import       Import plugins from configuration file [aliases: i]
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

```
Usage: ptr.exe list
```

## Pin

```pin --help
Pin plugins so it's not updated with `update --all`

Usage: ptr.exe pin <COMMAND>

Commands:
  add     Add pins [aliases: a]
  remove  Remove pins [aliases: r]
  list    List pins [aliases: l]
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

```
Usage: ptr.exe restart
```

## Edit

```
Usage: ptr.exe edit
```

## Self Update

```
Usage: ptr.exe self-update
```

## Completion

```
Usage: ptr.exe completion
```

Add this line in your PowerShell `$PROFILE`:

```pwsh
(ptr completion) -join "`n" | iex
```