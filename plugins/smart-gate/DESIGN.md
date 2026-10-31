# Smart Gate editor

Operate mode. Inherit the CleanSlate plugin contract and use the owner's installed
smart:gate screenshot as a hierarchy reference: dominant signal display, quiet
guides, precise nearby controls. Use original CleanSlate artwork and colors.

Default logical size is 1000×620; compact layout remains usable at 640×440.
The graph grows with available height. Original peaks are gray, processed peaks
teal, applied reduction purple. Source and processed envelopes share one time
axis with the reduction trace. Label the reduction scale, time scale and dBFS
meters. Keep detector readiness separate from actual applied attenuation.

Use real host data only. A 512-bin history contains 10 ms peak envelopes across
stereo channels; it is not a sample-exact waveform or a confidence estimate.
No audio produces an empty-state instruction. Stopped callbacks clear live
meters after 250 ms while history remains inspectable. Freeze affects only
the history display. No decorative motion or changes to DSP timing.

Controls below the graph: saved Bypass, temporary Arm/Disarm, Freeze,
Original/Processed visibility, and numeric/drag/slider Range (0–6 dB).
Fixed detector/timing values remain visible and read-only. Do not draw target,
Impact or multiband controls without corresponding processing.

Reference evidence: the primary checkout's
`docs/research/smartgate-native-reference-2026-10-31.md`, especially fresh REA
record `ev_b492a411c3bd3af57cf93f003629f2b891b6daaef387b5702dd5eac7c5c0ab79`.
The reference establishes shipped structure and control text, not animation
timing or native performance equivalence.
