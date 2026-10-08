# Import fixtures

These tiny synthetic recordings are generated locally; they contain no owner audio.
FFmpeg generates fixtures only. CleanSlate runtime decoding uses Rust libraries.

The `mono44100` fixtures encode the same 100 ms, 1 kHz mono sine at 44.1 kHz.
Their accepted native timeline is the decoder's actual output, with validated
gapless container timing where the pinned decoder supports it:

| File | Encoder/profile | Native decoded frames |
| --- | --- | --- |
| `mono44100.aiff` | PCM16 big endian | 4,410 |
| `mono44100.flac` | FLAC | 4,410 |
| `mono44100.mp3` | libmp3lame | 4,410 |
| `mono44100-aac.m4a` | AAC-LC | 6,144 |
| `mono44100-alac.m4a` | ALAC | 4,410 |
| `mono44100.ogg` | libvorbis | 4,410 |

AAC-LC is not advertised as gapless. These six AAC packets decode 1,024 frames
each; presentation edit-list trimming is not applied. Import retains all 6,144
decoded frames and warns about priming/padding. A 4,410-frame PCM companion fails
physical-duration validation; declaring synchronization never overrides that
conflict. Symphonia is pinned to 0.6.1: the short single-page Vorbis fixture and
AIFF COMM count also regress timing problems observed with 0.5.5.

`mono48000.wav` and `mono96000.wav` are the 100 ms sine as IEEE float32 WAV.
`pcm8.aiff`, `pcm24.aiff`, and `pcm32.aiff` cover signed AIFF PCM8/24/32 at
44.1 kHz, each with 4,410 decoded frames. WAV PCM support is specifically
16/24/32; `unsupported-pcm8.wav` is unsigned PCM8 and is rejected before its
unsupported bwav frame-reader branch can be called.
`pcm16.wav`, `pcm24.wav`, `pcm32.wav`, and `float64.wav` exercise WAV PCM16/24/32
and IEEE float64 at 48 kHz. Decoding/analysis uses internal f32, so float64 and
PCM32 precision beyond f32 is quantized in that internal representation. Source
bytes, native rate, and native frame count are retained; no bit-exact float64
decode/export round trip is claimed.
The retained renderer exports float64 input as explicit native-rate float32 WAV;
it does not call bwavfile's unsupported float64 frame writer.
`mono44100-aac.mp4` and `mono44100-alac.mp4` exercise the MP4 brands with the same
respective decoded lengths as their M4A counterparts.
`impulses44100.wav`, `impulses48000.wav`, and `impulses96000.wav` contain impulses
at 10 ms and 90 ms, exercising resampler delay compensation and partial tail flush.
`outofband48000.wav` and `outofband96000.wav` contain a one-second 12 kHz sine;
the central output measures rejection above the 16 kHz analysis Nyquist limit.

`unsupported-opus.ogg` is a real libopus encoding; `unsupported-stereo.wav` is a
two-channel PCM16 WAV. Neither profile is accepted.
`unsupported-multiple-audio.mp4` contains two separately encoded mono ALAC streams
and must fail rather than silently selecting one stream.
Tests create truncated copies of accepted encoded fixtures and synthetic BWFs
with rate-normalized origins.
No HE-AAC support is claimed: only AAC AudioSpecificConfig object type 2 is accepted.

Analysis conversion uses bounded native decode blocks and disk-backed mono 16 kHz
float32 copies. Rubato's rational FFT geometry is checked before allocation:
input FFT length at most 1,048,576 and output FFT length at most 65,536,
with input chunks at most 4,096 frames. This is a geometry limit rather than a
sample-rate range; safe low, high and odd rates are tested. Resampler recipe v2
uses power-of-two rational factors, compensates startup delay and flushes exactly
the ceiling of the physical source duration. Recipe changes invalidate copies.

Generation examples (replace codec/output for each table row):

```powershell
ffmpeg -hide_banner -loglevel error -f lavfi -i 'sine=frequency=1000:sample_rate=44100:duration=0.1' -c:a pcm_s16be mono44100.aiff
ffmpeg -hide_banner -loglevel error -f lavfi -i 'sine=frequency=1000:sample_rate=44100:duration=0.1' -c:a aac mono44100-aac.m4a
ffmpeg -hide_banner -loglevel error -f lavfi -i 'sine=frequency=1000:sample_rate=48000:duration=0.1' -c:a pcm_f32le mono48000.wav
ffmpeg -hide_banner -loglevel error -f lavfi -i 'aevalsrc=if(eq(n\,480)\,1\,if(eq(n\,4320)\,1\,0)):s=48000:d=0.1' -c:a pcm_f32le impulses48000.wav
ffmpeg -hide_banner -loglevel error -f lavfi -i 'sine=frequency=12000:sample_rate=48000:duration=1' -c:a pcm_f32le outofband48000.wav
ffmpeg -hide_banner -loglevel error -f lavfi -i 'sine=frequency=1000:sample_rate=48000:duration=0.1' -c:a libopus unsupported-opus.ogg
```
