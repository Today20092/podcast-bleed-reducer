//! Manually requested, authenticated portable updates. Startup never uses the network.
mod package;
mod transaction;
use semver::Version;
use serde::Deserialize;
use std::{
    io::{Read, Write},
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
pub use transaction::stage as download;
pub use transaction::{
    InstallationLock, Prepared, cleanup, cli, eligible, health_acknowledge, health_mode, launch,
    launch_recovery, pending, previous, recover,
};

pub const REPOSITORY: &str = "Today20092/podcast-bleed-reducer";
pub const RELEASES: &str = "https://github.com/Today20092/podcast-bleed-reducer/releases";
pub fn manual_asset(version: &str) -> Option<String> {
    let platform = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("windows", "x86_64") => "windows-x64.zip",
        ("macos", "aarch64") => "macos-arm64.dmg",
        ("macos", "x86_64") => "macos-x64.dmg",
        ("linux", "x86_64") => "linux-x64.AppImage",
        _ => return None,
    };
    Some(format!("CleanSlate-{version}-{platform}"))
}
// No unsigned metadata or environment variable may replace this bootstrap trust root.
#[cfg(not(all(feature = "updater-test-key", debug_assertions)))]
pub const PUBLIC_KEY: &str = include_str!("../assets/update-public-key.txt");
#[cfg(all(feature = "updater-test-key", debug_assertions))]
pub const PUBLIC_KEY: &str = include_str!("../tests/fixtures/updater-test-public-key.txt");
#[cfg(all(feature = "updater-test-key", not(debug_assertions)))]
compile_error!("updater-test-key must never be used in distributed release builds");
pub type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Clone, Deserialize)]
pub struct Asset {
    pub name: String,
    pub size: u64,
}
#[derive(Debug, Clone, Deserialize)]
struct Release {
    tag_name: String,
    draft: bool,
    prerelease: bool,
    #[serde(default)]
    body: Option<String>,
    assets: Vec<Asset>,
}
#[derive(Debug, Clone)]
pub struct Offer {
    pub version: String,
    pub notes: String,
    pub tag: String,
    pub assets: Vec<Asset>,
}
impl Offer {
    pub fn asset(&self, name: &str) -> Result<&Asset> {
        let matches: Vec<_> = self.assets.iter().filter(|a| a.name == name).collect();
        if matches.len() != 1 {
            return Err(format!(
                "Expected exactly one release asset: {name}. Use the release page for manual download."
            ));
        }
        Ok(matches[0])
    }
    pub fn page(&self) -> String {
        format!("{RELEASES}/tag/{}", self.tag)
    }
}
pub fn select_release(data: &[u8], current: &str, prereleases: bool) -> Result<Option<Offer>> {
    let releases: Vec<Release> =
        serde_json::from_slice(data).map_err(|e| format!("Invalid release metadata: {e}"))?;
    let current = Version::parse(current).map_err(|e| e.to_string())?;
    let mut selected: Option<(Version, Offer)> = None;
    for release in releases {
        if release.draft {
            continue;
        }
        let text = release
            .tag_name
            .strip_prefix('v')
            .unwrap_or(&release.tag_name);
        let version = Version::parse(text)
            .map_err(|_| format!("Malformed release version: {}", release.tag_name))?;
        if version.to_string() != text || release.prerelease != !version.pre.is_empty() {
            return Err("Release channel/version mismatch".into());
        }
        if (!prereleases && release.prerelease) || !version.cmp_precedence(&current).is_gt() {
            continue;
        }
        if selected
            .as_ref()
            .is_none_or(|(v, _)| version.cmp_precedence(v).is_gt())
        {
            selected = Some((
                version.clone(),
                Offer {
                    version: version.to_string(),
                    notes: release
                        .body
                        .unwrap_or_default()
                        .chars()
                        .take(20_000)
                        .collect(),
                    tag: release.tag_name,
                    assets: release.assets,
                },
            ));
        }
    }
    Ok(selected.map(|(_, offer)| offer))
}
pub fn cancelled(cancel: &AtomicBool) -> Result<()> {
    if cancel.load(Ordering::Relaxed) {
        Err("Update cancelled; the installed application was retained.".into())
    } else {
        Ok(())
    }
}

/// URLs are constructed locally. JSON never supplies request destinations.
fn download_url(tag: &str, name: &str) -> Result<String> {
    if tag
        .bytes()
        .chain(name.bytes())
        .any(|c| !c.is_ascii_alphanumeric() && !b".-_+".contains(&c))
    {
        return Err("Invalid release asset identity".into());
    }
    Ok(format!("{RELEASES}/download/{tag}/{name}"))
}
fn delivery_redirect(url: &str) -> bool {
    url.starts_with("https://release-assets.githubusercontent.com/")
        && !url.chars().any(|c| c.is_control() || c == '\\')
}
fn transfer(
    url: &str,
    output: &mut impl Write,
    max: u64,
    cancel: &AtomicBool,
    progress: &mut impl FnMut(u64),
) -> Result<u64> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .https_only(true)
        .max_redirects(0)
        .max_redirects_will_error(false)
        .http_status_as_error(false)
        .timeout_global(Some(Duration::from_secs(600)))
        .timeout_connect(Some(Duration::from_secs(15)))
        .timeout_recv_body(Some(Duration::from_secs(180)))
        .user_agent("CleanSlate-updater/1")
        .build()
        .into();
    let mut destination = url.to_string();
    for hop in 0..=3 {
        cancelled(cancel)?;
        let mut response = agent
            .get(&destination)
            .header("Accept-Encoding", "identity")
            .call()
            .map_err(|e| {
                format!(
                    "Network request failed: {e}. Check your connection or use manual download."
                )
            })?;
        let status = response.status().as_u16();
        if status == 403 || status == 429 {
            let retry = response
                .headers()
                .get("retry-after")
                .or_else(|| response.headers().get("x-ratelimit-reset"))
                .and_then(|h| h.to_str().ok())
                .unwrap_or("later");
            return Err(format!(
                "GitHub limited this request. Retry after {retry}; no automatic retry was attempted."
            ));
        }
        if matches!(status, 301 | 302 | 303 | 307 | 308) {
            let next = response
                .headers()
                .get("location")
                .and_then(|h| h.to_str().ok())
                .ok_or("Missing download redirect")?;
            if url.starts_with("https://api.github.com/") || !delivery_redirect(next) || hop == 3 {
                return Err("Unapproved release delivery redirect. Use manual download.".into());
            }
            destination = next.to_string();
            continue;
        }
        if status != 200 {
            return Err(format!(
                "GitHub returned HTTP {status}; use manual download or retry later."
            ));
        }
        if response
            .headers()
            .get("content-length")
            .and_then(|s| s.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .is_some_and(|n| n > max)
        {
            return Err("Download exceeds its size budget".into());
        }
        let mut reader = response.body_mut().as_reader();
        let mut buffer = [0u8; 64 * 1024];
        let mut total = 0u64;
        loop {
            cancelled(cancel)?;
            let n = reader
                .read(&mut buffer)
                .map_err(|e| format!("Download interrupted: {e}"))?;
            if n == 0 {
                return Ok(total);
            }
            total = total
                .checked_add(n as u64)
                .ok_or("Download size overflow")?;
            if total > max {
                return Err("Download exceeds its size budget".into());
            }
            output.write_all(&buffer[..n]).map_err(|e| {
                format!("Cannot stage update: {e}. Check free space and folder permissions.")
            })?;
            progress(total);
        }
    }
    Err("Too many redirects".into())
}
pub fn check(current: &str, prereleases: bool, cancel: &AtomicBool) -> Result<Option<Offer>> {
    let mut selected: Option<Offer> = None;
    for page in 1..=5 {
        let mut bytes = Vec::new();
        transfer(
            &format!("https://api.github.com/repos/{REPOSITORY}/releases?per_page=100&page={page}"),
            &mut bytes,
            4 * 1024 * 1024,
            cancel,
            &mut |_| {},
        )?;
        let count = serde_json::from_slice::<Vec<Release>>(&bytes)
            .map_err(|e| e.to_string())?
            .len();
        let floor = selected.as_ref().map_or(current, |o| &o.version);
        if let Some(offer) = select_release(&bytes, floor, prereleases)? {
            selected = Some(offer);
        }
        if count < 100 {
            return Ok(selected);
        }
    }
    Err(
        "Release history exceeded the bounded check; results are incomplete. Use the release page."
            .into(),
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn malformed_channels_drafts_missing_architecture_and_redirects_fail_closed() {
        assert!(
            super::select_release(
                br#"[{"tag_name":"v1.1.0","draft":false,"prerelease":true,"assets":[]}]"#,
                "1.0.0",
                true
            )
            .is_err()
        );
        assert!(
            super::select_release(
                br#"[{"tag_name":"v01.0.0","draft":false,"prerelease":false,"assets":[]}]"#,
                "1.0.0",
                false
            )
            .is_err()
        );
        assert!(
            super::select_release(
                br#"[{"tag_name":"v99.0.0","draft":true,"prerelease":false,"assets":[]}]"#,
                "1.0.0",
                false
            )
            .unwrap()
            .is_none()
        );
        let offer=super::select_release(br#"[{"tag_name":"v1.1.0","draft":false,"prerelease":false,"assets":[{"name":"CleanSlate-1.1.0-macos-arm64.dmg","size":123}]}]"#,"1.0.0",false).unwrap().unwrap();
        assert!(offer.asset("CleanSlate-1.1.0-windows-x64.zip").is_err());
        for url in [
            "http://release-assets.githubusercontent.com/a",
            "https://release-assets.githubusercontent.com.evil/a",
            "https://user@release-assets.githubusercontent.com/a",
            "https://release-assets.githubusercontent.com:444/a",
            "https://github.com/other/release",
        ] {
            assert!(!super::delivery_redirect(url));
        }
        assert!(super::delivery_redirect(
            "https://release-assets.githubusercontent.com/a?token=123"
        ));
        assert!(super::download_url("v1.1.0", "../../evil").is_err());
        assert!(super::cancelled(&std::sync::atomic::AtomicBool::new(true)).is_err());
    }
    #[test]
    fn release_selection_uses_semantic_precedence_and_stable_default() {
        let data = br#"[
          {"tag_name":"v1.10.0","draft":false,"prerelease":false,"body":"Ten","assets":[]},
          {"tag_name":"v2.0.0-beta.1","draft":false,"prerelease":true,"body":"Beta","assets":[]},
          {"tag_name":"v1.9.0","draft":false,"prerelease":false,"body":"Nine","assets":[]}
        ]"#;
        assert_eq!(
            super::select_release(data, "0.1.0-alpha.8", false)
                .unwrap()
                .unwrap()
                .version,
            "1.10.0"
        );
        assert!(
            super::select_release(data, "1.10.0+local", false)
                .unwrap()
                .is_none()
        );
        assert_eq!(
            super::select_release(data, "1.0.0", true)
                .unwrap()
                .unwrap()
                .version,
            "2.0.0-beta.1"
        );
    }
}
