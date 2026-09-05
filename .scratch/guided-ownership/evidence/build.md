# Guided ownership test build

- Branch: `codex/cleanslate-ownership`.
- Built source commit: `a2de197d55d9b37fcdb0e788cccfa62f054988c3`. Later delivery-log changes are documentation only.
- Command: `cargo build --release --locked --bins --target-dir C:/Users/User/.codex/worktrees/dd95/podcast-automix-rust-gui/target`.
- EXE: `C:/Users/User/Documents/Codex/2026-08-14/podcast-automix-rust-gui/dist/CleanSlate-ownership-a2de197/CleanSlate.exe`.
- SHA-256: `5F0FD7128F43C4ECB782CEFF3E0DA599F4013E5465885CFE9BCFB109C5CA722D`.
- Packaged EXE hash matches the compiled `rust-app.exe`. No application launch or listening test was performed.

82 automated tests passed, five ignored; formatting, strict all-target Clippy, locked release build and independent final reviews passed. See `integration-checks.md`, `quality-results.md` and `performance-results.md` for scope and limits. This is a test build: the circled A02/A03 bleed remains unresolved, as tracked in issue 05.
