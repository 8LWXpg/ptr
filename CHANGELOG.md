# Change Log

## [0.4.0]

### Added

- Support for renaming the downloaded plugin folder to the provided name.

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