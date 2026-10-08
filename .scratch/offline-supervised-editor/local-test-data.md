# Owner-supplied local test audio

On 2026-10-08 the owner authorized using and copying `G:\261005\261005-T001.WAV` for testing. The primary checkout stores the original and three mono derivatives in its Git-ignored `test-audio/` directory. Worktrees do not receive this directory automatically; reference the primary checkout's absolute path. Do not commit or publish recordings.

Primary checkout: `C:\Users\User\Documents\Codex\2026-08-14\podcast-automix-rust-gui`.

- Original: `test-audio/261005-T001.WAV`, verified SHA-256 `7B255B9D8D47B79735210B9BAE3AC59AED5D4CCEE3AAF1A54E85024586CFF016`.
- Mono inputs: `test-audio/261005-channel-1.wav`, `261005-channel-2.wav`, and `261005-channel-3.wav`.
- Each channel: 48 kHz, 32-bit IEEE float, 154,128,376 frames, 3,211.007833333 seconds (53 minutes 31 seconds).
- The owner reports synchronization. Every sample of each mono derivative was compared with the original interleaved channel and matched bit-for-bit; no resampling or mixing occurred.
- Original metadata: BEXT time reference 98,158,861 samples; iXML interleave 1/2/3 corresponds to recorder channels 7/8/9 named Tr5/Tr6/Tr7. The mono derivatives do not carry original recorder metadata; use supplied-alignment declaration, and retain the original for metadata tests.

Speaker identities and reference/independent evaluation labels are pending. Do not infer identities from recorder channel names. This recording can support workflow tests now; quality acceptance requires explicitly separated training/reference and independent evaluation material. It does not cover the eight-track/two-hour release gate by itself.
