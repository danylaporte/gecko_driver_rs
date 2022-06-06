//! Manage the gecko driver (firefox).
//!
//! This module download the driver, defines arguments to launch the driver etc.

mod capabilities;
mod error;
mod gecko_process;

use bytes::Bytes;
pub use capabilities::{Args, Capabilities};
pub use error::{Error, Result};
pub use gecko_process::GeckoProcess;
use reqwest::{get, header::USER_AGENT, Client};
use serde::Deserialize;
use std::{fs::remove_dir_all, io::Cursor, path::Path, process::Command};

const USER_AGENT_VALUE: &str = "Mozilla/5.0 (X11; Linux i586; rv:31.0) Gecko/20100101 Firefox/72.0";

/// download the latest gecko driver from github to be able to run it directly
pub async fn download_latest() -> Result<()> {
    let releases = download_release_list().await?;
    let url = find_download_url(releases)?;
    let bytes = download_driver(&url).await?;

    let _ = remove_dir_all("drivers/gecko");
    archive_unpack(&bytes, "drivers/gecko")
}

async fn download_driver(url: &str) -> Result<Bytes> {
    Ok(get(url).await?.bytes().await?)
}

async fn download_release_list() -> Result<Releases> {
    const URL: &str = "http://api.github.com/repos/mozilla/geckodriver/releases/latest";

    let request = Client::builder()
        .build()?
        .get(URL)
        .header(USER_AGENT, USER_AGENT_VALUE);

    Ok(request.send().await?.json().await?)
}

/// find the gecko driver download url for the current platform / arch
fn find_download_url(releases: Releases) -> Result<String> {
    #[cfg(all(target_os = "linux", target_arch = "x86"))]
    const ASSET_NAME: &str = "linux32.tar.gz";

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    const ASSET_NAME: &str = "linux64.tar.gz";

    #[cfg(target_os = "macos")]
    const ASSET_NAME: &str = "macos.tar.gz";

    #[cfg(all(target_os = "windows", target_arch = "x86"))]
    const ASSET_NAME: &str = "win32.zip";

    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    const ASSET_NAME: &str = "win64.zip";

    let (_, url) = releases
        .assets
        .iter()
        .filter_map(|a| Some((a.name.as_ref()?, a.browser_download_url.as_ref()?)))
        .filter(|(name, _)| name.to_lowercase().ends_with(ASSET_NAME))
        .next()
        .ok_or(Error::ReleaseNotFound)?;

    Ok(url.clone())
}

#[cfg(target_os = "linux")]
fn archive_unpack<P: AsRef<Path>>(bytes: &[u8], dst: P) -> Result<()> {
    use flate2::read::GzDecoder;
    use tar::Archive;

    let tar = GzDecoder::new(Cursor::new(bytes));
    let mut archive = Archive::new(tar);
    Ok(archive.unpack(dst)?)
}

#[cfg(target_os = "windows")]
fn archive_unpack<P: AsRef<Path>>(bytes: &[u8], dst: P) -> Result<()> {
    use std::{
        fs::{create_dir_all, File},
        io::copy,
    };

    use zip::ZipArchive;

    let dst = dst.as_ref();
    let mut archive = ZipArchive::new(Cursor::new(bytes))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;

        let outpath = dst.join(file.mangled_name());

        if (&*file.name()).ends_with('/') {
            create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    create_dir_all(&p)?;
                }
            }

            let mut outfile = File::create(&outpath)?;

            copy(&mut file, &mut outfile)?;
        }

        // Get and Set permissions
        #[cfg(unix)]
        {
            use std::fs::{set_permissions, Permissions};
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                set_permissions(&outpath, std::Permissions::from_mode(mode))?;
            }
        }
    }

    Ok(())
}

/// run the geckodriver (firefox)
///
/// The default port should be 4444
pub fn run(port: u16) -> Result<GeckoProcess> {
    let child = Command::new("drivers/gecko/geckodriver")
        .arg(format!("-p{}", port))
        .spawn()?;

    Ok(GeckoProcess(child))
}

#[derive(Deserialize)]
struct Assets {
    name: Option<String>,
    browser_download_url: Option<String>,
}

#[derive(Deserialize)]
struct Releases {
    assets: Vec<Assets>,
}

#[cfg(test)]
#[tokio::test]
async fn download_run_works() {
    download_latest().await.expect("download");
    run(4444).expect("run");
}
