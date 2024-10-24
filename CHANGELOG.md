# Change Log

## [0.6.0]

### Added

- Added `--version` flag in `update` command to specify the version of the plugin to update.

## [0.5.0]

### Added

- Added `pt_path` field in the configuration file to specify the path to PowerToys installation. **(Breaking Change)**
- Added `--dry-run (-d)` flag in `import` command to only update the configuration file without downloading the plugin, useful when config file spec is changed.

### Changed

- Support `ARM64` along with `arm64` in the archive name.
- Check for `.zip` file extension in the archive name.
- Only check plugins field on `import` command.

## [0.4.2]

### Fixed

- Fixed extracting plugin with different folder name.

## [0.4.1]

### Fixed

- Fixed extracting plugin with different folder name.

## [0.4.0]

### Added

- Support for renaming the downloaded plugin folder to the provided name.

### Changed

- Default to using `winapi` to elevate the process, as there's no major difference between `sudo` and `winapi`.

## [0.3.0]

### Added

- Now polling for file access after killing PowerToys, with interval of 50ms and max retries of 10.
- Create feature `winapi` that uses Windows API to elevate the process, the default is using `sudo`.

## [0.2.0]

### Added

- Support for killing and restarting PowerToys, this will pop 2 UAC prompts.

## [0.1.1]

### Changed

- Removed the progress bar, as most plugins are too small to display a meaningful download progress.
- Replaced asynchronous code with `reqwest::blocking` for simplicity.

## [0.1.0]

First release