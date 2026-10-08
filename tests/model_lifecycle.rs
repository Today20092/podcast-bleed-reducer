use rust_app::model::{ModelCache, ModelIdentity};
use std::io::Cursor;
use std::sync::atomic::AtomicBool;

#[test]
fn failed_native_helper_returns_actionable_error_and_retains_installed_model() {
    let directory =
        std::env::temp_dir().join(format!("cleanslate-helper-model-{}", std::process::id()));
    let cache = ModelCache::new(directory.clone());
    let identity = ModelIdentity {
        file_name: "fixture.gguf",
        bytes: 3,
        sha256: "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
    };
    cache
        .install(
            &identity,
            Cursor::new(b"abc"),
            &AtomicBool::new(false),
            |_| {},
        )
        .unwrap();
    let mut helper = std::process::Command::new(std::env::current_exe().unwrap());
    helper
        .args([
            "--ignored",
            "--exact",
            "native_abort_fixture",
            "--nocapture",
        ])
        .env("CLEANSLATE_NATIVE_ABORT_FIXTURE", "1");
    let error =
        rust_app::model::verify_native_runtime(helper, &AtomicBool::new(false)).unwrap_err();
    assert!(error.contains("Native model verification process failed"));
    assert!(error.contains("installed model and project results retained"));
    assert!(cache.verified_path(&identity).is_ok());
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
#[ignore = "fixture invoked only as an owned subprocess by the verification test"]
fn native_abort_fixture() {
    if std::env::var_os("CLEANSLATE_NATIVE_ABORT_FIXTURE").is_some() {
        std::process::abort();
    }
}

#[test]
fn cancelling_native_verification_terminates_the_owned_helper() {
    use std::time::{Duration, Instant};
    let cancel = std::sync::Arc::new(AtomicBool::new(false));
    let cancelling = cancel.clone();
    let cancellation = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(200));
        cancelling.store(true, std::sync::atomic::Ordering::Relaxed);
    });
    let mut helper = std::process::Command::new(std::env::current_exe().unwrap());
    helper
        .args(["--ignored", "--exact", "native_wait_fixture", "--nocapture"])
        .env("CLEANSLATE_NATIVE_WAIT_FIXTURE", "1");
    let start = Instant::now();
    let error = rust_app::model::verify_native_runtime(helper, &cancel).unwrap_err();
    cancellation.join().unwrap();
    assert!(error.contains("cancelled"));
    assert!(start.elapsed() < Duration::from_secs(5));
}

#[test]
#[ignore = "fixture invoked only as an owned subprocess by the cancellation test"]
fn native_wait_fixture() {
    if std::env::var_os("CLEANSLATE_NATIVE_WAIT_FIXTURE").is_some() {
        std::thread::sleep(std::time::Duration::from_secs(30));
    }
}

#[test]
fn missing_native_runtime_is_actionable_and_never_falls_back() {
    let error =
        rust_app::model::NativeRuntime::open(std::path::Path::new("nonexistent-native-runtime"))
            .err()
            .unwrap();
    assert!(error.contains("native runtime"));
}

#[test]
fn incompatible_or_corrupt_native_package_is_rejected_without_fallback() {
    let directory =
        std::env::temp_dir().join(format!("cleanslate-invalid-native-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let library = directory.join("nemo_speech_asr_c.dll");
    std::fs::write(&library, b"abc").unwrap();
    let manifest = rust_app::model::RuntimeProvenance {
        revision: rust_app::model::RUNTIME_REVISION.into(),
        recipe_sha256: "0".repeat(64),
        binaries: std::collections::BTreeMap::from([(
            "nemo_speech_asr_c.dll".into(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".into(),
        )]),
    };
    std::fs::write(
        directory.join("runtime-manifest.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    let error = rust_app::model::NativeRuntime::open(&directory)
        .err()
        .unwrap();
    assert!(error.contains("Cannot load required native runtime"));
    std::fs::write(&library, b"bad").unwrap();
    let error = rust_app::model::NativeRuntime::open(&directory)
        .err()
        .unwrap();
    assert!(error.contains("integrity"));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn explicit_acquisition_reports_progress_and_retains_good_model_on_bad_bytes() {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    let directory =
        std::env::temp_dir().join(format!("cleanslate-http-model-{}", std::process::id()));
    let cache = ModelCache::new(directory.clone());
    let identity = ModelIdentity {
        file_name: "fixture.gguf",
        bytes: 3,
        sha256: "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
    };
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/model", listener.local_addr().unwrap());
    let server = std::thread::spawn(move || {
        for body in [b"abc", b"bad"] {
            let (mut socket, _) = listener.accept().unwrap();
            let mut request = [0; 2048];
            let _ = socket.read(&mut request).unwrap();
            socket
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\nConnection: close\r\n\r\n")
                .unwrap();
            socket.write_all(body).unwrap();
        }
    });
    let mut transferred = 0;
    cache
        .download_from(&identity, &url, &AtomicBool::new(false), |n| {
            transferred = n
        })
        .unwrap();
    assert_eq!(transferred, 3);
    assert!(
        cache
            .download_from(&identity, &url, &AtomicBool::new(false), |_| {})
            .is_err()
    );
    assert!(cache.verified_path(&identity).is_ok());
    server.join().unwrap();
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn verified_download_is_available_on_offline_relaunch() {
    let directory = std::env::temp_dir().join(format!("cleanslate-model-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let cache = ModelCache::new(directory.clone());
    let identity = ModelIdentity {
        file_name: "fixture.gguf",
        bytes: 3,
        sha256: "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
    };
    cache
        .install(
            &identity,
            Cursor::new(b"abc"),
            &AtomicBool::new(false),
            |_| {},
        )
        .unwrap();
    assert_eq!(
        ModelCache::new(directory.clone())
            .verified_path(&identity)
            .unwrap(),
        directory.join("fixture.gguf")
    );
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn cancellation_and_partial_download_keep_the_previous_verified_install() {
    use std::sync::atomic::Ordering;
    let directory =
        std::env::temp_dir().join(format!("cleanslate-cancel-model-{}", std::process::id()));
    let cache = ModelCache::new(directory.clone());
    let identity = ModelIdentity {
        file_name: "fixture.gguf",
        bytes: 3,
        sha256: "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
    };
    let cancelled = AtomicBool::new(false);
    cache
        .install(&identity, Cursor::new(b"abc"), &cancelled, |_| {})
        .unwrap();
    let result = cache.install(&identity, Cursor::new(b"abc"), &cancelled, |_| {
        cancelled.store(true, Ordering::Relaxed)
    });
    assert!(result.unwrap_err().contains("cancelled"));
    assert!(!directory.join("fixture.gguf.download").exists());
    assert!(
        cache
            .install(
                &identity,
                Cursor::new(b"ab"),
                &AtomicBool::new(false),
                |_| {}
            )
            .is_err()
    );
    assert!(cache.verified_path(&identity).is_ok());
    std::fs::write(directory.join("fixture.gguf"), b"bad").unwrap();
    assert!(
        cache
            .verified_path(&identity)
            .unwrap_err()
            .contains("integrity")
    );
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn cancellation_while_the_network_stalls_returns_without_installing() {
    use std::net::TcpListener;
    use std::time::{Duration, Instant};
    let directory =
        std::env::temp_dir().join(format!("cleanslate-stall-model-{}", std::process::id()));
    let cache = ModelCache::new(directory.clone());
    let identity = ModelIdentity {
        file_name: "fixture.gguf",
        bytes: 3,
        sha256: "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
    };
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/model", listener.local_addr().unwrap());
    let cancel = std::sync::Arc::new(AtomicBool::new(false));
    let cancelling = cancel.clone();
    let server = std::thread::spawn(move || {
        let (_socket, _) = listener.accept().unwrap();
        std::thread::sleep(Duration::from_millis(100));
        cancelling.store(true, std::sync::atomic::Ordering::Relaxed);
        std::thread::sleep(Duration::from_millis(300));
    });
    let start = Instant::now();
    assert!(
        cache
            .download_from(&identity, &url, &cancel, |_| {})
            .is_err()
    );
    assert!(start.elapsed() < Duration::from_secs(2));
    assert!(!directory.join("fixture.gguf").exists());
    assert!(!directory.join("fixture.gguf.download").exists());
    server.join().unwrap();
    std::fs::remove_dir_all(directory).unwrap();
}
