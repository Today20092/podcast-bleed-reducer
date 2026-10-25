# Auto-Align Post 2.3.4: evidence and CleanSlate opportunities

Date: 2026-10-22. Owner-requested research, tracked in [ticket 70](../../.scratch/offline-supervised-editor/issues/70-auto-align-post-research.md). This report does not authorize DSP changes or extend the accepted export contract.

## Practical conclusion

Auto-Align Post addresses microphones recording common material with different arrival times and frequency-dependent phase. CleanSlate instead estimates which microphone's assigned speaker is present and applies attenuation envelopes. The most useful independent experiment is **better local delay tracking inside analysis**, coupled with rejection of unreliable matches. Applying alignment or spectral phase correction to exported audio would be a separate product change.

We have not recovered the proprietary estimator, filter-design equations, or a complete DSP call chain. Embedded help and option names establish intended features, not execution or numerical implementation. The exact algorithms remain unresolved.

## Artifact identity and scope

The installed target is `C:/Program Files/Common Files/VST3/AutoAlignPost.vst3/Contents/x86_64-win/AutoAlignPost.vst3`, an x86-64 PE DLL. Its file/product version is 2.3.4, size 44,436,312 bytes, SHA-256 `1b3c52650e4e307f5c9b61a2d75c9bcee464d8d292f45553504b3774ee621c5f`.

The supplied installer is `C:/Users/User/Downloads/Auto-Align Post 2.3.4 Setup.exe`, size 178,963,592 bytes, product version 2.3.4, SHA-256 `12b468c3b54a58427f63280d858bdb0ea8cc625458f4142e7056e327aa395752`. It was identified without launching it. Matching product versions do not prove that the installed DLL was extracted from those installer bytes.

REA 6.7.0 opened the installed DLL with Ghidra 12.1.4. REA independently recorded the same target hash. The analysis profile digest is `189821c22f9a3c65d594fc3eda7b5a03806941e5d6504c95c1b268b890a7c7ff`. Ghidra uses a private ephemeral import and does not modify the installed plugin. No plugin/DAW execution, audio rendering, licensing modification, or installer execution was performed.

## Documented behavior

The [vendor product page](https://www.soundradix.com/products/auto-align-post/) describes dynamic time alignment for moving microphones, a static mode, spectral phase correction, and a spectral correlation meter. It advertises a ±100 ms alignment range and VST3 integration through ARA2. Its high-pass-filter example explains why matching arrival times alone may leave frequency-dependent phase differences. These are vendor claims, not measurements from this investigation.

The [release history](https://www.soundradix.com/products/auto-align-post/release-notes/) records repeated improvements for silent reference audio, including 2.3.4. Earlier entries mention failures involving no shared content and spectral-filter correlation. This supports testing those adverse conditions; it does not expose the corresponding safeguards or estimator.

The linked current manual could not be retrieved through the web tool because its PDF exceeds that tool's response-size limit. Search snippets from older manuals were not treated as a complete current manual.

## Static evidence ledger

`PE-1`: REA `inspect_pe_resources`, Evidence ID `ev_34010062c5241b3597b81daa115d178b5a5f0ada560a0e38e7ef67c037d813d4`, observes PE32+, machine 34404 (AMD64), ten resources and complete traversal of 24 resource entries. Version resource bytes occupy file offset 34,710,656 for 496 bytes. Resources are opaque except icon-group relationships; they do not establish DSP behavior.

`RAW-1`: supplementary read-only byte inspection of the exact hashed DLL, with ISO-8859-1 decoding to preserve one character per byte and printable-ASCII matching. Offsets below are decimal file offsets, not runtime addresses. They are reproducible observations, not REA Evidence IDs.

| Offset | Observation | What it establishes |
| --- | --- | --- |
| 29,258,024 | Phase Alignment Mode | Separate phase-mode UI text exists. |
| 29,932,608 | `--no-allpass` | An option-name literal exists; reachability and effect are unknown. |
| 32,343,600 | Keep the same alignment over time. | Embedded static-mode help. |
| 32,343,640 | Change alignment over time to best match conditions. | Embedded dynamic-mode help. |
| 32,343,696 and 32,343,733 | Help describes applying all-pass filters for frequency-wide phase matching. | Intended spectral-mode mechanism; no recovered coefficients or topology. |
| 29,319,696 | Message about skipped alignment for invalid reference/target samples. | An error-message literal exists; validation coverage is unverified. |
| 29,318,256 | Auto-Align Post requires ARA2. | Host-integration message; a plain streaming VST3 interface cannot be assumed. |

An initial broad ASCII decoding pass replaced non-ASCII bytes and produced noise. Those incidental matches were discarded; the ledger uses the corrected byte-preserving pass only. Generic graphics filter names were also excluded from DSP conclusions.

The DLL also contains a PDB basename and Eden wrapper build-path strings. Those do not provide the PDB, prove which regions are protected, or justify treating wrapper code as audio processing. A string about unimplemented fractional sampling cannot establish whether the active DSP supports fractional delays.

## What CleanSlate already has

### Algorithm ideas, without attributing an implementation

A pure sample delay has frequency response `exp(-j * omega * d)`: its phase changes linearly with frequency. Matching one arrival-time offset can therefore correct travel delay while leaving phase differences introduced by microphones, filters and room paths.

An ideal all-pass filter changes phase while retaining unit magnitude. For example, the generic first-order form `(a + z^-1) / (1 + a*z^-1)` is stable for real `abs(a) < 1`. This is standard DSP background, **not recovered Auto-Align code**. The binary's all-pass help does not tell us the order, number of stages, optimization target or coefficient-update policy.

For ownership analysis, the useful question is how well one microphone explains delayed common content in another. Improving that estimate can strengthen evidence for bleed. Aligning two waveforms by itself cannot separate two people talking simultaneously, and processing only their summed signal cannot selectively attenuate one voice with a single gain envelope.

### Existing implementation

Repository discovery used the code graph and focused source inspection. These observations describe CleanSlate, not recovered Sound Radix code.

- [JointFacts::prepare_blocks](../../src/joint_detector.rs) filters and decimates analysis audio to 8 kHz, chooses one coarse lag per 256 ms block and refines around it for 32 ms frames using up to 96 ms context. Its comment identifies faster microphone movement as a limitation of the one-lag scheme.
- `correlation` in that module computes squared normalized correlation, accepting inverted-polarity copies as similar. Simply adding polarity-insensitive correlation would duplicate current behavior.
- [AcousticEvidenceCache::analyze](../../src/acoustics.rs) learns directed speaker-to-other-microphone relationships from trusted solo references, with room-tone, energy and consistency checks.
- [qualify and residual_frames](../../src/acoustic_transfer.rs) use 16 kHz analysis copies, a ±192-sample (±12 ms) lag search and a regularized two-tap transfer fit. FIR selection requires held-out improvement from disjoint trusted reference segments. The resulting residuals are features, not replacement delivered audio.
- [SessionOwnershipCache](../../src/session_ownership.rs) integrates acoustic features and speech presence into independent per-track ownership estimates. Overlap can remain owned on more than one track.

## Independent experiments worth considering

| Priority | Experiment | Why it could help | Required evidence before adoption |
| --- | --- | --- | --- |
| 1 | Track multiple plausible local delays or use shorter reliable analysis windows, with continuity checks. | A head turn or changed mic position can invalidate the block's single lag. | Synthetic known-delay movement plus held-out real speech, showing fewer ownership errors without damaging quiet overlap. |
| 2 | Measure peak ambiguity and abstain when shared evidence is weak. | Silence, periodic signals, noise and independent simultaneous voices can produce misleading best matches. | Explicit silent-reference, unrelated-voice, repeated-peak and low-energy cases; verify preservation fallback. |
| 3 | Evaluate regional acoustic transfer models using trusted solo references. | A single directed-pair lag/two-tap fit may age poorly as positions change. | Disjoint validation, adequate references per region and conservative fallback where evidence is insufficient. |
| Later | Evaluate frequency-dependent coherence or a richer transfer feature. | A pure delay plus two taps may not describe filtered/reverberant bleed well. | Demonstrated improvement over existing features, controlled complexity and no source-audio replacement. |

These are independent design hypotheses. We did not establish that Auto-Align uses cross-correlation, GCC-PHAT, a particular FFT/window, dynamic programming, a particular all-pass order, or any specific confidence formula. Adding any of those now would be a choice requiring its own evaluation.

Use the frozen detector baseline `6a60cc2` when the future experiment's acceptance requires it, and also compare against the current implementation so an apparent gain does not merely rediscover work already shipped. Keep numerical preservation/error results separate from listening acceptance.

## Product boundary

The [accepted specification](../../.scratch/offline-supervised-migration/spec.md) defers automatic alignment/stretching/drift checking and requires native-rate source audio multiplied by the effective envelope, preserving timing and frame counts. [ADR-0006](../adr/0006-offline-supervised-editable-envelopes.md) retains human truth, independent overlap and protection for missing evidence. The [synchronization research](synchronization-and-drift.md) distinguishes recorder clock drift from acoustic travel delay.

Accordingly, the initial experiment should change analysis features only. A changing correlation peak cannot by itself diagnose recorder drift. A single global reference is also insufficient to establish every speaker's directed bleed path in a podcast with several independent sources.

The installed ARA2 plugin can be considered a future comparison tool in a compatible host. Installation alone does not establish an embeddable SDK, redistribution rights, unattended processing access, or ordinary low-latency VST3 operation. No vendor code or binary component was added to CleanSlate.

## Investigation status

- Artifact identity: answered.
- Intended time/phase modes and all-pass help: answered at documentation/embedded-text level.
- Exact estimator, interpolation, smoothing and filter construction: unresolved.
- Fit with CleanSlate and useful independent experiments: answered as design hypotheses.
- Real-recording benefit, audio quality, performance and runtime paths: untested.

## Native-analysis outcome and tooling

The default Ghidra import did not complete before REA's 330,000 ms startup deadline (`provider_timeout`, run `b969c7a7-d864-4e92-aa51-47e615480675`). It returned no usable function inventory or decompilation. Partial provider logs are diagnostics, not a successful function analysis.

After the owner authorized improving the setup, a scoped CLI fallback used the existing Ghidra/JDK paths, a 900,000 ms deadline, and disabled Decompiler Switch Analysis, Stack and Windows x86 PE Exception Handling. This was a distinct reduced-coverage profile, digest `60cad537d9c29d790468947a31c7a550a1595a90e65d8a5035091c397a81c040`. It failed with `execution_failure`: `ERR_REA_WINDOWS_NATIVE`, Win32 1450, insufficient system resources while observing the private bridge endpoint (run `34a8df70-d710-46a8-83fa-f45c4a3199ff`). No successful native snapshot was produced. Temporary shell settings did not change Codex/REA registration or installed engines.

Supplementary byte-frequency measurement found `.text` entropy 8.0000 bits/byte to four decimals across 28,583,936 bytes; `.rdata` was 6.8748 and `.guard` 7.2273. High entropy and the wrapper markers are consistent with packing/protection, but do not establish encryption, a specific scheme, or which DSP regions are recoverable. A new disassembler alone cannot be assumed to resolve this.

The [REA setup guide](https://rea.tools/get-started/) and [native guide](https://rea.tools/guides/native/) confirm that one native provider is sufficient and Windows Ghidra support is experimental. Existing Ghidra 12.1.4 and a 64-bit JDK 21 were already available. No additional software was installed. A subsequent doctor check found Codex registration healthy; a fresh CLI shell needs its Ghidra installation path supplied explicitly, unlike the configured MCP process. After cleanup, no Java process remained and the MCP session reported closed with no cleanup pending.

Preserved records:

- [Complete returned PE-resource Evidence record](../../.scratch/auto-align-post-research/pe-resource-evidence.json), retaining the original Evidence ID, provider metadata, locations and limitations.
- [Native-attempt and entropy ledger](../../.scratch/auto-align-post-research/analysis-attempts.json), explicitly an analyst summary rather than a successful native Evidence record.

Six embedded-text offset assertions passed against the hashed DLL, and all local report links resolved. A separate read-only review verified the CleanSlate timing, lag, held-out-validation and preservation claims. No application build or audio test was needed for these documentation-only changes; no runtime quality benefit is claimed.

A deeper continuation would first need a native backend run that completes under Windows resource limits (or another already available supported host/provider). Only then should it follow references from the phase-mode help/options into processing functions and validate candidate equations against instructions, constants and controlled audio. Exact DSP recovery remains open; the high-level feature findings are not a substitute for that evidence.
