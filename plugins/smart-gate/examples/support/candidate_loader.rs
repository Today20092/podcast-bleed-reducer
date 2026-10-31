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

    pub fn configure(binary: &str) {
        assert!(
            module().is_null(),
            "DirectML was already loaded before Gate"
        );
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
                println!("CRT_PRELOADED: {name}: {}", module_path(handle).display());
            } else {
                println!("CRT_NOT_PRELOADED: {name}");
            }
        }
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
        let expected = std::fs::canonicalize(
            std::path::Path::new(binary)
                .parent()
                .unwrap()
                .join("DirectML.dll"),
        )
        .unwrap();
        assert_eq!(
            loaded, expected,
            "DirectML must resolve from the candidate sidecar"
        );
        println!("DIRECTML_PATH: {}", loaded.display());
    }
}

#[cfg(windows)]
pub use windows::{configure, verify};

#[cfg(not(windows))]
pub fn configure(_: &str) {
    panic!("Candidate loader verification requires Windows");
}
#[cfg(not(windows))]
pub fn verify(_: &str) {
    panic!("Candidate loader verification requires Windows");
}
