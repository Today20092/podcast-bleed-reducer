//! Signed package validation.
use super::{Result, cancelled};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
};

pub(super) fn safe_path(name: &str) -> Result<PathBuf> {
    if name.is_empty() || name.len() > 240 || !name.is_ascii() {
        return Err("Invalid package path length/encoding".into());
    }
    for part in name.split('/') {
        let stem = part.split('.').next().unwrap_or("").to_ascii_uppercase();
        if part.is_empty()
            || part == "."
            || part == ".."
            || part.ends_with(['.', ' '])
            || !part
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b"._-+".contains(&c))
            || matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL" | "CLOCK$")
            || (stem.len() == 4
                && (stem.starts_with("COM") || stem.starts_with("LPT"))
                && stem.as_bytes()[3].is_ascii_digit())
        {
            return Err(format!("Unsafe package path: {name}"));
        }
    }
    Ok(PathBuf::from(name))
}

pub(super) fn hash_file(path: &Path) -> Result<String> {
    let mut input = File::open(path).map_err(|e| e.to_string())?;
    let mut hash = Sha256::new();
    let mut buffer = [0; 64 * 1024];
    loop {
        let n = input.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
    }
    Ok(format!("{:x}", hash.finalize()))
}
pub(super) fn hash_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub(super) fn regular(path: &Path) -> Result<()> {
    let meta = fs::symlink_metadata(path)
        .map_err(|e| format!("Cannot inspect {}: {e}", path.display()))?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if meta.file_attributes() & 0x400 != 0 {
            return Err("Reparse points are not updateable".into());
        }
    }
    if meta.is_symlink() {
        return Err("Links are not updateable".into());
    }
    Ok(())
}
pub(super) fn ancestors(path: &Path) -> Result<()> {
    for ancestor in path.ancestors() {
        regular(ancestor)?;
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Descriptor {
    pub schema: u32,
    pub product: String,
    pub repository: String,
    pub version: String,
    pub tag: String,
    pub platform: String,
    pub archive: String,
    pub sha256: String,
    pub bytes: u64,
    pub expanded_bytes: u64,
    pub entries: u64,
    pub manifest_sha256: String,
}
impl Descriptor {
    pub fn verify(bytes: &[u8], signature: &str, public_key: &str, version: &str) -> Result<Self> {
        if bytes.len() > 64 * 1024 || signature.len() > 4096 || public_key.trim().is_empty() {
            return Err("No trusted update signature/bootstrap key. Use manual download.".into());
        }
        let key = minisign_verify::PublicKey::from_base64(public_key.trim())
            .map_err(|e| e.to_string())?;
        let signature = minisign_verify::Signature::decode(signature).map_err(|e| e.to_string())?;
        key.verify(bytes, &signature, false)
            .map_err(|e| format!("Untrusted update metadata: {e}"))?;
        let d: Self =
            serde_json::from_slice(bytes).map_err(|e| format!("Invalid signed descriptor: {e}"))?;
        if d.schema != 1
            || d.product != "CleanSlate"
            || d.repository != super::REPOSITORY
            || d.version != version
            || d.tag != format!("v{version}")
            || d.platform != "windows-x64"
            || d.archive != format!("CleanSlate-{version}-windows-x64.zip")
            || d.bytes == 0
            || d.bytes > 1024 * 1024 * 1024
            || d.expanded_bytes == 0
            || d.expanded_bytes > 3 * 1024 * 1024 * 1024
            || d.entries == 0
            || d.entries > 20_000
            || !valid_hash(&d.sha256)
            || !valid_hash(&d.manifest_sha256)
        {
            return Err("Signed update identity or resource budget mismatch".into());
        }
        Ok(d)
    }
}
fn valid_hash(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PackageManifest {
    pub schema: u32,
    pub product: String,
    pub package_kind: String,
    pub entrypoint: String,
    pub version: String,
    pub platform: String,
    pub commit: String,
    pub dirty: bool,
    pub rustc: String,
    pub cargo_lock_sha256: String,
    pub files: Vec<OwnedFile>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct OwnedFile {
    pub path: String,
    pub sha256: String,
    pub bytes: u64,
}

pub(super) fn validate_tree(root: &Path, expected_hash: Option<&str>) -> Result<PackageManifest> {
    ancestors(root)?;
    let path = root.join("package-manifest.json");
    regular(&path)?;
    if fs::metadata(&path).map_err(|e| e.to_string())?.len() > 4 * 1024 * 1024 {
        return Err("Oversized package manifest".into());
    }
    let bytes = fs::read(&path).map_err(|e| e.to_string())?;
    if expected_hash.is_some_and(|h| h != hash_bytes(&bytes)) {
        return Err("Package manifest authentication failed".into());
    }
    let m: PackageManifest = serde_json::from_slice(
        bytes.strip_prefix(&[239, 187, 191]).unwrap_or(&bytes),
    )
    .map_err(|e| format!("Unsupported/invalid package manifest (manual update required): {e}"))?;
    if m.schema != 1
        || m.product != "CleanSlate"
        || m.platform != "windows-x64"
        || m.entrypoint != "CleanSlate.exe"
        || m.package_kind != "portable-zip"
        || m.files.is_empty()
        || m.files.len() > 20_000
        || semver::Version::parse(&m.version).is_err()
    {
        return Err("Unsupported package identity".into());
    }
    let mut expected = BTreeSet::from(["package-manifest.json".to_string()]);
    let mut dirs = BTreeSet::new();
    for file in &m.files {
        let relative = safe_path(&file.path)?;
        if !expected.insert(file.path.to_ascii_lowercase())
            || !valid_hash(&file.sha256)
            || file.bytes > 512 * 1024 * 1024
        {
            return Err("Invalid or duplicate owned file".into());
        }
        for parent in relative
            .ancestors()
            .skip(1)
            .filter(|p| !p.as_os_str().is_empty())
        {
            dirs.insert(
                parent
                    .to_string_lossy()
                    .replace('\\', "/")
                    .to_ascii_lowercase(),
            );
        }
        let full = root.join(relative);
        ancestors(&full)?;
        if !full.is_file()
            || fs::metadata(&full).map_err(|e| e.to_string())?.len() != file.bytes
            || hash_file(&full)? != file.sha256
        {
            return Err(format!(
                "Modified or missing package file: {}. Use manual download.",
                file.path
            ));
        }
    }
    for required in [
        "CleanSlate.exe",
        "concrt140.dll",
        "msvcp140.dll",
        "msvcp140_1.dll",
        "msvcp140_2.dll",
        "msvcp140_atomic_wait.dll",
        "msvcp140_codecvt_ids.dll",
        "vcruntime140.dll",
        "vcruntime140_1.dll",
        "LICENSE",
        "README.txt",
        "THIRD-PARTY-NOTICES.txt",
    ] {
        if !expected.contains(&required.to_ascii_lowercase()) {
            return Err(format!("Incomplete portable package: {required}"));
        }
    }
    if !expected.iter().any(|p| p.starts_with("licenses/")) {
        return Err("Missing package license texts".into());
    }
    fn walk(
        root: &Path,
        dir: &Path,
        files: &mut BTreeSet<String>,
        dirs: &BTreeSet<String>,
    ) -> Result<()> {
        for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            regular(&path)?;
            let name = path
                .strip_prefix(root)
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .replace('\\', "/");
            safe_path(&name)?;
            let name = name.to_ascii_lowercase();
            if path.is_dir() {
                if !dirs.contains(&name) {
                    return Err("Unrelated directories found; use manual download and leave this folder intact.".into());
                }
                walk(root, &path, files, dirs)?;
            } else if !path.is_file() || !files.insert(name) {
                return Err("Invalid/duplicate filesystem entry".into());
            }
        }
        Ok(())
    }
    let mut actual = BTreeSet::new();
    walk(root, root, &mut actual, &dirs)?;
    if actual != expected {
        return Err("Mixed portable folder: unrelated files found. Extract the new package into a new folder; leave this folder intact.".into());
    }
    Ok(m)
}

pub(super) fn extract(
    archive: &Path,
    destination: &Path,
    descriptor: &Descriptor,
    cancel: &AtomicBool,
) -> Result<PathBuf> {
    if fs::metadata(archive).map_err(|e| e.to_string())?.len() != descriptor.bytes
        || hash_file(archive)? != descriptor.sha256
    {
        return Err("Corrupt or truncated update archive".into());
    }
    // These packages never need ZIP64 or multi-disk archives. Bound central-directory
    // allocation before handing metadata to the ZIP parser.
    use std::io::{Seek, SeekFrom};
    let mut input = File::open(archive).map_err(|e| e.to_string())?;
    let tail_len = descriptor.bytes.min(65_557);
    input
        .seek(SeekFrom::End(-(tail_len as i64)))
        .map_err(|e| e.to_string())?;
    let mut tail = Vec::new();
    input.read_to_end(&mut tail).map_err(|e| e.to_string())?;
    let offset = (0..tail.len().saturating_sub(21))
        .rev()
        .find(|&i| {
            tail[i..].starts_with(b"PK\x05\x06")
                && i + 22 + u16::from_le_bytes([tail[i + 20], tail[i + 21]]) as usize == tail.len()
        })
        .ok_or("Missing/truncated ZIP directory")?;
    let e = &tail[offset..];
    let count = u16::from_le_bytes([e[10], e[11]]) as u64;
    if e[4..8] != [0, 0, 0, 0]
        || e[8..10] != e[10..12]
        || count == 65535
        || count > descriptor.entries
        || u32::from_le_bytes(e[12..16].try_into().unwrap()) > 16 * 1024 * 1024
    {
        return Err("Unsupported or oversized ZIP directory".into());
    }
    input.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipArchive::new(input).map_err(|e| e.to_string())?;
    if zip.len() as u64 != count {
        return Err("Archive entry limit mismatch".into());
    }
    let top = format!("CleanSlate-{}-windows-x64", descriptor.version);
    let mut names = std::collections::BTreeMap::new();
    let mut total = 0u64;
    // Validate every central-directory entry before creating any output.
    for i in 0..zip.len() {
        let file = zip.by_index(i).map_err(|e| e.to_string())?;
        let name = file.name().map_err(|e| e.to_string())?;
        let name = name.strip_suffix('/').unwrap_or(&name);
        let relative = safe_path(name)?;
        if !(name == top || name.starts_with(&format!("{top}/")))
            || names
                .insert(name.to_ascii_lowercase(), file.is_dir())
                .is_some()
            || file.encrypted()
            || file.is_symlink()
            || file
                .unix_mode()
                .is_some_and(|m| !matches!(m & 0o170000, 0 | 0o040000 | 0o100000))
            || file.external_attributes() & 0x400 != 0
            || file.size() > 512 * 1024 * 1024
        {
            return Err("Unsafe archive entry".into());
        }
        for parent in relative
            .ancestors()
            .skip(1)
            .filter(|p| !p.as_os_str().is_empty())
        {
            if names.get(
                &parent
                    .to_string_lossy()
                    .replace('\\', "/")
                    .to_ascii_lowercase(),
            ) == Some(&false)
            {
                return Err("Archive file/directory collision".into());
            }
        }
        if !file.is_dir()
            && (name == top
                || names
                    .keys()
                    .any(|other| other.starts_with(&format!("{}/", name.to_ascii_lowercase()))))
        {
            return Err("Archive file/directory collision".into());
        }
        total = total
            .checked_add(file.size())
            .ok_or("Archive size overflow")?;
        if total > descriptor.expanded_bytes {
            return Err("Expanded archive size limit exceeded".into());
        }
    }
    cancelled(cancel)?;
    ancestors(destination.parent().ok_or("Missing staging parent")?)?;
    fs::create_dir(destination).map_err(|e| e.to_string())?;
    let mut decoded = 0u64;
    for i in 0..zip.len() {
        cancelled(cancel)?;
        let mut file = zip.by_index(i).map_err(|e| e.to_string())?;
        let name = file.name().map_err(|e| e.to_string())?.into_owned();
        let target = destination.join(safe_path(name.trim_end_matches('/'))?);
        if file.is_dir() {
            fs::create_dir_all(&target).map_err(|e| e.to_string())?;
            ancestors(&target)?;
            continue;
        }
        let parent = target.parent().ok_or("Invalid archive parent")?;
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        ancestors(parent)?;
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&target)
            .map_err(|e| e.to_string())?;
        let mut buffer = [0; 64 * 1024];
        let mut file_bytes = 0;
        loop {
            cancelled(cancel)?;
            let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            decoded += n as u64;
            file_bytes += n as u64;
            if decoded > descriptor.expanded_bytes || file_bytes > file.size() {
                return Err("Decoded archive exceeds signed limits".into());
            }
            output
                .write_all(&buffer[..n])
                .map_err(|e| format!("Cannot stage update: {e}"))?;
        }
        if file_bytes != file.size() {
            return Err("Truncated archive entry".into());
        }
        output.sync_all().map_err(|e| e.to_string())?;
    }
    let root = destination.join(top);
    let manifest = validate_tree(&root, Some(&descriptor.manifest_sha256))?;
    if manifest.version != descriptor.version {
        return Err("Archive version mismatch".into());
    }
    Ok(root)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn signed_descriptor_requires_pinned_identity_and_exact_bytes() {
        let bytes = include_bytes!("../../tests/fixtures/updater-descriptor.json");
        let signature = include_str!("../../tests/fixtures/updater-descriptor.json.minisig");
        let key = include_str!("../../tests/fixtures/updater-test-public-key.txt");
        assert!(Descriptor::verify(bytes, signature, key, "1.0.1").is_ok());
        assert!(Descriptor::verify(bytes, signature, "", "1.0.1").is_err());
        assert!(
            Descriptor::verify(
                bytes,
                signature,
                include_str!("../../assets/update-public-key.txt"),
                "1.0.1"
            )
            .is_err()
        );
        assert!(Descriptor::verify(bytes, signature, key, "1.0.2").is_err());
        let mut changed = bytes.to_vec();
        changed.push(b' ');
        assert!(Descriptor::verify(&changed, signature, key, "1.0.1").is_err());
        assert!(
            Descriptor::verify(
                include_bytes!("../../tests/fixtures/updater-descriptor-duplicate.json"),
                include_str!("../../tests/fixtures/updater-descriptor-duplicate.json.minisig"),
                key,
                "1.0.1"
            )
            .is_err()
        );
    }
    #[test]
    fn extraction_rejects_links_aliases_truncation_and_resource_overruns_without_live_writes() {
        use zip::write::SimpleFileOptions;
        let parent =
            std::env::temp_dir().join(format!("cleanslate-archive-tests-{}", std::process::id()));
        fs::create_dir_all(&parent).unwrap();
        let cancel = AtomicBool::new(false);
        let bytes = include_bytes!("../../tests/fixtures/updater-descriptor.json");
        let mut d: Descriptor = serde_json::from_slice(bytes).unwrap();
        let top = "CleanSlate-1.0.1-windows-x64";
        for (case, name, link) in [
            ("traversal", format!("{top}/../outside"), false),
            ("alias", format!("{top}/CON.txt"), false),
            ("link", format!("{top}/link"), true),
            ("oversize", format!("{top}/file"), false),
        ] {
            let archive = parent.join(format!("{case}.zip"));
            let output = File::create(&archive).unwrap();
            let mut writer = zip::ZipWriter::new(output);
            if link {
                writer
                    .add_symlink(name, "../../outside", SimpleFileOptions::default())
                    .unwrap();
            } else {
                writer
                    .start_file(name, SimpleFileOptions::default())
                    .unwrap();
                writer.write_all(b"data").unwrap();
            }
            writer.finish().unwrap();
            d.bytes = fs::metadata(&archive).unwrap().len();
            d.sha256 = hash_file(&archive).unwrap();
            d.expanded_bytes = if case == "oversize" { 1 } else { 4096 };
            let dest = parent.join(case);
            assert!(extract(&archive, &dest, &d, &cancel).is_err(), "{case}");
            assert!(!dest.exists());
        }
        let archive = parent.join("truncated.zip");
        fs::write(&archive, b"PK").unwrap();
        d.bytes = 2;
        d.sha256 = hash_file(&archive).unwrap();
        assert!(extract(&archive, &parent.join("truncated"), &d, &cancel).is_err());
        d.bytes = 3;
        assert!(extract(&archive, &parent.join("bad-size"), &d, &cancel).is_err());
        fs::remove_dir_all(parent).unwrap();
    }
    #[test]
    fn unsafe_archive_names_are_rejected_before_extraction() {
        for name in [
            "../recording.wav",
            "/CleanSlate.exe",
            "C:/CleanSlate.exe",
            "a\\b",
            "a:stream",
            "CON",
            "nul.txt",
            "a./b",
            "a /b",
            "a//b",
            "a/../b",
        ] {
            assert!(super::safe_path(name).is_err(), "{name}");
        }
        assert!(super::safe_path("licenses/zip-9.0.5/LICENSE").is_ok());
    }
}
