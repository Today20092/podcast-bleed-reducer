//! Dedicated-process loader check; this is not a DAW deployment strategy.
#[cfg(windows)]
mod windows {
    use std::{ffi::c_void, os::windows::ffi::OsStrExt, path::PathBuf};

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn SetDefaultDllDirectories(flags: u32) -> i32;
        fn AddDllDirectory(path: *const u16) -> *mut c_void;
        fn GetModuleHandleW(name: *const u16) -> *mut c_void;
        fn GetModuleFileNameW(module: *mut c_void, path: *mut u16, size: u32) -> u32;
    }

    fn module() -> *mut c_void {
        let name: Vec<_> = "DirectML.dll\0".encode_utf16().collect();
        // SAFETY: name is a live, NUL-terminated UTF-16 string.
        unsafe { GetModuleHandleW(name.as_ptr()) }
    }

    fn report_crt(phase: &str) {
        for name in [
            "MSVCP140.dll",
            "MSVCP140_1.dll",
            "VCRUNTIME140.dll",
            "VCRUNTIME140_1.dll",
        ] {
            let wide: Vec<_> = name.encode_utf16().chain(Some(0)).collect();
            // SAFETY: wide is a live, NUL-terminated module name.
            let handle = unsafe { GetModuleHandleW(wide.as_ptr()) };
            if !handle.is_null() {
                println!(
                    "CRT_{phase}_PATH: {name}: {}",
                    module_path(handle).display()
                );
            } else {
                println!("CRT_{phase}_ABSENT: {name}");
            }
        }
    }

    pub fn configure(binary: &str) {
        assert!(
            module().is_null(),
            "DirectML was already loaded before Gate"
        );
        report_crt("BEFORE_GATE");
        let binary = std::fs::canonicalize(binary).unwrap();
        let mut directory: Vec<_> = binary.parent().unwrap().as_os_str().encode_wide().collect();
        directory.push(0);
        // SAFETY: this dedicated single-threaded host configures its process search
        // before loading any plugin; the absolute directory string is terminated.
        unsafe {
            // SYSTEM32 | USER_DIRS excludes application directory, CWD and PATH.
            assert_ne!(SetDefaultDllDirectories(0x800 | 0x400), 0);
            assert!(!AddDllDirectory(directory.as_ptr()).is_null());
        }
    }

    fn module_path(module: *mut c_void) -> PathBuf {
        let mut path = vec![0u16; 32768];
        // SAFETY: the loaded module handle is valid and path has the stated capacity.
        let length = unsafe { GetModuleFileNameW(module, path.as_mut_ptr(), path.len() as u32) };
        assert!(length > 0 && (length as usize) < path.len());
        std::fs::canonicalize(PathBuf::from(
            String::from_utf16(&path[..length as usize]).unwrap(),
        ))
        .unwrap()
    }

    pub fn verify(binary: &str) {
        let module = module();
        assert!(!module.is_null(), "Gate activation did not load DirectML");
        let loaded = module_path(module);
        println!("DIRECTML_PATH: {}", loaded.display());
        report_crt("AFTER_GATE");
        let binary = std::fs::canonicalize(binary).unwrap();
        let expected = binary.parent().unwrap().join("DirectML.dll");
        if loaded != expected {
            eprintln!(
                "LOADER_RUNTIME_PATH_REJECTED: loaded {}; expected {}",
                loaded.display(),
                expected.display()
            );
            std::process::exit(42);
        }
    }
}

#[cfg(windows)]
pub use windows::{configure, verify};

/// Loads the trusted engineering candidate, classifying only Windows error 126.
///
/// # Safety
/// The caller must satisfy the plugin library's initialization requirements.
pub unsafe fn load(path: &str, isolated: bool) -> libloading::Library {
    // SAFETY: the ABI host caller owns the candidate's initialization contract.
    match unsafe { libloading::Library::new(path) } {
        Ok(library) => library,
        Err(error) => {
            #[cfg(windows)]
            if isolated
                && matches!(error, libloading::Error::LoadLibraryExW { .. })
                && std::error::Error::source(&error)
                    .and_then(|source| source.downcast_ref::<std::io::Error>())
                    .and_then(std::io::Error::raw_os_error)
                    == Some(126)
            {
                eprintln!("LOADER_DEPENDENCY_MISSING: Windows error 126");
                std::process::exit(43);
            }
            let _ = isolated;
            panic!("Cannot load plugin: {error}");
        }
    }
}

#[cfg(not(windows))]
pub fn configure(_: &str) {
    panic!("Candidate loader verification requires Windows");
}
#[cfg(not(windows))]
pub fn verify(_: &str) {
    panic!("Candidate loader verification requires Windows");
}
