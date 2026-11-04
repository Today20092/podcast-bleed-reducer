# Actual PostGate DSP: bounded native findings

Target SHA256 `edeb18dcb0b7557123cc46036fd17b2d08cbd45d3bf1cb882e4b5a8a4a35f95c`.
Read-only interpretation of retained run-02 Ghidra exports, checked against ordinary
disassembly for the consumer, curve and output multiplication. No target execution,
plugin changes, or independent Ghidra import. Bodies and matching metadata/assembly
are under `run-02/dossiers/` beside this report's parent directory.

## What the stage actually does

`181794b00` is the proved consumer of the PostGate state P embedded at D+360.
It reads P+04/+08/+10/+14 together. It forms a detector buffer from the context's
sample pointers at +10, channel count +18, offset +20 and length +28. It computes
an absolute-value or RMS-style envelope, optionally normalizes that envelope,
maps it to a gain, smooths the gain, and multiplies the separate output block
referenced by context+08. Ordinary disassembly of `180af7150` confirms actual
elementwise sample multiplication; this is not merely a displayed gain value.

Its detector is **not a direct ONNX score load**. Where neural evidence affects
the earlier spectral processing is a separate analysis route. The chain caller
`180a5ecd0` invokes `180a64c50` with a descriptor taken from context+08 before
calling PostGate with a copy of the same complete context. Thus separate detector
and output routes are established. Whether the detector's +10 pointer aliases
original, delayed, remixed or processed PCM still requires the upstream context
construction and preceding stage binding. Do not call it a neural confidence
envelope, or claim original-input identity from pointer offsets alone.

The decompiler repeats the first channel pointer in its summing loop. Ordinary
assembly confirms this: 181794c88 loads context+10, 181794c98 loads `[rax]`
without a channel index, and the outer ccc branch returns to c80. Thus this body
adds the first pointed sample stream channelCount times. Whether this is a mono
specialization, an upstream replicated stream or a routing issue remains unknown;
do not describe it as stereo averaging. Helper `180af6f10` determines the final
channel-count normalization. Its now-exported scalar/SSE branch explicitly
divides each buffered sample by the supplied channel count; consequently this
branch's repeated-first-stream sum reduces to the first stream. Other CPU
dispatch branches call 180af8150/180af8590 and are not interpreted here.

The exported preceding stage `180a64c50` accumulates blocks until state+6698
reaches0, applies per-channel helper180a6d610 with data at +6558, then calls
the inner processor **180a6daf0(state+40, context)** unless state+20 is set.
It calls the per-channel helper again with data at +6578 and enqueues the
result in its output buffer. This establishes a block-processing wrapper;
window type, hop/FFT size and neural/spectral gain math require the inner body,
which has been requested. No FFT equation is inferred from the wrapper name.

Follow-up: inner wrapper180a6daf0 is also exported. Its assembly calls
180001160 at180a6db9c with a channel's time-domain source and transform buffer,
then calls **180a532e0(state+40, spectralContext)** at180a6dbe9 unless bypassed,
then calls180002270 at180a6dc37 with the channel transform buffer and output.
The context passed to180a532e0 begins with state+6448. The actual numerical
spectral processor is therefore180a532e0, now requested. Ghidra's mistaken
noreturn annotation collapses both transform calls in pseudocode to
thunk181b3d260 and omits their fallthrough. It is not evidence that channels>0
abort processing. Exact transform identity/scaling and omitted fallthrough
must be checked in ordinary disassembly; no dead-path claim is made.

## Exact local gain equations

Let x be the sample entering this detector and z its previous level state.
With detector mode P+260=1, q=x*x; otherwise q=abs(x).

`z_new = q + (z-q)*c`, selecting P+258 when q>z and P+25c otherwise.
For mode1, `e=sqrt(z_new)`; otherwise `e=z_new`.
If P+01 is enabled, the curve input is `a=e/(e+b)`, where b=P+08.
Otherwise a=e. This gives an actual meaning to the controller's exponential
level-bias conversion; it is not evidence of a weighted ML/level blend.

The controller ingress already proved:

`b = 2^((public_level_bias/100)*6.228818 - 9.965784)`.

For threshold T=P+10, tolerance v=P+14, and floor m in dB, `181795da0` is:

```
L = (1-v)*T
if a >= T: dB = 0
else if a <= L or T-L < 0.001: dB = m
else:
    g_floor = 10^(m/20)
    g = g_floor + (a-L)*(1-g_floor)/(T-L)
    dB = 20*log10(g + FLT_MIN)
```

This is a relative-width, linear-amplitude soft transition. This helper contains
no previous-open/closed state and therefore is not itself Schmitt hysteresis.
The strict width refusal is `<0.001`, verified in assembly. Other stages may
have other state and must not be described by this helper alone.

If P+0c==1 the consumer replaces dB with `m-dB`; it then clamps to [m,0]
and computes `10^(dB/20)`. This is a local inverted attenuation branch.
The public Duck selector has **not** been proved to write P+0c. In particular,
the known PostGateController operation-mode handler changes stage eligibility;
that must not be mistaken for this member's writer.

After AHR smoothing the final gain is `1-I + I*g`, I=P+04. The known PostGate
controller sets I to public impact/100 multiplied by an attack-dependent factor:
`1-(clamp(attack_ms,10,50)-10)/40`. Thus this cleanup stage fades away by50ms
attack. This is not the whole plugin's Impact law: the spectral controller also
receives Impact separately, as described below.

## AHR state and recurrence, not a guessed fixed time constant

`1817aa280` compares current target gain t with previous emitted gain y. If they
differ, the direction flag becomes t<y. Falling and rising directions have
separate counters; each call clears the opposite counter and increments the
selected one: `i=min(counter,H); counter=i+1`. With selected
counter-derived `n=(i+1)-H`, negative n returns the prior y unchanged.
Otherwise, if transition length N>0:

`c = exp(min(n,N)*slope + intercept)`

and if N<=0, c=0. Output and saved state are:

`y_new = (1-c)*t + c*y`.

Crucially the counter caps at H before adding1. Therefore, once it reaches the
hold boundary, n saturates at1 rather than continuing through the transition
length N. For ordinary nonnegative H this yields a steady direction-specific
coefficient after the hold, not an N-sample coefficient sweep. The exact
recurrence governs interpretation; the duration argument alone does not prove
a time-varying ramp. Earlier shorthand calling it a nonstationary exponential
envelope was too strong and is superseded by this explicit counter behavior.

The coefficient setup `1817a9750`, for a kernel expressed as float slots k[],
is directly recovered as:

```
N = k[0] * k[1]                     # sample rate * transition seconds
H = rounded_int(k[0] * k[2])        # sample rate * hold seconds
shape = 1 + (k[6]-1)*max(k[4],0)
base = log(1-k[7])/(N+1)
intercept = shape*base
slope = 2*(1-shape)*base/N          # N>0; otherwise large sentinel
```

The rounding uses the familiar double add-2^52 integer conversion; assembly
should govern tie/rounding details. The consumer divides the first and fourth
fields of the queued six-float timing record by5 before assigning the two
kernels. P+1c controls which kernel receives which half. Exact public attack,
release, hold labels for each direction require this assignment plus runtime
kernel initialization; names alone are not enough. Existing ingress confirms
public milliseconds become seconds and an internal release offset exists.
Do not present these internal construction values as measured factory presets.

Constructor `181795160` initially writes detector mode0 but later **overwrites
P+260 with1**, establishing final constructed RMS mode. P+18 is initialized
to `10^-5`, so the local constructed floor m is approximately -100dB. Later
prepare/settings writers can change construction state; this does not establish
the full runtime preset or plugin-reported latency. The constructor also builds
AHR state through `1817ba8c0`; that body is needed for numerical shape/accuracy
defaults rather than guessing them.

## Same public controls feed a distinct spectral stage

Actual dispatcher `1817d6460` belongs to the independently typed
SpectralProcessingController/ComplexSpectralGate route. Its parameter ingress
proves the following separate updates:

| Public identifier | Value queued into spectral state |
|---|---|
| input_gain | x<=-100 gives0, otherwise 10^(x/20), at state+1de0 queue |
| learner_profile | tagged enum retained at controller+28; a changed non-sentinel profile sets spectral state+2214 invalidation flag |
| background_suppression_strength | x/100 at state+b00 queue |
| background_suppression_enabled | boolean at state+a00 queue |
| gate_threshold | x/100 at state+11a0 |
| gate_tolerance | x/100 at state+1080 |
| impact_mix | sqrt(x/100) at state+1cc0 |
| impact_low/mid/high | x/100 into three separate queues |
| impact_enabled_low/mid/high | separate boolean queues |
| gate_attack/release/release_hold | milliseconds/1000 in a timing record |
| level_bias | raw x into state+c0 queue |
| gate_max_attenuation | raw x into state+f60 queue |
| gate_operation_mode | enum payload into state+e80 queue |
| impact_lower/upper_split_frequency | ordered split-frequency updates |
| sidechain_channel_mode | enum payload at state+8e0 queue |

These are ingress facts, not yet the spectral algorithm's consumer equations.
They establish that replacing the complete plugin with only this PostGate knee
would omit independently controlled spectral processing. No whole-plugin
Threshold/Tolerance/LevelBias/Impact/Duck/latency claim follows from PostGate alone.

This is the complete unique string-dispatch inventory in these two exported
controllers: 23 spectral identifiers, 12 PostGate identifiers (all a subset of
the spectral identifiers). It is not a complete parameter-registry inventory.
In particular output_gain, global bypass, input routing, monitoring and GUI-only
state are not dispatched in these two bodies; absence here is not absence from
the product. These native handlers establish internal identifiers, payloads and
conversions. A visible widget, automation range/default or manual-facing label
requires separate registry/manual evidence. The `learner_profile` name and
invalidation flag do not prove user-supervised model training.

PostGate eligibility coupling is explicitly
`P[0] = S[128] && S[129] && S[12a] && S[12b] && S[12c] && S[12d]`.
The operation enum sets S129=(payload==0), solo enum sets S12a=(payload==0),
and band-enable booleans set S12b/c/d. Thus a nonzero operation enum, nonzero
solo selection or any disabled band disables this cleanup stage. This does
not establish which public enum label is Duck and does not write P+0c.
The origin/runtime writers of S128 still require separate evidence.

## Evidence manifest

All addresses have `.c`, `.metadata.json` and `.assembly.json` retained.

| Address | Evidence ID | .c SHA256 |
|---|---|---|
|181794b00|ev_54073490a9dd09245916e49806fd4f9c71163320e3bef8aa3207ba537f601e29|413d49343c8f2eb86b87a66d93c47daaca3ba04964e717280329ae72db302a6c|
|181795da0|ev_e16e06e9541613ebb18a0805723456ef42fa35e0d69f366c27d2b8236445e045|0c2f694921dbd3af02c62af2c3699b7ebd21c9f87616cdee03f27a5c7dd6463e|
|1817aa280|ev_b63612705dfea74cdd103c37b9bcc384f6cd484468938ccc72ba0e1ca0469c53|9ed2eac08d5730443699cc7beb452f1a48bace8122c880b5c90fd35046b9ec9d|
|1817a9750|ev_3299927edd586d5e77beb1f66de7fc05bb24643d263f06ee2d3ed5f07cf07fc7|e964db59fd62d73aa48907481009902e702f57b11dd8a45ac58f2624dfc5de82|
|180a5ecd0|ev_d4a597232aab3cbdbbe34625df0ffd028fabcdad48852f2803b6fe2ead7caa9f|075177caae03895619ee1227be3616a11295eb29be7331261db08274b6fc7d77|
|180a5fb00|ev_7528279da30dfc2a451c2940df8348b67301a0171c3aede44a2ba0c0952b0f3c|725ba44e804704b900bc34000abc70d1ad2364a3e1f33b50c449a4f4d80beeba|
|1817d6460|ev_0d3a21116b8e21fd255bb9ff59d8a3ef854d7b818b28194e3c40a3166b5c7473|2708c311aff1a468f99aa54001cb7dbae88d5c20c533412976106bbf190d8f83|
|181795160|ev_07c419429d9c5e647e764874fa9b0709c5ae04611f38477b1ef6e36bdcb66078|d398c0572b95501a9d750f023c4c0284f07c5cdaf461681edd8714e6c7129ab5|
|180a64c50|ev_20e50d54eeb83273e80784ed34750cf7032c1cc41761dd2bf0c8513c12548e0d|e580e83eacea6bd919b64233d8508b16cff618879e05c958a19a8fef338eee05|
|180af6f10|ev_f9ee2bce61e234e0f0118b94b31ec31b2c62ce45519e05fafac5864be21370b8|914fd9e51e6ad2935723cbcec926e5d9ab76d79d833f1aa8a51aa6f02dfdd122|
|180a6daf0|ev_1a1fa7318c8dcc1f1a2843ea20ebd6c6e83beab4508a4eff7b8f6f66a58fbfcb|6b997fac1519f233db8a3e4fac98e80f6209348f4bf70cff56a9f5fd244414ff|

Decompiler limitations: missing inferred float/register arguments in some calls,
unreachable-block warnings and split-function ranges require ordinary assembly
cross-checks. The verified full consumer is not its tiny initial pdata prologue.
No infinite-loop conclusion is drawn from security-cookie decompiler artifacts.

