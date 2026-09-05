# Calibration mapping provenance

On 2026-09-05, read-only PCM comparisons found the original `00005603_A01/A02/A03.wav` excerpts match `chopshow podcast 155_A01/A02/A03.wav` at an offset of **8,973,765 samples**, or 186.9534375 seconds at 48 kHz.

Compared 4,800-sample windows beginning at excerpt-relative 0, 5, 7.386, 47.868 and 145.229 seconds on every microphone. All 15 windows had maximum absolute sample difference zero. At the 5-second window, candidate offsets 8,973,761 through 8,973,767 were tested. Only 8,973,765 matched exactly on all three microphones. The adjacent 8,973,764 offset gave maximum absolute differences 0.0022151482, 0.0018413700 and 0.0227812417 on A01/A02/A03 respectively.

BEXT reports 8,973,764, one sample earlier. Use the measured PCM offset for example metadata. Excerpt labels remain excerpt-relative when scoring excerpt files; no offset is applied there.

The `full` evaluator preset maps only the four calibration ranges from evaluation-plan.md. It does not use 828–831 seconds to calibrate. Candidate solos are inferred from sparse human labels and do not prove that no unannotated speaker was present. A03 is the only microphone with a human room-tone example.

Method: imported the existing read-only `probe_copy.py` `read_section` function through `uv run --no-project --python C:/Users/User/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe python -c ...`; compared `numpy.max(numpy.abs(excerpt-full))`. No source or generated audio files were written.
