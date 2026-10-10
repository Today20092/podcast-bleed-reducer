#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

fn main() -> iced::Result {
    let args: Vec<_> = std::env::args_os().collect();
    if args.iter().any(|arg| arg == "--package-self-check") {
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
    rust_app::app::run()
}
