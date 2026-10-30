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
meters or reduction. Native readiness/fault/stale qualification remains open.
No Session setup is needed.

The probe retains its own source, format identifiers, state and installed links.
This package has no desktop GUI, ownership trainer or probe dependency. Prototype
version 0.0.1 does not inherit the desktop's stable-release promise.
