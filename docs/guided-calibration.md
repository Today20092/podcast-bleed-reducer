# Calibrating microphone bleed

New sessions compare microphones to distinguish direct speech from bleed. This is the algorithm's "ownership" decision: which microphone or microphones contain the intended speaker. For a saved session in legacy mode, choose **Enable microphone comparison** under **Microphone calibration**. The previous build called this button **Enable joint ownership**.

1. Choose **1. Select an example**. Press the left mouse button on a waveform, drag across a few seconds, then release. Dragging in either direction works. The blue highlight shows the same time interval on all tracks. A single click creates no usable range.
2. For **room tone**, select a section where nobody is speaking, then choose **Save room tone** under the intended microphone. This teaches its background level.
3. For **solo speech**, select a section where only that microphone's participant speaks, then choose **Save solo speech** under that track. Other microphones may contain copies of that voice; the comparison uses them together.
4. Add solo examples for other participants. Examples must span at least 0.25 seconds; choose a few seconds of representative speech when available. Avoid overlapping speakers in solo examples.
5. Compare **Original**, **Processed**, and **Removed** audition modes. Wanted words or laughter in Removed indicate a mistake: check your examples, remove misleading ones, and protect wanted speech where necessary.

The selection summary shows its start, end and duration. Save buttons stay disabled for an empty or shorter-than-0.25-second range, or while legacy detection is selected. Click **Finish selecting** to return to ordinary playhead navigation. You can also hold Shift while dragging to create a normal timeline selection.

For precise entry, expand **Exact times / existing selection**. **Copy highlighted timeline range** copies the current blue highlight into the time fields. This was called **Use timeline selection** in the previous build. **Start at playhead** and **End at playhead** copy the white cursor position; clicking both without moving it produces a zero-length range.

For example, if only the A01 participant is speaking, select that time and use **Save solo speech** under A01, even though the voice is audible as bleed on A02/A03. For room tone, choose a time when nobody is speaking, then save it under the microphone whose background you want to describe. Room tone is not a label for another participant's voice leaking into that microphone.

Room tone alone cannot identify which participant is speaking. Solo examples provide the microphone relationship; they do not train a voice identity model. Very quiet independent speech mixed with reverberant bleed remains a difficult case and needs listening.

If a solo example is rejected because it does not show a clear relationship between microphones, remove it and choose a clearer interval with one participant speaking. The previous completed preview stays visible until valid settings produce a new one.

Examples are optional, saved with the session, and separate from protection ranges. Removing an example updates the preview. Replacing or reordering source recordings invalidates old examples. Changing settings requires a new final render before exporting updated audio.

Protection forces unity immediately at its start and keeps every protected sample unchanged. Include a little lead-in and place protection edges in quiet audio to avoid an abrupt gain change; ordinary closing resumes after the range ends.

Use the three original `chopshow podcast 155_A01.wav`, `_A02.wav`, and `_A03.wav` tracks together. These controls do not modify the original recordings.

Known limit in this build: the screenshot interval around 828–831 seconds still preserves the A02/A03 bleed. Guided calibration modestly improves other labeled bleed sections, but does not solve every copied-speech case. More aggressive padding changes were rejected because they could damage short independent speech. See `.scratch/guided-ownership/evidence/quality-results.md` for the measurements and remaining cases; the recordings still need your listening review.
