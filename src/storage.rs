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

pub(crate) fn is_local_project(path: &Path) -> bool {
    root()
        .ok()
        .and_then(|root| fs::canonicalize(root.join("Projects")).ok())
        .zip(fs::canonicalize(path).ok())
        .is_some_and(|(root, path)| path.starts_with(root))
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

    #[test]
    fn named_files_in_working_storage_still_reopen_as_named_projects() {
        let recovery = new_project().unwrap();
        let root = recovery.parent().unwrap();
        let source = root.join("voice.wav");
        let writer =
            bwavfile::WaveWriter::create(&source, bwavfile::WaveFmt::new_pcm_mono(48000, 16))
                .unwrap();
        let mut frames = writer.audio_frame_writer().unwrap();
        frames.write_frames(&[0.1; 100]).unwrap();
        frames.end().unwrap();
        let mut project = crate::Project::create(&recovery, vec![source]).unwrap();
        assert!(
            crate::Project::open(&recovery)
                .unwrap()
                .save_destination()
                .is_none()
        );
        let named = root.join("chosen.cleanslate");
        project.save_as(&named).unwrap();
        assert_eq!(
            crate::Project::open(&recovery).unwrap().save_destination(),
            Some(fs::canonicalize(&named).unwrap().as_path())
        );
        let saved = fs::read(&named).unwrap();
        let mut reopened = crate::Project::open(&named).unwrap();
        assert_eq!(
            reopened.save_destination(),
            Some(fs::canonicalize(&named).unwrap().as_path())
        );
        assert!(reopened.localize_for_editing().unwrap());
        reopened
            .set_parameters(crate::AutomixParameters {
                attenuation_db: -12.0,
                ..reopened.parameters()
            })
            .unwrap();
        assert_eq!(fs::read(&named).unwrap(), saved);
        let localized = reopened.path().parent().unwrap().to_owned();
        project.save_as(&recovery).unwrap();
        assert_ne!(project.path(), recovery);
        assert!(
            crate::Project::open(&recovery)
                .unwrap()
                .save_destination()
                .is_some()
        );
        let rehomed = project.path().parent().unwrap().to_owned();
        let external = crate::audio_io::multichannel_tests::root();
        let imported = external.join("imported.cleanslate");
        let victim = fs::read(&named).unwrap();
        let mut forged = serde_json::to_value(&project).unwrap();
        forged["recovery_path"] = serde_json::to_value(&imported).unwrap();
        forged["named_destination"] = serde_json::to_value(&named).unwrap();
        fs::write(&imported, serde_json::to_vec(&forged).unwrap()).unwrap();
        let mut imported_project = crate::Project::open(&imported).unwrap();
        assert_eq!(
            imported_project.save_destination(),
            Some(fs::canonicalize(&imported).unwrap().as_path())
        );
        assert!(imported_project.localize_for_editing().unwrap());
        imported_project
            .set_parameters(crate::AutomixParameters {
                attenuation_db: -6.0,
                ..imported_project.parameters()
            })
            .unwrap();
        imported_project.save_named().unwrap();
        assert_eq!(fs::read(&named).unwrap(), victim);
        assert_eq!(
            crate::Project::open(&imported)
                .unwrap()
                .parameters()
                .attenuation_db,
            -6.0
        );
        let imported_recovery = imported_project.path().parent().unwrap().to_owned();
        fs::remove_dir_all(imported_recovery).unwrap();
        fs::remove_dir_all(external).unwrap();
        fs::remove_dir_all(localized).unwrap();
        fs::remove_dir_all(rehomed).unwrap();
        fs::remove_dir_all(root).unwrap();
    }
}
