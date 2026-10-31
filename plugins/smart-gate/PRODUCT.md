# CleanSlate Smart Gate

Operate mode. A separate single-input mono/stereo microphone plugin, with CLAP
and VST3 adapters. Ticket 91 establishes identity and a safe pass-through
prototype; detector feasibility, qualified gating and the full interface belong
to 92, 93 and 94. No automatic volume leveling or paired Session capture.

The accepted visual authority is [the plugin UI contract](../../docs/design/plugin-ui-contract.md).
The last checked native candidate remains pass-through. Current ticket 93 source
adds explicit temporary Arm / Disarm alongside saved Bypass; arming never clears
bypass and lifecycle changes retire old consent/gestures. Describe experimental
processing and actual preparation availability without claiming that consent
proves current inference or gain reduction. Do not display invented confidence,
meters or reduction. Headless callback-fault display and reset recovery pass;
native readiness/fault notification and stale-evidence qualification remain open.
Source-level callback feedback now distinguishes waiting/current detection data
for the last processed block, including valid absence. Exact source-age expiry,
epoch and reset boundaries are checked; native realtime timing is not qualified.
No Session setup is needed.

2026-10-31 engineering GUI: real original/processed 10 ms peak envelopes and
aligned applied-reduction history cover 5.12 seconds, with live input/output
meters. Freeze and waveform visibility affect display only. Range is now a
host parameter from 0 to 6 dB, saved independently of temporary consent; restored
projects remain bypassed. No confidence curve is invented from gain. Detector
thresholds, hold, ramps and evidence expiry remain fixed and visibly described.

The probe retains its own source, format identifiers, state and installed links.
This package has no desktop GUI, ownership trainer or probe dependency. Prototype
version 0.0.1 does not inherit the desktop's stable-release promise.
