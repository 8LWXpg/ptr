use anyhow::{bail, Result};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::header::{HeaderMap, ACCEPT, USER_AGENT};
use reqwest::Client;
use serde::Deserialize;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;
use zip::ZipArchive;

use crate::PLUGIN_PATH;

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
#[macro_export]
macro_rules! gh_dl {
    ($repo:expr, $version:expr, $arch:expr) => {
        $crate::util::gh_dl($repo, $version, $arch, None)
    };
    ($repo:expr, $version:expr, $arch:expr, $current_version:expr) => {
        $crate::util::gh_dl($repo, $version, $arch, Some($current_version))
    };
}

/// Download a file from a GitHub repository.
///
/// # Arguments
///
/// * `repo` - The repository to download from.
/// * `version` - The tagged version of the repository to download.
///
/// # Returns
/// The version of the repository that was downloaded.
pub async fn gh_dl(
    repo: &str,
    version: Option<String>,
    arch: &str,
    current_version: Option<String>,
) -> Result<String> {
    let url = if let Some(version) = version {
        format!(
            "https://api.github.com/repos/{}/releases/tags/{}",
            repo, version
        )
    } else {
        format!("https://api.github.com/repos/{}/releases/latest", repo)
    };
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, "reqwest".parse().unwrap());
    headers.insert(ACCEPT, "application/vnd.github+json".parse().unwrap());
    headers.insert("X-GitHub-Api-Version", "2022-11-28".parse().unwrap());
    let res = Client::new().get(&url).headers(headers).send().await?;
    if !res.status().is_success() {
        bail!(
            "Failed to fetch the latest release: {}",
            res.status().canonical_reason().unwrap_or("Unknown"),
        );
    }
    let res = res.json::<ApiResponse>().await?;
    let tag = res.tag_name;
    if let Some(current_version) = current_version {
        if tag == current_version {
            return Ok(current_version);
        }
    }

    let asset = &res
        .assets
        .iter()
        .find(|a| a.name.contains(arch))
        .expect("No asset found for the current architecture");
    let (url, name) = (&asset.browser_download_url, &asset.name);
    let res = Client::new().get(url).send().await?;
    let total_size = res.content_length().unwrap_or(0);

    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg:.cyan} [{bar}] {bytes}/{total_bytes} ({eta})")?
            .progress_chars("=> "),
    );
    pb.set_message("Downloading");
    let file_path = PLUGIN_PATH.join(name);
    let mut file = File::create(&file_path)?;
    let mut stream = res.bytes_stream();
    while let Some(item) = stream.next().await {
        let chunk = item?;
        file.write_all(&chunk)?;
        pb.inc(chunk.len() as u64);
    }
    pb.finish_and_clear();

    extract_zip(&file_path, &PLUGIN_PATH)?;
    fs::remove_file(&file_path)?;

    Ok(tag)
}

fn extract_zip(zip_path: &Path, output_dir: &Path) -> Result<()> {
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let out_path = Path::new(output_dir).join(file.name());

        if (file.name()).ends_with('/') {
            fs::create_dir_all(&out_path)?;
        } else {
            if let Some(p) = out_path.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?;
                }
            }
            let mut out_file = File::create(&out_path)?;
            io::copy(&mut file, &mut out_file)?;
        }
    }

    Ok(())
}

#[macro_export]
macro_rules! tabwriter {
    ($fmt:expr, $($arg:tt)*) => {{
        use std::io::Write;
        use colored::Colorize;
        let mut tw = tabwriter::TabWriter::new(vec![]);
        write!(&mut tw, $fmt, $($arg)*).expect("Failed to write to TabWriter");
        tw.flush().expect("Failed to flush TabWriter");
        println!("{}", String::from_utf8(tw.into_inner().unwrap()).unwrap());
    }};
}

#[macro_export]
macro_rules! print_message {
    ($symbol:expr, $color:ident, $msg:expr) => {
        $crate::tabwriter!("{} {}", $symbol.$color().bold(), $msg)
    };
    ($symbol:expr, $color:ident, $fmt:expr, $($arg:tt)*) => {
        $crate::tabwriter!("{} {}", $symbol.$color().bold(), format!($fmt, $($arg)*))
    };
}

/// print message for adding an item.
#[macro_export]
macro_rules! add {
    ($($arg:tt)*) => {
        $crate::print_message!("+", bright_green, $($arg)*)
    };
}

/// print message for cloning an item.
#[macro_export]
macro_rules! clone {
    ($($arg:tt)*) => {
        $crate::print_message!("=", bright_blue, $($arg)*)
    };
}

/// print message for removing an item.
#[macro_export]
macro_rules! remove {
    ($($arg:tt)*) => {
        $crate::print_message!("-", bright_red, $($arg)*)
    };
}

/// Print an error message to stderr.
#[macro_export]
macro_rules! error {
    ($msg:expr) => {{
        use colored::Colorize;
        eprintln!("{} {}", "error:".bright_red().bold(), $msg)
    }};
    ($fmt:expr, $($arg:tt)*) => {{
        use colored::Colorize;
        eprintln!("{} {}", "error:".bright_red().bold(), format!($fmt, $($arg)*))
    }};
}

/// prompt the user for a yes/no response.
///
/// # Arguments
/// `message` - The prompt to display, appended with " [y/N]: "
pub fn prompt(message: &str) -> Result<bool> {
    let mut input = String::new();
    print!("{} [y/N]: ", message);
    io::stdout().flush()?; // Make sure the prompt is immediately displayed
    io::stdin().read_line(&mut input)?;
    match input.trim().to_lowercase().as_str() {
        "y" => Ok(true),
        "n" => Ok(false),
        _ => Ok(false),
    }
}
