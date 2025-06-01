use anyhow::{anyhow, bail, Ok, Result};
use colored::Colorize;
use regex::Regex;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::Deserialize;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, mem};
use zip::ZipArchive;

use crate::config::Arch;
use crate::polling;
use crate::PLUGIN_PATH;
use crate::{error, exit};

#[derive(Deserialize)]
struct ApiResponse {
	tag_name: String,
	assets: Box<[Assets]>,
}

#[derive(Deserialize)]
struct Assets {
	name: String,
	browser_download_url: String,
}

impl Assets {
	/// Currently match for upper and lower case arch names.
	///
	fn is_arch(&self, arch: &Arch) -> bool {
		let arch = &arch.to_string();
		(self.name.contains(arch) || self.name.contains(&arch.to_uppercase()))
			&& self.name.ends_with(".zip")
	}
}

#[macro_export]
macro_rules! gh_dl {
	($root_name:expr, $repo:expr, $version:expr, $arch:expr, $pattern:expr, $token:expr) => {
		$crate::util::gh_dl($root_name, $repo, $version, $arch, None, $pattern, $token)
	};
	($root_name:expr, $repo:expr, $version:expr, $arch:expr, $current_version:expr, $pattern:expr, $token:expr) => {
		$crate::util::gh_dl(
			$root_name,
			$repo,
			$version,
			$arch,
			Some($current_version),
			$pattern,
			$token,
		)
	};
}

/// Download a file from a GitHub repository.
///
/// # Arguments
///
/// * `repo` - Repository identifier.
/// * `version` - Tagged version.
/// * `arch` - Architecture of the system, either x64 or arm64.
/// * `current_version` - Current tagged version.
/// * `pattern` - Match pattern for assets.
/// * `token` - GitHub auth token.
///
/// # Returns
/// The version of the repository that was downloaded.
pub fn gh_dl(
	root_name: &str,
	repo: &str,
	version: Option<&str>,
	arch: &Arch,
	current_version: Option<&str>,
	pattern: Option<&str>,
	token: Option<&str>,
) -> Result<String> {
	let url = if let Some(version) = version {
		format!("https://api.github.com/repos/{repo}/releases/tags/{version}")
	} else {
		format!("https://api.github.com/repos/{repo}/releases/latest")
	};
	let mut headers = HeaderMap::new();
	headers.insert(USER_AGENT, "reqwest".parse().unwrap());
	headers.insert(ACCEPT, "application/vnd.github+json".parse().unwrap());
	headers.insert("X-GitHub-Api-Version", "2022-11-28".parse().unwrap());
	if let Some(token) = token {
		headers.insert(AUTHORIZATION, format!("Bearer {token}").parse().unwrap());
	}
	let res = Client::new().get(&url).headers(headers).send()?;
	if !res.status().is_success() {
		bail!(
			"Failed to fetch {}: {}",
			version.unwrap_or("latest"),
			res.status().canonical_reason().unwrap_or("Unknown"),
		);
	}
	let res: ApiResponse = res.json()?;
	let tag = res.tag_name;
	if let Some(current_version) = current_version {
		if tag == current_version {
			return Ok(current_version.to_string());
		}
	}

	let assets = res.assets;
	let asset = match assets.iter().find(|a| {
		if let Some(pattern) = pattern {
			let p = Regex::new(pattern).unwrap_or_else(|e| {
				exit!(anyhow!(e).context(format!("Invalid regex pattern: '{}': ", pattern)))
			});
			p.is_match(&a.name)
		} else {
			a.is_arch(arch)
		}
	}) {
		Some(asset) => asset,
		None => manual_select(&assets)?,
	};
	let (url, name) = (&asset.browser_download_url, &asset.name);
	let res = Client::new().get(url).send()?;

	let file_path = PLUGIN_PATH.join(name);
	let mut file = File::create(&file_path)?;
	file.write_all(&res.bytes()?)?;

	extract_zip(&file_path, root_name)?;
	fs::remove_file(&file_path)?;

	Ok(tag)
}

fn manual_select(assets: &[Assets]) -> Result<&Assets> {
	if assets.len() == 1 {
		return Ok(&assets[0]);
	}

	for (i, asset) in assets.iter().enumerate() {
		println!("{}: {}", i.to_string().bright_yellow(), asset.name);
	}
	let index: usize = prompt("Fail to match assets, please select one: ")?.parse()?;
	assets.get(index).ok_or(anyhow!("Invalid index"))
}

fn extract_zip(zip_path: &Path, root_name: &str) -> Result<()> {
	let mut archive = ZipArchive::new(File::open(zip_path)?)?;
	env::set_current_dir(&*PLUGIN_PATH)?;

	// Locate for.dll file and find it's parent
	let dll = archive
		.file_names()
		.find(|f| f.ends_with(".dll"))
		.ok_or(anyhow!("No .dll file found in zip"))?
		.to_owned();
	let parent = Path::new(&dll).parent().unwrap_or(Path::new(""));

	// Extract all files and keep the directory structure
	let root = PathBuf::from(root_name);
	for i in 0..archive.len() {
		let mut file = archive.by_index(i)?;

		let out_path =
			if let std::result::Result::Ok(path) = Path::new(file.name()).strip_prefix(parent) {
				root.join(path)
			} else {
				error!("Unexpected file in zip at {}", file.name());
				continue;
			};

		if file.is_dir() {
			fs::create_dir_all(out_path)?;
		} else {
			if let Some(p) = out_path.parent() {
				fs::create_dir_all(p)?;
			}
			let mut out_file = File::create(out_path)?;
			polling::copy(&mut file, &mut out_file)?;
		}
	}

	Ok(())
}

fn run_process(program: &str, args: &str, admin: bool) -> Result<()> {
	use windows::core::{w, HSTRING, PCWSTR};
	use windows::Win32::Foundation::CloseHandle;
	use windows::Win32::System::Threading::{GetExitCodeProcess, WaitForSingleObject, INFINITE};
	use windows::Win32::UI::Shell::{ShellExecuteExW, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW};

	let mut sei: SHELLEXECUTEINFOW = unsafe { mem::zeroed() };
	sei.cbSize = mem::size_of::<SHELLEXECUTEINFOW>() as u32;
	sei.fMask = SEE_MASK_NOCLOSEPROCESS;
	if admin {
		sei.lpVerb = w!("runas");
	}
	let h_file = HSTRING::from(program);
	sei.lpFile = PCWSTR(h_file.as_ptr());
	sei.nShow = 0; // SW_HIDE
	let h_args = HSTRING::from(args);
	sei.lpParameters = PCWSTR(h_args.as_ptr());

	unsafe {
		ShellExecuteExW(&mut sei)?;
		let process = sei.hProcess;
		_ = WaitForSingleObject(process, INFINITE);
		let mut exit_code = 0;
		GetExitCodeProcess(process, &mut exit_code)?;
		CloseHandle(process)?;

		if exit_code == 0 {
			Ok(())
		} else {
			Err(io::Error::from_raw_os_error(exit_code as i32).into())
		}
	}
}

pub fn kill_ptr(admin: bool) -> Result<()> {
	run_process("taskkill.exe", "/F /FI \"IMAGENAME eq PowerToys*\"", admin)
		.map_err(|e| anyhow!("Failed to kill PowerToys: {}", e))?;
	Ok(())
}

pub fn start_ptr(powertoys_path: &Path) -> Result<()> {
	let c = Command::new(powertoys_path)
		.spawn()
		.map_err(|e| anyhow!("Failed to start PowerToys: {}", e))?;
	mem::forget(c);
	Ok(())
}

pub fn get_powertoys_path() -> Result<PathBuf> {
	let possible_paths = [
		PathBuf::from(env::var("ProgramFiles").unwrap_or_default()),
		PathBuf::from(env::var("LOCALAPPDATA").unwrap_or_default()),
	]
	.map(|p| p.join(r"PowerToys\PowerToys.exe"));
	for path in possible_paths {
		if path.exists() {
			return Ok(path);
		}
	}
	prompt("PowerToys executable not found in any of the expected locations\nEnter path: ")
		.map(|s| s.into())
}

/// Prompt the user for string input.
pub fn prompt(msg: &str) -> Result<String> {
	let mut input = String::new();
	print!("{msg}");
	io::stdout().flush()?;
	io::stdin().read_line(&mut input)?;
	Ok(input.trim().to_string())
}

pub fn self_update() -> Result<()> {
	use crate::{add, up_to_date};

	// download asset
	let current_version = env!("CARGO_PKG_VERSION");
	let url = "https://api.github.com/repos/8LWXpg/ptr/releases/latest";
	let mut headers = HeaderMap::new();
	headers.insert(USER_AGENT, "reqwest".parse().unwrap());
	headers.insert(ACCEPT, "application/vnd.github+json".parse().unwrap());
	headers.insert("X-GitHub-Api-Version", "2022-11-28".parse().unwrap());
	let res = Client::new().get(url).headers(headers).send()?;
	if !res.status().is_success() {
		bail!(
			"Failed to fetch latest: {}",
			res.status().canonical_reason().unwrap_or("Unknown"),
		);
	}
	let res: ApiResponse = res.json()?;
	let tag = res.tag_name;
	if tag == format!("v{current_version}") {
		up_to_date!("ptr", current_version);
		return Ok(());
	}

	let assets = res.assets;
	let asset = assets
		.iter()
		.find(|a| a.name.contains(std::env::consts::ARCH))
		.unwrap();
	let (url, name) = (&asset.browser_download_url, &asset.name);
	let res = Client::new().get(url).send()?;

	let file_path = env::temp_dir().join(name);
	File::create(&file_path)?.write_all(&res.bytes()?)?;

	// extract and self replace
	let mut archive = ZipArchive::new(File::open(&file_path)?)?;
	let out_path = env::temp_dir().join("ptr.exe");
	let mut out_file = File::create(&out_path)?;
	io::copy(&mut archive.by_name("ptr.exe")?, &mut out_file)?;
	self_replace::self_replace(&out_path)?;
	fs::remove_file(&file_path)?;
	fs::remove_file(&out_path)?;
	add!("ptr", tag);
	Ok(())
}

// region: macro
#[macro_export]
macro_rules! print_message {
    ($symbol:expr, $color:ident, $msg:expr) => {
        println!("{} {}", $symbol.$color().bold(), $msg)
    };
    ($symbol:expr, $color:ident, $fmt:expr, $($arg:tt)*) => {
        println!("{} {}", $symbol.$color().bold(), format!($fmt, $($arg)*))
    };
}

/// Print message as following format for adding an item.
///
/// `+ name@version`
#[macro_export]
macro_rules! add {
	($name:expr, $version:expr) => {
		$crate::print_message!("+", bright_green, "{}@{}", $name, $version)
	};
}

/// Print message as following format for item that is up to date.
///
/// `= name@version`
#[macro_export]
macro_rules! up_to_date {
	($name:expr, $version:expr) => {
		$crate::print_message!("=", bright_blue, "{}@{}", $name, $version)
	};
}

/// Print message as following format for removing an item.
///
/// `- name`
#[macro_export]
macro_rules! remove {
	($name:expr) => {
		$crate::print_message!("-", bright_red, $name)
	};
}

/// Print an error message to stderr.
#[macro_export]
macro_rules! error {
    ($msg:expr) => {{
        use colored::Colorize;
        eprintln!("{} {:#}", "error:".bright_red().bold(), $msg)
    }};
    ($fmt:expr, $($arg:tt)*) => {{
        use colored::Colorize;
        eprintln!("{} {}", "error:".bright_red().bold(), format!($fmt, $($arg)*))
    }};
}

/// Print an error message to stderr and exit with code 1.
#[macro_export]
macro_rules! exit {
    ($($arg:tt)*) => {{
        $crate::error!($($arg)*);
        std::process::exit(1);
    }};
}
// endregion

pub trait ResultExit<T> {
	/// Exit the program with error code 1 if Result is Err, otherwise return the Ok value
	fn exit_on_error(self) -> T;
}

impl<T> ResultExit<T> for Result<T> {
	fn exit_on_error(self) -> T {
		self.unwrap_or_else(|e| exit!(e))
	}
}
