# Calibrating microphone bleed

New sessions use joint ownership. For a saved session from an older build, choose **Enable joint ownership** in the calibration controls.

1. Choose **Select calibration range** and drag across a useful part of the waveform. You can also enter start/end seconds, capture the playhead, or use the current timeline selection.
2. For **room tone**, select a section where nobody is speaking, then choose **Save room tone** under the intended microphone. This teaches its background level.
3. For **solo speech**, select a section where only that microphone's participant speaks, then choose **Save solo speech** under that track. Other microphones may contain copies of that voice; the comparison uses them together.
4. Add solo examples for other participants. Examples must span at least 0.25 seconds; choose a few seconds of representative speech when available. Avoid overlapping speakers in solo examples.
5. Compare **Original**, **Processed**, and **Removed** audition modes. Wanted words or laughter in Removed indicate a mistake: check your examples, remove misleading ones, and protect wanted speech where necessary.

Room tone alone cannot identify which participant is speaking. Solo examples provide the microphone relationship; they do not train a voice identity model. Very quiet independent speech mixed with reverberant bleed remains a difficult case and needs listening.

If a solo example is rejected because it does not show a clear relationship between microphones, remove it and choose a clearer interval with one participant speaking. The previous completed preview stays visible until valid settings produce a new one.

Examples are optional, saved with the session, and separate from protection ranges. Removing an example updates the preview. Replacing or reordering source recordings invalidates old examples. Changing settings requires a new final render before exporting updated audio.

Protection forces unity immediately at its start and keeps every protected sample unchanged. Include a little lead-in and place protection edges in quiet audio to avoid an abrupt gain change; ordinary closing resumes after the range ends.

Use the three original `chopshow podcast 155_A01.wav`, `_A02.wav`, and `_A03.wav` tracks together. These controls do not modify the original recordings.

Known limit in this build: the screenshot interval around 828–831 seconds still preserves the A02/A03 bleed. Guided calibration modestly improves other labeled bleed sections, but does not solve every copied-speech case. More aggressive padding changes were rejected because they could damage short independent speech. See `.scratch/guided-ownership/evidence/quality-results.md` for the measurements and remaining cases; the recordings still need your listening review.
