# Product opportunities for podcast bleed reduction

2026-10-22. Research requested by the owner. These are recommendations for discussion, not accepted implementation scope or a replacement roadmap. No application behavior, ticket, release decision or dependency is changed.

## Recommendation

Make CleanSlate unusually good at answering three questions: **Did it keep every voice? What should I listen to next? Can I safely hand these tracks back to my editor?** The strongest next investments are better representative examples, faster comparison and focused listening. A larger mastering chain would contribute less directly to those questions.

Ozone is a useful model for evaluating decisions: quick alternative settings, accessible bypass and deliberate audition. RX is a closer technical reference for restoration. Neither product's documentation establishes that its algorithms outperform CleanSlate on the owner's recordings. All rankings, effort estimates and proposed success checks below are engineering judgments, not measured user-study results.

The three most promising workflow investigations are **guided representative examples**, **two sound-setting snapshots**, and **opt-in focused audition**. Rank their prototypes by preserved quiet speech and reduced review time, rather than by resemblance to a commercial suite. The best bounded audio experiment is to measure whether later representative solo examples improve existing acoustic fits after movement before adding time-local calibration.

## Current product and boundaries

The current [domain vocabulary](../../CONTEXT.md), [ADR-0006](../adr/0006-offline-supervised-editable-envelopes.md) and latest amendments to the [accepted specification](../../.scratch/offline-supervised-migration/spec.md) take precedence over older passages in that specification. In particular:

- CleanSlate already runs local acoustic ownership analysis with human examples, embedded speech detection and independent per-track ownership models. Nemotron has been removed.
- Original, Processed and Removed audition, combinations of soloed tracks, live attenuation/fade/gap adjustment, protected manual edits and explicit Analyze already exist. These should be improved where needed, not proposed as new features.
- Multichannel recorder import, separate/multichannel WAV delivery, source timing safeguards, project persistence and customizable shortcuts already exist.
- [Acoustic evidence](../acoustic-evidence.md) already includes directed leakage, short-delay correlation and a conditional held-out-validated two-tap transfer filter. Adding FIR evidence is not a new feature opportunity. Only solo references fit pure bleed relationships; overlap labels teach ownership instead.
- The current GUI uses waveform inspection and audition. The historical uncertainty review queue is absent by owner refinement; restoring it requires a new product decision.
- The owner accepted current sound/performance and closed ticket 14. Independent and long-session measurements that were not performed remain unperformed. This note does not reopen that ticket or reinterpret acceptance as statistical proof.
- The [current implementation queue](../../.scratch/offline-supervised-migration/implementation.md) retains cleanup 22. Menus 60, action registration 63 and toolbar icons 69 are owner-accepted and integrated into development. Advanced training 17 and waveform-overlay editing 35 remain deferred; live-plugin discovery 67 is outside the build queue. Proposals here require their own accepted scope before implementation.

The engine applies gain to a whole microphone signal at each instant. If a wanted voice and another speaker's bleed coexist in that microphone, reducing that signal also reduces the wanted voice. Better ownership and fades can reduce incorrect attenuation, but cannot selectively extract one voice from simultaneous content. A future separation/subtraction engine would be a different audio-processing contract.

## What the references actually support

| Reference | Verified feature or documented behavior | Useful implication for CleanSlate |
| --- | --- | --- |
| [Ozone 12 general controls](https://docs.izotope.com/ozone12/en/general-controls.html) | Gain Match, bypass, reference playback and four A/B/C/D history snapshots are documented. | Let users compare alternatives without losing their work. Start with two parameter states; four slots are not a requirement. |
| [Ozone 12 Master Assistant tutorial](https://www.izotope.com/community/blog/how-to-use-master-assistant-in-ozone) | Analysis proposes a starting point; users can customize targets and processing, and relearn. | Assistance should suggest a next action while exposing the decision. A music genre target is not a speaker-ownership target. |
| [Ozone 12 Codec Preview](https://docs.izotope.com/ozone12/en/codec-preview.html) | Users can hear codec changes, including isolated removed/modified content, before export. | Inspect consequences audibly. CleanSlate's existing Removed mode already captures the core principle for its own gain processing. Codec preview becomes useful only if final distribution encoding enters scope. |
| [RX 12 common controls](https://docs.izotope.com/rx12/en/common-module-controls.html) | Preview, looping, removed-content listening in some modules, and Compare support evaluation before rendering. | Keep listening and comparison inexpensive and separate from committing deliverables. |
| [RX 12 De-bleed](https://docs.izotope.com/rx12/en/de-bleed.html) | Current De-bleed uses ML signal isolation and offers an inverted-bleed audition. Classic learns the relationship between two tracks and requires matching sample rates and time alignment. | Research separation separately from gain envelopes. Do not copy RX's equal-rate requirement into CleanSlate's existing mixed-native-rate contract. |
| [Auphonic multitrack algorithms](https://auphonic.com/help/algorithms/multitrack.html) | Auphonic describes joint track analysis for bleed reduction and gating, with loudness normalization/true-peak limiting on the final mixdown. | Distinguish bleed cleanup from delivery mastering. Activity and bleed context should be considered across microphones. |
| [Auphonic multitrack best practices](https://auphonic.com/help/algorithms/multitrack_tips.html) | Documentation calls for parallel timing and separate speaker tracks, supports processed-track output, and warns that disabling several reduction algorithms can preserve ambience. | Make recording suitability and downstream handoff clear. Evaluate room naturalness as well as how much attenuation occurred. |
| [Sound Radix Auto-Align Post](https://www.soundradix.com/products/auto-align-post/) | Dynamic/static time alignment, spectral phase correction and a correlation meter address microphone timing/phase relationships. | Diagnose changing microphone relationships before considering automatic correction. Clock drift, propagation delay and wrong speaker ownership are different problems. |
| [Hush Pro](https://hush.audio/products/hush-pro) | Hush Mix offers speech/reverb/noise faders and solo/bypass preview; Hush Split renders separate layers. The documented product targets Apple Silicon and Pro Tools AudioSuite. | Simple controls plus immediate audition are useful references. Dialogue/noise separation is not evidence of separating two overlapping podcast speakers, and this is not a Windows integration recommendation. |
| [Waves Dugan Automixer](https://www.waves.com/mixers-racks/dugan-automixer) | The documented gain-sharing approach distributes one open microphone's gain over the system and provides automatic/manual/mute states. | Smoothness and understandable intervention matter, but fixed total-gain sharing would conflict with CleanSlate's exact-unity overlap preservation. Do not substitute it for current ownership. |

These are first-party feature descriptions. Claims such as perfect recognition, transparency or minimal artifacts on vendor pages are marketing claims, not independent evidence. No listening comparison was performed for this note.

## Ranked opportunities

### 1. Help users choose representative examples

**Value:** high. **Estimated effort:** small to medium for guidance; larger for reliable automatic suggestions. **Status:** proposed extension of the current labeling workflow, distinct from deferred advanced training panel 17.

Offer concise, recording-specific advice beside the existing example controls: include normal and quiet speech for each speaker, an interruption/overlap, laughter when relevant, and clean room tone. Show which people have examples and where they occur in the recording. Suggest a later example when all examples come from the first few minutes. Do not invent a fixed number of examples that guarantees quality.

The first version can use label coverage and existing validation without adding a model. More ambitious candidate suggestions must remain unlabeled until the user listens and confirms. Predictions must never become training truth automatically. Ozone's editable assistant is the interaction precedent; the concrete reference guidance is a CleanSlate proposal.

**Success check:** on a small set of authorized sessions, record setup time, reference count and corrections required after the first analysis. Include quiet speakers and later microphone movement. Compare with today's workflow without changing sources or test regions between alternatives. Saving an example must retain current stale-analysis and manual-edit behavior.

### 2. Add two fast sound-setting snapshots

**Value:** high. **Estimated effort:** medium. **Status:** new proposal using the existing live envelope path.

Let the user compare “gentler” and “stronger” sound settings at the same selection, solo state and playhead, borrowing the comparison principle from Ozone's snapshots and RX Compare. Start by snapshotting attenuation, fades and gap settings only. Keep the same current analysis and manual edits in both states. Changing labels invalidates the underlying analysis normally; snapshots must not hide that state.

Keep temporary comparison separate from “use these settings,” with one undoable application. Avoid a full project-branching system or retained duplicate PCM. Reuse the effective-envelope evaluator.

**Success check:** switching states does not seek, retrain, drop manual corrections or change source identity. The chosen state exports the same envelope heard in ordinary Processed audition. Listen through quiet interjections, rapid turns and overlap before judging stronger reduction as better.

### 3. Make a short, deliberate listening pass easier

**Value:** high. **Estimated effort:** medium. **Status:** requires owner agreement because the old review queue was intentionally removed.

Explore a lightweight “next place to listen” command rather than rebuilding the historical queue. Candidate jumps could include heavy attenuation near speech-like evidence, rapid ownership changes, or a user bookmark. Each suggestion needs a plain reason and enough surrounding audio to hear the transition. A reason is a heuristic, not a calibrated probability that speech was lost.

Use the existing Original/Processed/Removed modes and solo controls. Show the current track and reason while preserving selection and transport context. Evaluate whether a simple user-created bookmark plus next/previous navigation already solves most of the need before adding automatic ranking.

**Success check:** measure time to find deliberately included quiet interjections and bad transitions compared with free waveform scanning. Record misses as well as false alarms. A heuristic must not change the gain result or turn “visited” into “verified safe.”

### 4. Explain changing acoustic relationships

**Value:** high for long or mobile sessions. **Estimated effort:** medium to large; research before implementation. **Status:** future diagnostic proposal; automatic alignment remains deferred.

Investigate whether existing acoustic-fit evidence can identify regions that disagree with the recording's reference relationships. Explain a useful next step: listen here, add an example from this later section, or inspect externally for synchronization trouble. Avoid “microphone moved” unless the evidence can distinguish movement from background changes, overlapping voices and weak measurements.

The owner's [synchronization and drift decision](synchronization-and-drift.md) explicitly defers alignment and clock-drift correction to an external editor. Acoustic-change diagnosis must not quietly revive that work. Use the implemented scalar/filter alternatives as experiment controls, not an assumed need for a new filter. Frequency-dependent attenuation would also change the existing whole-signal gain contract and needs separate evidence and approval.

A later experiment could compare one whole-session calibration with explicit user-defined recording sections. Section changes must preserve human constraints, stable time coordinates and manual edits. Do not add segmented models before demonstrating that the simpler extra-reference workflow fails.

**Success check:** test known head turns, microphone movement, level changes, quiet speech and independently recorded clock drift. Report detection coverage and false alerts separately. No diagnostic may silently stretch, shift or replace a native source.

### 5. Make exported-track handoff self-explanatory

**Value:** medium to high. **Estimated effort:** small for clearer existing UI; medium for a durable report. **Status:** extension of existing safe exports, not a new render engine.

Explore a compact export summary: exact destination, separate versus multichannel layout, channel names/order, native rates and lengths, encoding, analysis identity and completion status. Offer a small optional sidecar for reproducibility if users actually hand projects to someone else. It should identify processing/settings and files without embedding source recordings, private absolute paths or unnecessary machine data.

Keep “ready for further editing” distinct from “mastered for publication.” Auphonic's separate processed-track and final-mix stages are a relevant precedent. Re-importing processed stems into a DAW is the first handoff target; AAF, EDL and automation export need separate requirements and are outside current scope.

**Success check:** import exported stems into the owner's actual editor and verify common starts, sample counts, channel identity and transitions against CleanSlate audition. Cancel/failure must retain existing atomic publication guarantees. Existing export protection should be reused.

### 6. Turn difficult real recordings into a reusable listening set

**Value:** high for choosing the next algorithm improvement. **Estimated effort:** medium, mostly curation. **Status:** optional research evidence, not reopening owner-closed ticket 14.

Keep a private set of short authorized cases: whisper under loud speech, “yeah” interruptions, simultaneous laughter, breath-to-word onsets, room tone between phrases, long quiet returns, and microphone movement. Separate development examples from held-out recordings. Use the frozen `6a60cc2` baseline only where the acceptance comparison specifically calls for it; today's implementation also deserves its own reproducible reference.

Record wanted-speech attenuation, residual bleed on known non-owner spans, manual repair count and listening preference separately. “More reduction” and “fewer audible voices” are not sufficient quality metrics. Preserve exact unity on trusted speech/overlap absent an intentional manual edit, and inspect Removed for wanted speech.

Include desired breaths, laughter and background continuity in listening review: a quieter room is not automatically a more natural room. First evaluate the existing attenuation floor, fades and gap controls for pumping. Synthesizing replacement room tone would add audio that is absent from the current native-source gain path and is a separate proposal, not the default fix.

**Success check:** a proposed algorithm change beats the current version on its stated target cases without hiding a new preservation regression. Keep numerical measurements and subjective listening decisions separate. Publish no private recording by default.

### 7. Consider optional comparison gain compensation carefully

**Value:** medium. **Estimated effort:** medium. **Status:** experimental monitor-only proposal, lower priority than snapshots.

Ozone's Gain Match reduces loudness bias when comparing processing. CleanSlate already uses identical monitor headroom across listening modes; that is not gain matching. Whole-session loudness matching can be misleading here: removing unwanted inactive-microphone sound legitimately changes the total level, while boosting Processed may conceal a preservation problem.

If explored, derive compensation from trusted wanted-speech material, show the offset explicitly, bound it safely, and keep raw comparison readily accessible. Never apply compensation to exports or to the mathematical Removed calculation. First test whether same-gain comparison plus snapshots is sufficient.

Use one documented calibration over a meaningful selected speech/program interval across the monitored stems. Do not normalize each inactive microphone or tiny segment independently: doing so can amplify residual bleed and erase the benefit being evaluated. Keep Removed as the unmodified source-minus-processed diagnostic, separate from any optionally matched Original/Processed audition.

**Success check:** audition cannot clip, raw mode remains reproducible, wanted-speech losses remain audible, and monitor offsets never enter project gain edits or rendered PCM.

### 8. Separate simultaneous-speaker bleed as a longer-term experiment

**Value:** potentially high; **effort and preservation risk:** high. **Status:** outside the accepted attenuation-only architecture.

RX's learned/reference-based and ML isolation modes justify investigating other processing families, not assuming their result transfers to podcasts. A research experiment could compare multichannel reference-conditioned separation or adaptive transfer/subtraction against the current envelopes on difficult overlap. Start offline on copies with a bypassable result and private held-out material. Licensing, model size, CPU/memory, timing and source-identity effects require investigation before integration.

**Success check:** intelligibility and wanted-voice timbre survive while simultaneous unwanted voice decreases; no time shift, missing tail, phase damage or new artifacts are hidden by an average score. If this does not demonstrate a clear benefit on the owner's recordings, stop the experiment. Current unity protections must not be weakened to make the baseline look worse.

## Ozone ideas to borrow and ideas to defer

Borrow the ease of comparing two states, clear bypass, editable assistive suggestions and consequence-focused listening. Keep Removed prominent: that part of the restoration workflow is already here.

Defer a mastering chain, stereo widening, exciters, music-style tonal targets and aggressive limiting. Loudness/true-peak measurement could eventually be useful as a handoff check, but applying a publication target requires a defined final mix and delivery destination. Do not normalize each stem independently to an assumed podcast LUFS number. Codec preview is similarly premature while the product delivers editable WAV tracks.

Defer transcripts, filler/silence cutting, cloud processing, plugin hosting, live DAW plugins, automatic alignment and full DAW interchange until a specific owner workflow justifies them. They add separate privacy, timing, platform or editing contracts. None is necessary to validate the first three opportunities.

## Suggested next decision

Discuss a small discovery batch: **representative-example guidance, two sound-setting snapshots, and a lightweight listening-navigation prototype**. Validate them on two or three contrasting owner sessions before writing implementation tickets. Keep the navigation prototype optional because it revisits a previously simplified workflow.

Useful questions for that discussion: Where is most time currently lost—finding examples, repairing a missed voice, deciding how aggressive to be, or exporting? How often do microphone relationships change during a session? Which editor receives the stems? Is a small amount of residual overlap bleed preferable to any separation artifacts? Answers should change this ranking.

## Research limits

Official documentation was checked on 2026-10-22. No proprietary binaries, installed Ozone/RX sessions, audio examples, DAW round trips, user-study timings or algorithm outputs were tested. No implementation or dependency was selected, and no backlog item was claimed. Estimates remain qualitative; the current source was assessed through project documentation rather than a fresh code audit. Recommendations are proposals, and vendor feature descriptions are not comparative sound-quality evidence.
