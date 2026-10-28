# Podcast 103: allocation observability in pinned sources

2026-10-27. Read-only inspection; no build, profiler, dependency/configuration change or repository edit.

**Finding:** no existing public hook meets the requested combination of GUI allocation-call **and byte** measurement, the current single allocator, unchanged shipping behavior, no private API and no new dependency. Keep allocation counts/bytes explicitly unavailable in 103. Mesh capacities measure retained geometry storage, not allocation traffic or process memory.

## Installed evidence

Registry root `R` is `C:/Users/User/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`. Podcast pins nice-plug 0.4.2 and enables `assert_process_allocs`; its lock resolves nice-assert-no-alloc 1.0.0.

- [nice-plug wrapper/util.rs:30](C:/Users/User/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nice-plug-0.4.2/src/wrapper/util.rs:30) registers private global allocator `A: nice_assert_no_alloc::AllocDisabler` only under `debug_assertions && assert_process_allocs`. `process_wrapper` at line 195 invokes `assert_no_alloc` in that configuration. [wrapper.rs:6](C:/Users/User/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nice-plug-0.4.2/src/wrapper.rs:6) makes this utility module crate-private.
- Public `nice_plug::util::permit_alloc` is re-exported from [nice-plug-core util.rs:30](C:/Users/User/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nice-plug-core-0.4.2/src/util.rs:30). It permits allocations within a scope; it exposes no observer, count or byte statistics.
- [nice-assert-no-alloc lib.rs:130](C:/Users/User/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nice-assert-no-alloc-1.0.0/src/lib.rs:130) does have public `nice_assert_no_alloc::violation_count()` and `reset_violation_count()` (line 141), **only** with the applicable `warn_debug`/`warn_release` feature. These are thread-local forbidden-allocator-check counts, incremented only while forbid nesting exceeds permit nesting. They include deallocation checks, expose no bytes or allocation/deallocation breakdown, and are not ordinary unguarded GUI-allocation counters.
- `AllocDisabler::check` is private (line 160); its GlobalAlloc implementation at lines 203–213 checks then forwards `alloc`/`dealloc` to `System`. There is no public callback or accounting state. Nice-plug/core request backtrace/log features, not warning mode, and do not re-export these counters. Direct access would require a new direct dependency/feature arrangement. Enabling warning mode also changes forbidden-call handling from allocation-error termination to warning/count behavior for that build; it is not a transparent measurement switch.

Source SHA256: `nice-plug-0.4.2/src/wrapper/util.rs` = `f8bcf9759c2197fa1d910ecbddc106667c3e3667643b1d405f36106b7db34cec`; `nice-assert-no-alloc-1.0.0/src/lib.rs` = `c2999a75a9a38880f2489a0d8daf27d1d33aa9081fe460e0dcbd4191b947a99f`. Conclusions concern these installed sources, not latest upstream APIs. No documentation lookup was needed to inspect this exact implementation.

## Minimum future measurement boundary

If exact call/byte traffic becomes necessary, separately approve opt-in, per-thread accounting in the **existing allocator implementation**, with public snapshot/reset access and unchanged forbid/permit behavior. Count alloc, zeroed alloc, realloc and dealloc explicitly; define requested bytes versus live/peak bytes and avoid allocation/logging inside accounting. Validate using known allocation/reallocation/drop controls before measuring warmed GUI scopes. This requires an upstream hook or maintained dependency patch and is outside current constraints.

An alternative is a separate benchmark-only build without nice-plug's allocator registration and with one counting allocator. It changes the measurement build's allocation-guard configuration and is not equivalent to the guarded shipping/debug artifact; it needs an explicit separate scope. Neither approach captures native DLL/driver allocations or other threads automatically. No second global allocator or private TLS access is recommended.
