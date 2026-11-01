# Smart Gate editor

Operate mode. Inherit the CleanSlate plugin contract and use the owner's installed
smart:gate screenshot as a hierarchy reference: dominant signal display, quiet
guides, precise nearby controls. Use original CleanSlate artwork and colors.

Default logical size is 1000×620; compact layout remains usable at 640×440.
The graph grows with available height. Original peaks are gray, processed peaks
teal, applied reduction purple. Filled input/output level histories rise from
the bottom on a −90..0 dBFS scale, following the owner's Pro-L 2 preference.
Keep actual reduction in a separate upper strip, inspired by smart:gate. Source,
processed signal and reduction share one time axis. Peak reduction receives one
numeric annotation. Put the precise Range control in a compact panel inside
the lower graph, following Pro-Q 4's contextual hierarchy. Label all scales and
keep detector readiness separate from actual applied attenuation.

Use real host data only. A 512-bin history contains 10 ms peak envelopes across
stereo channels; it is not a sample-exact waveform or a confidence estimate.
No audio produces an empty-state instruction. Stopped callbacks clear live
meters after 250 ms while history remains inspectable. Freeze affects only
the history display. No decorative motion or changes to DSP timing.

Keep the signal view edge to edge inside an 8 px inset. The owner explicitly
removed the internal title, top-right reduction readout, experimental banner and
all permanent footer/status/timing copy. The native host already supplies product
identity. Current reduction belongs in the third right-edge GR meter, alongside
IN/OUT. Keep readiness and experimental-consent details in the gate control's
hint; show an actual processing failure inside the signal view.

Controls below the graph: saved Bypass, temporary Enable/Disable gate, Freeze and
Original/Processed visibility. Numeric/drag/slider Range (0–6 dB) stays inside
the graph without moving the outer layout cursor into the signal display.
Fixed detector/timing values stay read-only in control hints. Do not draw target,
Impact or multiband controls without corresponding processing.

Reference evidence: the primary checkout's
`docs/research/smartgate-native-reference-2026-10-31.md`, especially fresh REA
record `ev_b492a411c3bd3af57cf93f003629f2b891b6daaef387b5702dd5eac7c5c0ab79`.
The reference establishes shipped structure and control text, not animation
timing or native performance equivalence.
