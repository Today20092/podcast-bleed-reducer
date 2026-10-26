# CleanSlate Smart Gate

Operate mode. A separate single-input mono/stereo microphone plugin, with CLAP
and VST3 adapters. Ticket 91 establishes identity and a safe pass-through
prototype; detector feasibility, qualified gating and the full interface belong
to 92, 93 and 94. No automatic volume leveling or paired Session capture.

The accepted visual authority is [the plugin UI contract](../../docs/design/plugin-ui-contract.md).
For this prototype, show its name, an explicit pass-through notice and saved
Bypass. Keep original audio unchanged in every state; do not display invented
confidence, meters, reduction or detector readiness. No Session setup is needed.

The probe retains its own source, format identifiers, state and installed links.
This package has no desktop GUI, ownership trainer or probe dependency. Prototype
version 0.0.1 does not inherit the desktop's stable-release promise.
