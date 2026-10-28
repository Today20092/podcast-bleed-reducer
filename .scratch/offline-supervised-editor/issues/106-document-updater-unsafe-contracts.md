# 106: Document Windows updater unsafe contracts

Status: done
Assignee: Codex rust_ticket106
Readiness: ready-for-agent
Type: task
Blocked by: None (can start immediately)

## What to build

Maintainers can review each updater Windows FFI boundary using an explicit safety contract and technical justification, without changing updater behavior.

## Specification

[Accepted Google Rust audit corrections](../../offline-supervised-migration/rust-guidelines-corrections.md).

## Acceptance criteria

- [x] Every production updater unsafe boundary has a concise SAFETY contract describing its actual pointer/buffer initialization, capacity/lifetime, NUL termination and returned length/result obligations.
- [x] Parent-process identity ordering, handle validity/lifetime and exactly-once closure are documented against the actual control flow and pinned Windows API contracts.
- [x] The technical reason for the unsafe platform operation is clear; no undefined behavior is claimed or behavioral rewrite introduced without evidence.
- [x] Documentation/source consistency, links and the scoped diff are verified; no API, dependency, runtime or serialized behavior changes are included.

## Results

Completed 2026-10-27 on scoped branch `codex/rust-audit-106`, based on integration commit `5a321767b09cd42eb836c996979da5bea05b714c`. Added technical rationale and explicit SAFETY contracts at the two production updater Windows FFI blocks. Buffer capacities/lifetimes, initialization, string termination and success-before-output use are described. The parent-process block documents required access rights, query length semantics, identity-before-ready ordering, waiting with a live private handle and one close on each success/Result-error path. It deliberately does not claim panic-unwind cleanup that the existing code does not provide.

Verification: graph-first caller/source review (the graph overwrites cfg-duplicate symbols, so the Windows source was read directly); signatures checked against the locked `windows-sys` 0.61.2 source; official Microsoft contracts consulted below. A normalized before/after comparison, excluding full-line comments, confirmed identical transaction source at the implementation boundary. `git diff --check` passed; the relative specification link resolves. Only this ticket and source comments changed. No API, dependency, runtime, state or serialized behavior changed.

Context7 resolution was attempted first but its monthly quota was exhausted. Official fallback sources: [GetVolumePathNameW](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getvolumepathnamew), [GetDriveTypeW](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getdrivetypew), [GetVolumeInformationW](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getvolumeinformationw), [GetDiskFreeSpaceExW](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getdiskfreespaceexw), [OpenProcess](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-openprocess), [QueryFullProcessImageNameW](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-queryfullprocessimagenamew), [WaitForSingleObject](https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-waitforsingleobject), and [CloseHandle](https://learn.microsoft.com/en-us/windows/win32/api/handleapi/nf-handleapi-closehandle).

This documentation-only scope introduces no behavior needing a RED/GREEN test. No compiler, formatter, test suite, native updater/host or GUI was launched; full-batch verification remains with the integration coordinator under the existing resource guard.
