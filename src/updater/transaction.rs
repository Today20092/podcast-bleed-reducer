//! Recoverable installation transaction.
use super::{
    Result, cancelled,
    package::{self, Descriptor},
};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::atomic::AtomicBool,
    time::{Duration, Instant},
};

#[derive(Debug, Clone)]
pub struct Prepared {
    pub transaction: PathBuf,
    pub version: String,
    pub recovery: PathBuf,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum State {
    Prepared,
    MovingOld,
    OldMoved,
    MovingNew,
    CandidateInstalled,
    HealthPending,
    Committed,
    CleanupAcknowledged,
    RollingBack,
    RolledBack,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Journal {
    schema: u32,
    nonce: String,
    install_name: String,
    old_hash: String,
    new_hash: String,
    old_version: String,
    new_version: String,
    state: State,
}

fn nonce() -> Result<String> {
    let mut bytes = [0; 24];
    getrandom::fill(&mut bytes).map_err(|e| e.to_string())?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}
fn fresh(parent: &Path) -> Result<PathBuf> {
    let dir = parent.join(format!(".cleanslate-update-{}", nonce()?));
    fs::create_dir(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}
fn write_json(path: &Path, value: &impl Serialize) -> Result<()> {
    let temporary = path.with_extension(format!("{}.tmp", nonce()?));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|e| e.to_string())?;
    file.write_all(&serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    drop(file);
    fs::rename(&temporary, path).map_err(|e| e.to_string())
}
fn save(tx: &Path, j: &mut Journal, state: State) -> Result<()> {
    j.state = state;
    write_json(&tx.join("journal.json"), j)
}
fn read(tx: &Path) -> Result<Journal> {
    package::ancestors(tx)?;
    let bytes = fs::read(tx.join("journal.json")).map_err(|e| e.to_string())?;
    if bytes.len() > 16 * 1024 {
        return Err("Oversized recovery journal".into());
    }
    let j: Journal = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    if j.schema != 1
        || j.nonce.len() != 48
        || !j.nonce.bytes().all(|c| c.is_ascii_hexdigit())
        || tx.file_name().and_then(|n| n.to_str())
            != Some(&format!(".cleanslate-update-{}", j.nonce))
        || !j.install_name.starts_with("CleanSlate-")
        || package::safe_path(&j.install_name)?.components().count() != 1
        || j.old_hash.len() != 64
        || j.new_hash.len() != 64
    {
        return Err("Invalid recovery identity or paths".into());
    }
    Ok(j)
}
fn install(tx: &Path, j: &Journal) -> Result<PathBuf> {
    Ok(tx
        .parent()
        .ok_or("Missing recovery parent")?
        .join(&j.install_name))
}
fn index(root: &Path) -> Result<PathBuf> {
    let name = package::hash_bytes(root.to_string_lossy().to_ascii_lowercase().as_bytes());
    Ok(root
        .parent()
        .ok_or("Invalid installation root")?
        .join(format!(".cleanslate-install-{}.json", &name[..24])))
}
pub struct InstallationLock {
    _file: File,
}
impl InstallationLock {
    fn acquire(root: &Path, exclusive: bool) -> Result<Self> {
        let path = index(root)?.with_extension("lock");
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)
            .map_err(|e| e.to_string())?;
        let locked = if exclusive {
            file.try_lock()
        } else {
            file.try_lock_shared()
        };
        locked.map_err(|_| "Another application instance or update is using this installation. Close the other instance and retry.".to_string())?;
        Ok(Self { _file: file })
    }
    pub fn startup() -> Result<Option<Self>> {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let root = exe.parent().ok_or("Missing application folder")?;
        if exe.file_name().and_then(|s| s.to_str()) != Some("CleanSlate.exe")
            || !root.join("package-manifest.json").exists()
        {
            return Ok(None);
        }
        if !health_mode() && index(root)?.exists() {
            let tx: PathBuf =
                serde_json::from_slice(&fs::read(index(root)?).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            let j = read(&tx)?;
            if install(&tx, &j)? != root {
                return Err("Recovery index does not match this installation".into());
            }
            if !matches!(
                j.state,
                State::Committed | State::CleanupAcknowledged | State::RolledBack | State::Prepared
            ) {
                recover(&tx)?;
            }
        }
        Ok(Some(Self::acquire(root, false)?))
    }
}
fn transaction_lock(tx: &Path) -> Result<InstallationLock> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(tx.join("transaction.lock"))
        .map_err(|e| e.to_string())?;
    file.try_lock()
        .map_err(|_| "The update helper is still running. Wait for it to finish.".to_string())?;
    Ok(InstallationLock { _file: file })
}
fn staging_lock(root: &Path) -> Result<InstallationLock> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(index(root)?.with_extension("download.lock"))
        .map_err(|e| e.to_string())?;
    file.try_lock()
        .map_err(|_| "Another instance is staging or cleaning an update".to_string())?;
    Ok(InstallationLock { _file: file })
}

#[cfg(windows)]
pub fn storage_check(root: &Path, needed: u64) -> Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        GetDiskFreeSpaceExW, GetDriveTypeW, GetVolumeInformationW, GetVolumePathNameW,
    };
    package::ancestors(root)?;
    let full: Vec<_> = root.as_os_str().encode_wide().chain(Some(0)).collect();
    let mut volume = [0u16; 1024];
    let mut fs_name = [0u16; 64];
    let mut free = 0;
    // Buffers are owned here and all Windows strings are NUL terminated.
    unsafe {
        if GetVolumePathNameW(full.as_ptr(), volume.as_mut_ptr(), volume.len() as u32) == 0
            || GetDriveTypeW(volume.as_ptr()) != 3
            || GetVolumeInformationW(
                volume.as_ptr(),
                std::ptr::null_mut(),
                0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                fs_name.as_mut_ptr(),
                fs_name.len() as u32,
            ) == 0
            || String::from_utf16_lossy(
                &fs_name[..fs_name.iter().position(|c| *c == 0).unwrap_or(0)],
            ) != "NTFS"
            || GetDiskFreeSpaceExW(
                full.as_ptr(),
                &mut free,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            ) == 0
        {
            return Err("Automatic updates require a writable local fixed NTFS volume. Use manual download for this location.".into());
        }
    }
    if free < needed {
        return Err(format!(
            "Not enough free space for update staging (need {needed} bytes). Free space or use manual download."
        ));
    }
    Ok(())
}
#[cfg(not(windows))]
pub fn storage_check(_: &Path, _: u64) -> Result<()> {
    Err("Automatic installation is supported only for Windows x64 portable packages. Use the matching manual download.".into())
}

pub fn eligible() -> Result<PathBuf> {
    if !cfg!(all(windows, target_arch = "x86_64")) {
        return Err("This platform uses manual download.".into());
    }
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    if exe.file_name().and_then(|s| s.to_str()) != Some("CleanSlate.exe") {
        return Err("Development executables use manual download.".into());
    }
    let root = exe.parent().ok_or("No application folder")?.to_path_buf();
    if !root
        .file_name()
        .and_then(|s| s.to_str())
        .is_some_and(|s| s.starts_with("CleanSlate-"))
    {
        return Err(
            "Extract into a clean CleanSlate package folder to enable installation.".into(),
        );
    }
    let m = package::validate_tree(&root, None)?;
    if m.version != env!("CARGO_PKG_VERSION") {
        return Err("Running executable/package version mismatch".into());
    }
    storage_check(&root, 64 * 1024 * 1024)?;
    if index(&root)?.exists() {
        let old: PathBuf =
            serde_json::from_slice(&fs::read(index(&root)?).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        if read(&old)?.state != State::RolledBack {
            return Err("An earlier update is staged, needs recovery, or retains a previous package. Resolve it or acknowledge cleanup before another download.".into());
        }
    }
    Ok(root)
}

pub fn stage(
    offer: &super::Offer,
    cancel: &AtomicBool,
    progress: &mut impl FnMut(String),
) -> Result<Prepared> {
    let root = eligible()?;
    let _staging = staging_lock(&root)?;
    eligible()?;
    let archive_name = format!("CleanSlate-{}-windows-x64.zip", offer.version);
    let metadata_name = format!("{archive_name}.update.json");
    let mut metadata = Vec::new();
    let mut signature = Vec::new();
    for (name, output, max) in [
        (&metadata_name, &mut metadata, 64 * 1024),
        (&format!("{metadata_name}.minisig"), &mut signature, 4096),
    ] {
        let asset = offer.asset(name)?;
        if asset.size == 0 || asset.size > max {
            return Err("Invalid signed metadata asset size".into());
        }
        super::transfer(
            &super::download_url(&offer.tag, name)?,
            output,
            max,
            cancel,
            &mut |_| {},
        )?;
        if output.len() as u64 != asset.size {
            return Err("Truncated signed metadata".into());
        }
    }
    let d = Descriptor::verify(
        &metadata,
        std::str::from_utf8(&signature).map_err(|e| e.to_string())?,
        super::PUBLIC_KEY,
        &offer.version,
    )?;
    if offer.asset(&archive_name)?.size != d.bytes {
        return Err("Archive/API size mismatch".into());
    }
    let old_manifest = package::validate_tree(&root, None)?;
    let old_size: u64 = old_manifest.files.iter().map(|f| f.bytes).sum();
    storage_check(
        &root,
        d.bytes + d.expanded_bytes + old_size + 64 * 1024 * 1024,
    )?;
    let tx = fresh(root.parent().ok_or("No installation parent")?)?;
    let result = (|| {
        fs::write(tx.join("descriptor.json"), &metadata).map_err(|e| e.to_string())?;
        fs::write(tx.join("descriptor.minisig"), &signature).map_err(|e| e.to_string())?;
        progress("Downloading complete portable package…".into());
        let mut archive = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(tx.join("package.zip"))
            .map_err(|e| e.to_string())?;
        let count = super::transfer(
            &super::download_url(&offer.tag, &archive_name)?,
            &mut archive,
            d.bytes,
            cancel,
            &mut |n| progress(format!("Downloading {n} / {} bytes", d.bytes)),
        )?;
        archive.sync_all().map_err(|e| e.to_string())?;
        drop(archive);
        if count != d.bytes {
            return Err("Truncated package download".into());
        }
        progress("Verifying and extracting complete package…".into());
        let extracted = package::extract(&tx.join("package.zip"), &tx.join("extract"), &d, cancel)?;
        fs::rename(extracted, tx.join("candidate")).map_err(|e| e.to_string())?;
        prepare(&root, &tx, &d, cancel)
    })();
    if result.is_err() {
        // This directory was created exclusively by this invocation; no live tree moved yet.
        let _ = fs::remove_dir_all(&tx);
    }
    result
}
fn prepare(root: &Path, tx: &Path, d: &Descriptor, cancel: &AtomicBool) -> Result<Prepared> {
    let old = package::validate_tree(root, None)?;
    fs::create_dir(tx.join("helper")).map_err(|e| e.to_string())?;
    for file in old
        .files
        .iter()
        .map(|f| f.path.as_str())
        .chain(Some("package-manifest.json"))
    {
        cancelled(cancel)?;
        let target = tx.join("helper").join(package::safe_path(file)?);
        fs::create_dir_all(target.parent().ok_or("No helper parent")?)
            .map_err(|e| e.to_string())?;
        fs::copy(root.join(file), target).map_err(|e| e.to_string())?;
    }
    let old_hash = package::hash_file(&root.join("package-manifest.json"))?;
    package::validate_tree(&tx.join("helper"), Some(&old_hash))?;
    let mut j = Journal {
        schema: 1,
        nonce: tx
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or("Invalid transaction name")?
            .trim_start_matches(".cleanslate-update-")
            .into(),
        install_name: root
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or("Invalid installation name")?
            .into(),
        old_hash,
        new_hash: d.manifest_sha256.clone(),
        old_version: old.version,
        new_version: d.version.clone(),
        state: State::Prepared,
    };
    save(tx, &mut j, State::Prepared)?;
    let recovery = tx.join("helper/CleanSlate.exe");
    fs::write(tx.join("RECOVERY.txt"), format!("Keep this directory and the backup until a later successful launch.\nIf the normal app path is missing, run:\n\"{}\" --update-recover \"{}\"\nThe previous app will be restored to {}.\nDo not move projects here.\n", recovery.display(), tx.display(), root.display())).map_err(|e| e.to_string())?;
    write_json(&index(root)?, &tx)?;
    Ok(Prepared {
        transaction: tx.into(),
        version: d.version.clone(),
        recovery,
    })
}

fn rename(from: &Path, to: &Path) -> Result<()> {
    let mut last = None;
    for _ in 0..20 {
        match fs::rename(from, to) {
            Ok(()) => return Ok(()),
            Err(e) => last = Some(e),
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Err(format!(
        "Cannot move {} to {}: {}. Check permissions, free space and antivirus/file locks; run the retained recovery helper.",
        from.display(),
        to.display(),
        last.unwrap()
    ))
}
fn matching(root: &Path, hash: &str) -> bool {
    root.is_dir() && package::validate_tree(root, Some(hash)).is_ok()
}
pub fn recover(tx: &Path) -> Result<()> {
    let mut j = read(tx)?;
    let _tx_lock = transaction_lock(tx)?;
    let root = install(tx, &j)?;
    let _lock = InstallationLock::acquire(&root, true)?;
    restore(tx, &mut j)
}
fn restore(tx: &Path, j: &mut Journal) -> Result<()> {
    let root = install(tx, j)?;
    if matches!(j.state, State::Committed | State::CleanupAcknowledged) {
        return Ok(());
    }
    if matching(&root, &j.old_hash) {
        save(tx, j, State::RolledBack)?;
        return Ok(());
    }
    let backup = tx.join("backup");
    if !matching(&backup, &j.old_hash) {
        return Err(format!(
            "Cannot establish a runnable previous package. Nothing was removed. Inspect {} and {}.",
            root.display(),
            backup.display()
        ));
    }
    save(tx, j, State::RollingBack)?;
    if root.exists() {
        if !matching(&root, &j.new_hash) || tx.join("failed").exists() {
            return Err("Candidate identity is ambiguous; no tree was removed. Close the candidate and resume recovery.".into());
        }
        rename(&root, &tx.join("failed"))?;
    }
    rename(&backup, &root)?;
    save(tx, j, State::RolledBack)
}
fn hidden(command: &mut Command) -> &mut Command {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
}
fn child_wait(child: &mut Child, timeout: Duration) -> Result<()> {
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
            return if status.success() {
                Ok(())
            } else {
                Err(format!("Candidate process failed: {status}"))
            };
        }
        if Instant::now() > deadline {
            child.kill().map_err(|e| e.to_string())?;
            child.wait().map_err(|e| e.to_string())?;
            return Err("Candidate health check timed out".into());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}
fn health(root: &Path, j: &Journal) -> Result<Child> {
    let exe = root.join("CleanSlate.exe");
    let mut check = hidden(Command::new(&exe).arg("--package-self-check"))
        .current_dir(root)
        .spawn()
        .map_err(|e| e.to_string())?;
    child_wait(&mut check, Duration::from_secs(120))?;
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    listener.set_nonblocking(true).map_err(|e| e.to_string())?;
    let mut child = hidden(
        Command::new(&exe).args([
            "--update-health",
            &listener
                .local_addr()
                .map_err(|e| e.to_string())?
                .port()
                .to_string(),
            &j.nonce,
            &j.new_hash,
        ]),
    )
    .current_dir(root)
    .spawn()
    .map_err(|e| e.to_string())?;
    let expected = format!(
        "{}:{}:{}:{}",
        j.nonce,
        child.id(),
        j.new_version,
        j.new_hash
    );
    let result = (|| {
        let deadline = Instant::now() + Duration::from_secs(45);
        loop {
            if child.try_wait().map_err(|e| e.to_string())?.is_some() {
                return Err("Candidate exited before GUI health acknowledgment".into());
            }
            match listener.accept() {
                Ok((stream, _)) => {
                    stream
                        .set_read_timeout(Some(Duration::from_secs(2)))
                        .map_err(|e| e.to_string())?;
                    let mut bytes = Vec::new();
                    stream
                        .take(1024)
                        .read_to_end(&mut bytes)
                        .map_err(|e| e.to_string())?;
                    if bytes != expected.as_bytes() {
                        return Err("Candidate health identity mismatch".into());
                    }
                    break;
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(e) => return Err(e.to_string()),
            }
            if Instant::now() > deadline {
                return Err("Candidate did not initialize its GUI in time".into());
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        for _ in 0..30 {
            if child.try_wait().map_err(|e| e.to_string())?.is_some() {
                return Err("Candidate crashed during startup observation".into());
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        Ok(())
    })();
    if let Err(error) = result {
        let _ = child.kill();
        let _ = child.wait();
        return Err(error);
    }
    Ok(child)
}

pub fn health_mode() -> bool {
    std::env::args_os().any(|a| a == "--update-health")
}
pub fn health_acknowledge() -> Result<()> {
    #[cfg(all(feature = "updater-test-key", debug_assertions))]
    if std::env::var("CLEANSLATE_UPDATE_FIXTURE_FAILURE").as_deref() == Ok("gui-timeout") {
        return Ok(());
    }
    let args: Vec<_> = std::env::args().collect();
    let Some(i) = args.iter().position(|a| a == "--update-health") else {
        return Ok(());
    };
    let port: u16 = args
        .get(i + 1)
        .ok_or("Missing health port")?
        .parse()
        .map_err(|_| "Invalid health port")?;
    let nonce = args.get(i + 2).ok_or("Missing health nonce")?;
    let hash = args.get(i + 3).ok_or("Missing health manifest identity")?;
    let root = std::env::current_exe()
        .map_err(|e| e.to_string())?
        .parent()
        .ok_or("No candidate root")?
        .to_path_buf();
    package::validate_tree(&root, Some(hash))?;
    let mut socket = TcpStream::connect_timeout(
        &std::net::SocketAddr::from(([127, 0, 0, 1], port)),
        Duration::from_secs(5),
    )
    .map_err(|e| e.to_string())?;
    socket
        .write_all(
            format!(
                "{nonce}:{}:{}:{hash}",
                std::process::id(),
                env!("CARGO_PKG_VERSION")
            )
            .as_bytes(),
        )
        .map_err(|e| e.to_string())
}

#[cfg(windows)]
fn wait_parent(pid: u32, root: &Path, ready: &mut impl FnMut() -> Result<()>) -> Result<()> {
    use windows_sys::Win32::{
        Foundation::CloseHandle,
        System::Threading::{
            OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE,
            QueryFullProcessImageNameW, WaitForSingleObject,
        },
    };
    // Open before acknowledging readiness so PID reuse cannot change which process is awaited.
    unsafe {
        let handle = OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
            0,
            pid,
        );
        if handle.is_null() {
            return Err("Cannot acquire the actual parent process handle".into());
        }
        let result = (|| {
            let mut buffer = [0u16; 32768];
            let mut len = buffer.len() as u32;
            if QueryFullProcessImageNameW(handle, 0, buffer.as_mut_ptr(), &mut len) == 0
                || Path::new(&String::from_utf16_lossy(&buffer[..len as usize]))
                    != root.join("CleanSlate.exe")
            {
                return Err("Update parent identity mismatch".into());
            }
            ready()?;
            if WaitForSingleObject(handle, 120_000) != 0 {
                return Err("Parent did not exit; installation retained".into());
            }
            Ok(())
        })();
        CloseHandle(handle);
        result
    }
}
#[cfg(not(windows))]
fn wait_parent(_: u32, _: &Path, _: &mut impl FnMut() -> Result<()>) -> Result<()> {
    Err("Use manual download on this platform".into())
}

pub fn launch(prepared: &Prepared) -> Result<()> {
    launch_mode(prepared, false)
}
pub fn launch_recovery(prepared: &Prepared) -> Result<()> {
    launch_mode(prepared, true)
}
fn launch_mode(prepared: &Prepared, recovery: bool) -> Result<()> {
    let j = read(&prepared.transaction)?;
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    listener.set_nonblocking(true).map_err(|e| e.to_string())?;
    let mut child = hidden(
        Command::new(&prepared.recovery)
            .arg("--update-helper")
            .arg(&prepared.transaction)
            .arg(&j.nonce)
            .arg(std::process::id().to_string())
            .arg(
                listener
                    .local_addr()
                    .map_err(|e| e.to_string())?
                    .port()
                    .to_string(),
            )
            .arg(if recovery { "recover" } else { "install" }),
    )
    .current_dir(&prepared.transaction)
    .spawn()
    .map_err(|e| e.to_string())?;
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Ok((stream, _)) = listener.accept() {
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .map_err(|e| e.to_string())?;
            let mut bytes = Vec::new();
            stream
                .take(256)
                .read_to_end(&mut bytes)
                .map_err(|e| e.to_string())?;
            if bytes == j.nonce.as_bytes() {
                return Ok(());
            }
            break;
        }
        if child.try_wait().map_err(|e| e.to_string())?.is_some() || Instant::now() > deadline {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let _ = child.kill();
    let _ = child.wait();
    Err("Update helper could not confirm readiness; the current session remains open.".into())
}
fn apply(tx: &Path, j: &mut Journal) -> Result<()> {
    let root = install(tx, j)?;
    let _installation = InstallationLock::acquire(&root, true)?;
    package::validate_tree(&root, Some(&j.old_hash))?;
    let bytes = fs::read(tx.join("descriptor.json")).map_err(|e| e.to_string())?;
    let signature = fs::read_to_string(tx.join("descriptor.minisig")).map_err(|e| e.to_string())?;
    let d = Descriptor::verify(&bytes, &signature, super::PUBLIC_KEY, &j.new_version)?;
    if d.manifest_sha256 != j.new_hash {
        return Err("Journal/descriptor identity mismatch".into());
    }
    package::validate_tree(&tx.join("candidate"), Some(&j.new_hash))?;
    save(tx, j, State::MovingOld)?;
    rename(&root, &tx.join("backup"))?;
    #[cfg(all(feature = "updater-test-key", debug_assertions))]
    if std::env::var("CLEANSLATE_UPDATE_FIXTURE_FAILURE").as_deref() == Ok("interrupted-swap") {
        std::process::exit(73);
    }
    save(tx, j, State::OldMoved)?;
    save(tx, j, State::MovingNew)?;
    rename(&tx.join("candidate"), &root)?;
    save(tx, j, State::CandidateInstalled)?;
    save(tx, j, State::HealthPending)?;
    drop(_installation);
    let mut candidate = health(&root, j)?;
    if let Err(error) = save(tx, j, State::Committed) {
        let _ = candidate.kill();
        let _ = candidate.wait();
        return Err(error);
    }
    Ok(())
}

pub fn cli() -> Option<i32> {
    let args: Vec<_> = std::env::args_os().collect();
    let mode = args.get(1)?.to_str()?;
    #[cfg(all(feature = "updater-test-key", debug_assertions))]
    if mode == "--update-fixture-cleanup" {
        let result = previous().and_then(|p| cleanup(&p.ok_or("No retained fixture package")?));
        if let Err(error) = &result {
            eprintln!("{error}");
        }
        return Some(if result.is_ok() { 0 } else { 1 });
    }
    #[cfg(all(feature = "updater-test-key", debug_assertions))]
    if mode == "--update-fixture" {
        let result = (|| {
            let root = std::env::current_exe()
                .map_err(|e| e.to_string())?
                .parent()
                .ok_or("Missing fixture root")?
                .to_path_buf();
            let _lock = InstallationLock::acquire(&root, false)?;
            let archive = PathBuf::from(args.get(2).ok_or("Missing fixture archive")?);
            let metadata = fs::read(args.get(3).ok_or("Missing fixture metadata")?)
                .map_err(|e| e.to_string())?;
            let signature = fs::read_to_string(args.get(4).ok_or("Missing fixture signature")?)
                .map_err(|e| e.to_string())?;
            let release_path = args
                .get(5)
                .and_then(|s| s.to_str())
                .ok_or("Missing fixture release metadata")?;
            let offer = super::select_release(
                &fs::read(release_path).map_err(|e| e.to_string())?,
                env!("CARGO_PKG_VERSION"),
                false,
            )?
            .ok_or("No newer eligible fixture release")?;
            let d = Descriptor::verify(&metadata, &signature, super::PUBLIC_KEY, &offer.version)?;
            if offer.asset(&d.archive)?.size != d.bytes {
                return Err("Fixture release asset mismatch".into());
            }
            let tx = fresh(root.parent().ok_or("No fixture parent")?)?;
            let extracted =
                package::extract(&archive, &tx.join("extract"), &d, &AtomicBool::new(false))?;
            fs::rename(extracted, tx.join("candidate")).map_err(|e| e.to_string())?;
            fs::write(tx.join("descriptor.json"), metadata).map_err(|e| e.to_string())?;
            fs::write(tx.join("descriptor.minisig"), signature).map_err(|e| e.to_string())?;
            let prepared = prepare(&root, &tx, &d, &AtomicBool::new(false))?;
            launch(&prepared)
        })();
        if let Err(error) = &result {
            eprintln!("{error}");
        }
        return Some(if result.is_ok() { 0 } else { 1 });
    }
    if !matches!(mode, "--update-helper" | "--update-recover") {
        return None;
    }
    let result = (|| {
        let tx = PathBuf::from(args.get(2).ok_or("Missing transaction")?);
        let mut j = read(&tx)?;
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        if exe != tx.join("helper/CleanSlate.exe") {
            return Err("Recovery must use the retained trusted helper".into());
        }
        package::validate_tree(&tx.join("helper"), Some(&j.old_hash))?;
        if mode == "--update-recover" {
            recover(&tx)?;
        } else {
            let _lock = transaction_lock(&tx)?;
            if args.get(3).and_then(|a| a.to_str()) != Some(&j.nonce) {
                return Err("Invalid helper nonce".into());
            }
            let pid = args
                .get(4)
                .and_then(|a| a.to_str())
                .ok_or("Missing parent")?
                .parse::<u32>()
                .map_err(|e| e.to_string())?;
            let port = args
                .get(5)
                .and_then(|a| a.to_str())
                .ok_or("Missing ready port")?
                .parse::<u16>()
                .map_err(|e| e.to_string())?;
            wait_parent(pid, &install(&tx, &j)?, &mut || {
                let mut stream = TcpStream::connect((std::net::Ipv4Addr::LOCALHOST, port))
                    .map_err(|e| e.to_string())?;
                stream
                    .write_all(j.nonce.as_bytes())
                    .map_err(|e| e.to_string())
            })?;
            if args.get(6).and_then(|s| s.to_str()) == Some("recover") {
                let root = install(&tx, &j)?;
                let lock = InstallationLock::acquire(&root, true)?;
                restore(&tx, &mut j)?;
                drop(lock);
                hidden(&mut Command::new(root.join("CleanSlate.exe")))
                    .current_dir(&root)
                    .spawn()
                    .map_err(|e| e.to_string())?;
                return Ok(());
            }
            if let Err(error) = apply(&tx, &mut j) {
                let root = install(&tx, &j)?;
                let _install_lock = InstallationLock::acquire(&root, true)?;
                let recovery = restore(&tx, &mut j);
                drop(_install_lock);
                let detail = format!(
                    "Update failed: {error}\nRecovery: {recovery:?}\nHelper: {}",
                    exe.display()
                );
                fs::write(tx.join("failure.txt"), &detail).map_err(|e| e.to_string())?;
                if recovery.is_ok() {
                    let _ = hidden(&mut Command::new(root.join("CleanSlate.exe")))
                        .current_dir(&root)
                        .spawn();
                }
                return Err(detail);
            }
        }
        Ok(())
    })();
    if let Err(error) = result {
        rfd::MessageDialog::new()
            .set_title("CleanSlate update recovery")
            .set_description(&error)
            .set_level(rfd::MessageLevel::Error)
            .show();
        Some(1)
    } else {
        Some(0)
    }
}

pub fn pending() -> Result<Option<Prepared>> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let root = exe.parent().ok_or("No installation folder")?;
    let index = index(root)?;
    if !index.exists() {
        return Ok(None);
    }
    let tx: PathBuf = serde_json::from_slice(&fs::read(index).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    let j = read(&tx)?;
    if install(&tx, &j)? != root {
        return Err("Recovery index identity mismatch".into());
    }
    if matches!(
        j.state,
        State::Committed | State::CleanupAcknowledged | State::RolledBack
    ) {
        return Ok(None);
    }
    Ok(Some(Prepared {
        transaction: tx.clone(),
        version: j.new_version,
        recovery: tx.join("helper/CleanSlate.exe"),
    }))
}
pub fn previous() -> Result<Option<Prepared>> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let root = exe.parent().ok_or("No package root")?;
    if !index(root)?.exists() {
        return Ok(None);
    }
    let tx: PathBuf = serde_json::from_slice(&fs::read(index(root)?).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    let j = read(&tx)?;
    if install(&tx, &j)? != root
        || !matches!(j.state, State::Committed | State::CleanupAcknowledged)
    {
        return Ok(None);
    }
    Ok(Some(Prepared {
        transaction: tx.clone(),
        version: j.old_version,
        recovery: tx.join("helper/CleanSlate.exe"),
    }))
}
pub fn cleanup(prepared: &Prepared) -> Result<()> {
    let tx = &prepared.transaction;
    let mut j = read(tx)?;
    let root = install(tx, &j)?;
    let _staging = staging_lock(&root)?;
    let _lock = transaction_lock(tx)?;
    if !matches!(j.state, State::Committed | State::CleanupAcknowledged) {
        return Err("Only a successfully installed package may acknowledge cleanup".into());
    }
    package::validate_tree(&root, Some(&j.new_hash))?;
    let retained = tx.join("cleanup-manifest.json");
    if j.state == State::Committed {
        package::validate_tree(&tx.join("backup"), Some(&j.old_hash))?;
        fs::copy(tx.join("backup/package-manifest.json"), &retained).map_err(|e| e.to_string())?;
        save(tx, &mut j, State::CleanupAcknowledged)?;
    }
    let bytes = fs::read(&retained).map_err(|e| e.to_string())?;
    if package::hash_bytes(&bytes) != j.old_hash {
        return Err("Cleanup manifest identity mismatch".into());
    }
    let manifest: package::PackageManifest =
        serde_json::from_slice(bytes.strip_prefix(&[239, 187, 191]).unwrap_or(&bytes))
            .map_err(|e| e.to_string())?;
    // Interrupted cleanup may retry missing files; unrelated entries stay untouched.
    for name in ["helper", "backup"] {
        let dir = tx.join(name);
        if !dir.exists() {
            continue;
        }
        package::ancestors(&dir)?;
        for file in &manifest.files {
            let path = dir.join(package::safe_path(&file.path)?);
            if path.exists() {
                package::ancestors(&path)?;
                if package::hash_file(&path)? != file.sha256 {
                    return Err("Modified previous-package file; cleanup stopped".into());
                }
                fs::remove_file(path).map_err(|e| e.to_string())?;
            }
        }
        let path = dir.join("package-manifest.json");
        if path.exists() {
            package::regular(&path)?;
            if package::hash_file(&path)? != j.old_hash {
                return Err("Changed previous manifest".into());
            }
            fs::remove_file(path).map_err(|e| e.to_string())?;
        }
        fn empty_dirs(path: &Path) -> Result<()> {
            for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
                let path = entry.map_err(|e| e.to_string())?.path();
                package::regular(&path)?;
                if !path.is_dir() {
                    return Err("Unrelated retained file; cleanup stopped".into());
                }
                empty_dirs(&path)?;
            }
            fs::remove_dir(path).map_err(|e| e.to_string())
        }
        empty_dirs(&dir)?;
    }
    if tx.join("extract").exists() {
        fs::remove_dir(tx.join("extract")).map_err(|e| e.to_string())?;
    }
    for name in [
        "package.zip",
        "descriptor.json",
        "descriptor.minisig",
        "RECOVERY.txt",
        "failure.txt",
    ] {
        let path = tx.join(name);
        if path.exists() {
            package::regular(&path)?;
            fs::remove_file(path).map_err(|e| e.to_string())?;
        }
    }
    // Remove the index before its journal; an interrupted final cleanup cannot block startup.
    fs::remove_file(index(&root)?).map_err(|e| e.to_string())?;
    drop(_lock);
    for name in ["journal.json", "transaction.lock", "cleanup-manifest.json"] {
        let path = tx.join(name);
        if path.exists() {
            package::regular(&path)?;
            fs::remove_file(path).map_err(|e| e.to_string())?;
        }
    }
    fs::remove_dir(tx).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[cfg(windows)]
    fn disk_space_preflight_refuses_an_unfulfillable_budget_without_mutating_the_installation() {
        let root = std::env::temp_dir().join(format!("cleanslate-space-{}", nonce().unwrap()));
        fs::create_dir(&root).unwrap();
        fs::write(root.join("recording.wav"), b"preserve").unwrap();
        let error = storage_check(&root, u64::MAX).unwrap_err();
        assert!(error.contains("Not enough free space"), "{error}");
        assert_eq!(fs::read(root.join("recording.wav")).unwrap(), b"preserve");
        fs::remove_dir_all(root).unwrap();
    }
    fn fixture(root: &Path, version: &str) {
        fs::create_dir_all(root.join("licenses")).unwrap();
        let mut files = Vec::new();
        for name in [
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
            "licenses/test.txt",
        ] {
            let bytes = format!("synthetic {version} {name}").into_bytes();
            fs::write(root.join(name), &bytes).unwrap();
            files.push(package::OwnedFile {
                path: name.into(),
                sha256: package::hash_bytes(&bytes),
                bytes: bytes.len() as u64,
            });
        }
        let m = package::PackageManifest {
            schema: 1,
            product: "CleanSlate".into(),
            package_kind: "portable-zip".into(),
            entrypoint: "CleanSlate.exe".into(),
            version: version.into(),
            platform: "windows-x64".into(),
            commit: "fixture".into(),
            dirty: true,
            rustc: "fixture".into(),
            cargo_lock_sha256: "0".repeat(64),
            files,
        };
        fs::write(
            root.join("package-manifest.json"),
            serde_json::to_vec(&m).unwrap(),
        )
        .unwrap();
    }
    #[test]
    fn preparation_write_failure_never_publishes_a_dangling_installation_index() {
        let parent = fs::canonicalize(std::env::temp_dir())
            .unwrap()
            .join(format!("cleanslate-prepare-failure-{}", nonce().unwrap()));
        fs::create_dir(&parent).unwrap();
        let root = parent.join("CleanSlate-1.0.0-windows-x64");
        fixture(&root, "1.0.0");
        let tx = fresh(&parent).unwrap();
        fs::create_dir(tx.join("RECOVERY.txt")).unwrap();
        let d: Descriptor = serde_json::from_slice(include_bytes!(
            "../../tests/fixtures/updater-descriptor.json"
        ))
        .unwrap();
        assert!(prepare(&root, &tx, &d, &AtomicBool::new(false)).is_err());
        assert!(!index(&root).unwrap().exists());
        package::validate_tree(&root, None).unwrap();
        fs::remove_dir_all(parent).unwrap();
    }
    #[test]
    fn cleanup_preserves_unrelated_files_and_retries_an_interrupted_acknowledgment() {
        let parent = fs::canonicalize(std::env::temp_dir())
            .unwrap()
            .join(format!("cleanslate-cleanup-{}", nonce().unwrap()));
        fs::create_dir(&parent).unwrap();
        let root = parent.join("CleanSlate-1.0.0-windows-x64");
        fixture(&root, "1.0.1");
        package::validate_tree(&root, None).unwrap();
        let tx = fresh(&parent).unwrap();
        fixture(&tx.join("backup"), "1.0.0");
        fixture(&tx.join("helper"), "1.0.0");
        let mut j = Journal {
            schema: 1,
            nonce: tx
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .trim_start_matches(".cleanslate-update-")
                .into(),
            install_name: root.file_name().unwrap().to_str().unwrap().into(),
            old_hash: package::hash_file(&tx.join("backup/package-manifest.json")).unwrap(),
            new_hash: package::hash_file(&root.join("package-manifest.json")).unwrap(),
            old_version: "1.0.0".into(),
            new_version: "1.0.1".into(),
            state: State::Committed,
        };
        save(&tx, &mut j, State::Committed).unwrap();
        write_json(&index(&root).unwrap(), &tx).unwrap();
        let prepared = Prepared {
            transaction: tx.clone(),
            version: j.old_version.clone(),
            recovery: tx.join("helper/CleanSlate.exe"),
        };
        fs::write(tx.join("backup/unrelated.wav"), b"preserve").unwrap();
        // Initial authentication refuses unknown data before any cleanup acknowledgment.
        assert!(cleanup(&prepared).is_err());
        assert!(matching(&root, &j.new_hash));
        assert!(tx.join("backup/CleanSlate.exe").exists());
        fs::remove_file(tx.join("backup/unrelated.wav")).unwrap();
        fs::copy(
            tx.join("backup/package-manifest.json"),
            tx.join("cleanup-manifest.json"),
        )
        .unwrap();
        save(&tx, &mut j, State::CleanupAcknowledged).unwrap();
        fs::remove_file(tx.join("helper/CleanSlate.exe")).unwrap();
        fs::write(tx.join("helper/unrelated.wav"), b"preserve").unwrap();
        assert!(cleanup(&prepared).is_err());
        assert_eq!(
            fs::read(tx.join("helper/unrelated.wav")).unwrap(),
            b"preserve"
        );
        assert!(index(&root).unwrap().exists());
        fs::remove_file(tx.join("helper/unrelated.wav")).unwrap();
        cleanup(&prepared).unwrap();
        assert!(!index(&root).unwrap().exists());
        assert!(!tx.exists());
        assert!(matching(&root, &j.new_hash));
        fs::remove_dir_all(parent).unwrap();
    }
    #[test]
    fn recovery_refuses_unrelated_or_ambiguous_roots() {
        let dir =
            std::env::temp_dir().join(format!("cleanslate-recovery-reject-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("journal.json"), b"{}").unwrap();
        assert!(super::recover(&dir).is_err());
        assert!(dir.join("journal.json").exists());
        std::fs::remove_file(dir.join("journal.json")).unwrap();
        std::fs::remove_dir(dir).unwrap();
    }
}
