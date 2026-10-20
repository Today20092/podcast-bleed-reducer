//! Local working storage is independent of source media and export destinations.
use sha2::{Digest, Sha256};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub(crate) fn root() -> io::Result<PathBuf> {
    if cfg!(test) {
        return Ok(
            std::env::temp_dir().join(format!("cleanslate-working-tests-{}", std::process::id()))
        );
    }
    let absolute = |name| {
        std::env::var_os(name)
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
    };
    let root = match std::env::consts::OS {
        "windows" => absolute("LOCALAPPDATA").map(|p| p.join("CleanSlate")),
        "macos" => absolute("HOME")
            .map(|p| p.join("Library/Application Support/io.github.Today20092.CleanSlate")),
        _ => absolute("XDG_DATA_HOME")
            .or_else(|| absolute("HOME").map(|p| p.join(".local/share")))
            .map(|p| p.join("cleanslate")),
    };
    root.ok_or_else(|| io::Error::other("Local working storage is unavailable. Set a valid user data directory; source folders will not be used for temporary files."))
}

pub(crate) fn new_project() -> io::Result<PathBuf> {
    new_project_in(&root()?.join("Projects"))
}

fn new_project_in(root: &Path) -> io::Result<PathBuf> {
    fs::create_dir_all(root)
        .map_err(|e| crate::storage_io::context("create local project storage", root, e))?;
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    loop {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let sequence = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let directory = root.join(format!("session-{}-{nonce}-{sequence}", std::process::id()));
        match fs::create_dir(&directory) {
            Ok(()) => return Ok(directory.join("CleanSlate.cleanslate")),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(e) => {
                return Err(crate::storage_io::context(
                    "create local project",
                    &directory,
                    e,
                ));
            }
        }
    }
}

pub(crate) fn working_directory(project: &Path) -> io::Result<PathBuf> {
    let project = fs::canonicalize(project)?;
    let identity = format!(
        "{:x}",
        Sha256::digest(project.as_os_str().as_encoded_bytes())
    );
    let directory = root()?.join("Work").join(identity);
    fs::create_dir_all(&directory)
        .map_err(|e| crate::storage_io::context("create local working storage", &directory, e))?;
    Ok(directory)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn projects_are_unique_and_never_fall_back_to_source_or_current_directory() {
        let root = crate::audio_io::multichannel_tests::root();
        let a = new_project_in(&root).unwrap();
        let b = new_project_in(&root).unwrap();
        assert_ne!(a, b);
        assert!(a.starts_with(&root));
        assert!(a.parent().unwrap().is_dir());
        let blocked = root.join("blocked");
        fs::write(&blocked, b"keep").unwrap();
        let error = new_project_in(&blocked).unwrap_err().to_string();
        assert!(error.contains("create local project storage"));
        assert!(error.contains(&blocked.display().to_string()));
        assert_eq!(fs::read(&blocked).unwrap(), b"keep");
        fs::remove_dir_all(root).unwrap();
    }
}
