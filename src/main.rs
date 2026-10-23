#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

fn main() -> iced::Result {
    if let Some(code) = rust_app::updater::cli() {
        std::process::exit(code);
    }
    let args: Vec<_> = std::env::args_os().collect();
    #[cfg(all(feature = "updater-test-key", debug_assertions))]
    if args.iter().any(|a| a == "--update-health")
        && std::env::var("CLEANSLATE_UPDATE_FIXTURE_FAILURE").as_deref() == Ok("gui-crash")
    {
        std::process::exit(74);
    }
    if args.iter().any(|arg| arg == "--package-self-check") {
        #[cfg(all(feature = "updater-test-key", debug_assertions))]
        if std::env::var("CLEANSLATE_UPDATE_FIXTURE_FAILURE").as_deref() == Ok("self-check") {
            std::process::exit(75);
        }
        let result = rust_app::package_self_check();
        let report = match &result {
            Ok(()) => "PASS: offline model inference, analysis, project save/reopen, render and cancellation.\n".to_string(),
            Err(error) => format!("FAIL: {error}\n"),
        };
        if let Some(index) = args.iter().position(|arg| arg == "--report") {
            let path = args.get(index + 1).expect("--report requires a path");
            std::fs::write(path, &report).expect("Cannot write package check report");
        }
        print!("{report}");
        std::process::exit(if result.is_ok() { 0 } else { 1 });
    }
    let _installation = match rust_app::updater::InstallationLock::startup() {
        Ok(lock) => lock,
        Err(error) => {
            rfd::MessageDialog::new()
                .set_title("CleanSlate recovery")
                .set_description(&error)
                .set_level(rfd::MessageLevel::Error)
                .show();
            return Ok(());
        }
    };
    rust_app::app::run()
}
