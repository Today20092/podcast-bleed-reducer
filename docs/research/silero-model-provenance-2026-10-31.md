# Bundled Silero version: exact v6.2 match

Verified2026-10-31 with official upstream Git refs/tree metadata and a direct
commit-pinned raw model download, hashed in memory. No model was installed,
replaced, upgraded, executed or rebuilt.

**Yes: the bundled asset matches Silero VAD v6.2 byte for byte.** The repository's
`assets/README.md` and several research/recipe strings calling it `v5` are
incorrect metadata. The initial architecture audit repeated that label; its
version wording is now corrected. ORT2.0.0-rc.10 is a separate runtime version
and was not used to infer the model version.

The [official v6.2 tag](https://github.com/snakers4/silero-vad/releases/tag/v6.2)
resolves to commit `be95df9152c0d7618fa1edfeb296fc3dae32376f`.
The exact artifact is
[`src/silero_vad/data/silero_vad.onnx`](https://github.com/snakers4/silero-vad/blob/be95df9152c0d7618fa1edfeb296fc3dae32376f/src/silero_vad/data/silero_vad.onnx).
Its official Git blob is `80c5592ef1f4c9ede3e357bbd02eb863358a6a9d`, size
2,327,524bytes. A bounded4MB-buffer download from the
[commit-pinned raw URL](https://raw.githubusercontent.com/snakers4/silero-vad/be95df9152c0d7618fa1edfeb296fc3dae32376f/src/silero_vad/data/silero_vad.onnx)
has SHA256:

```text
1a153a22f4509e292a94e67d6f9b85e8deb25b4988682b7e174c65279d8788e3
```

That is the same SHA256 measured from local `assets/silero_vad.onnx`.
`git ls-tree COMMIT assets/silero_vad.onnx` gives that same Git blob in all
three requested source snapshots:

| Product snapshot | Source commit | Model blob |
| --- | --- | --- |
| Desktop development | `a2606de537ad99e1b36d6cd7748d131e9d5d5485` | `80c5592ef1f4c9ede3e357bbd02eb863358a6a9d` |
| Delivered Gate source | `2e3adbf12dd9c8f73105c4b99af7a81e3cf699c7` | same |
| GUI4afa Gate source | `4afa18a371cd811f8f91a4504c1a612ae38e3af0` | same |

Desktop embeds that file in `src/vad.rs`; Gate embeds it from
`plugins/gate-detector/src/lib.rs`. Gate's `MODEL_SHA256` already records the
matching checksum even though its recipe string says `silero-v5`.
This is source/model-asset provenance, not a new binary rebuild or claim that
every installed Desktop executable was independently extracted in this audit.
Delivered Gate source identity comes from the existing verified build receipt.

For contrast, the same upstream model path has different blobs at earlier
official tags, despite the same2,327,524byte length:

| Tag | Tag commit | Model blob |
| --- | --- | --- |
| v5.1 | `84768cefdf5a3852400e9d8237f7315d14b64a08` | `b3e3a900c0d70e67b5e2b90a33ad856ee7947930` |
| v6.0 | `fba061dc5559f696e62171e9a0741782b0fdc23c` | `cb605195e6975825c5fb941ad97e2e27f636d25a` |
| v6.1 | `3f0c9ead5490e20f6a2ce7e9d16a53af9e0e5818` | `cb605195e6975825c5fb941ad97e2e27f636d25a` |
| v6.2 | `be95df9152c0d7618fa1edfeb296fc3dae32376f` | `80c5592ef1f4c9ede3e357bbd02eb863358a6a9d` |

Metadata was fetched from official
`api.github.com/repos/snakers4/silero-vad/git/ref/tags/TAG` and
`git/trees/TAG?recursive=1`. The direct SHA256 comparison makes the conclusion
independent of filename, byte length and earlier handwritten version labels.
This does not establish that a newer package wrapper/alternate ONNX variant is
used, or imply better preservation than our measured fixtures demonstrate.
