# Install and verify the required Nemotron model and native runtime

Status: ready-for-agent
Blocked by: 01
Spec: ../spec.md

## Scope

Pin a NeMo-Speech.cpp revision, supported Nemotron V3 GGUF revision and expected model hash. Bundle the Windows native runtime/dependencies. Add an explicit first-use model download with progress/cancellation and actual runtime/model license notices. The runtime is Apache-2.0; weights use OpenMDW 1.1. Verify pinned redistribution/acquisition obligations and source URLs.

Download to a temporary file, verify identity/integrity, then atomically install in the local model cache. A failed/cancelled download cannot replace a good model. After installation, model loading and inference work offline without Python, keys or a GPU requirement. Do not auto-update model versions or silently fall back to an optional detector when the required model is absent.

## Reach

New narrow model lifecycle/native deployment responsibility, src/app.rs first-use state, project analysis provenance, packaging/build scripts and Cargo integration. Follow current pinned SDK/build documentation in feasibility notes.

## Acceptance

- A clean Windows machine with no Python can install the runtime, download once and load the verified model.
- Network-disabled relaunch loads the installed model; no startup/background download is attempted.
- Missing/corrupt model, bad checksum, partial download, incompatible binary and unavailable network produce actionable errors with existing results intact.
- Cancellation leaves no installed incomplete model; notices distinguish native-code and weights licenses.
- The exact dependency/model recipe is reproducible in development and release packages.
