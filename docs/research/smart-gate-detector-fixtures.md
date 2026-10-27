# Small licensed detector fixture candidates

Current status: the early acquisition failures below are historical. The final
Commons follow-up documents policy-compliant successful downloads, exact hashes,
private decoding and 225 actual detector runs. Freesound's GET failure remains
unresolved; no Freesound audio is claimed to have been acquired.

2026-10-26. Metadata-only follow-up for [ticket 92](../../.scratch/offline-supervised-editor/issues/92-single-input-detector-feasibility.md). No audio, weights, packages or archives were downloaded; no recordings were auditioned, relabeled or modified. This note supplies acquisition candidates, not additional detector results or ticket completion.

## Decision

Four individual recordings offer a bounded next experiment: an interview laugh, recorded whispering, a short spoken affirmative and natural overlapping pub voices. Their current first-party pages identify the uploader and a commercially reusable Creative Commons license. Official public HQ MP3 previews are linked in each page's `data-static-file-url`; unauthenticated HEAD requests returned HTTP 200 and the byte counts below. Original-file links say **Login to download** and are excluded from the ungated acquisition recommendation. Preview encodings are distinct artifacts: original WAV/M4A sample-rate, channel and duration metadata must not be silently applied to the MP3 bytes.

These are **uploader-described human recordings**, not independently validated FSD50K annotations or sample-level VAD truth. Record the labels as `uploader_clip_description`, and preserve uncertainty about exact speech/laugh/overlap spans. Do not turn an entire clip into a wanted-speech mask. Their value is broader category coverage without artificially attenuating, cropping or mixing existing speech. Listening, natural quiet starts/tails and controlled two-person overlap remain unqualified.

## Recommended acquisition manifest

All sizes below are live HEAD `Content-Length` values, not downloaded-file hashes. Every selected preview is below 5,000,000 bytes; the four total **3,470,008 bytes**. Recheck size and license before later acquisition, stream with a hard 5 MB cap, then hash the actual acquired bytes.

| Candidate and first-party record | Human description / original metadata | Clip license | Official individual preview; verified bytes |
| --- | --- | --- | --- |
| Natural-context laughter: **185227**, `nervous laugh.wav`, **tim.kahn**; [record](https://freesound.org/people/tim.kahn/sounds/185227/) | Uploader describes nervous laughter at the start of an interview, recorded with a lavalier microphone. Original: 3.322 s, mono, 48 kHz, 24-bit WAV, 467.9 KB. No exact laughter boundaries or independent audit. | [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/) | [185227_7037-hq.mp3](https://cdn.freesound.org/previews/185/185227_7037-hq.mp3), **73,248 bytes** |
| Recorded whispering: **513410**, `Woman Whispering ASMR 01.wav`, **vero.marengere**; [record](https://freesound.org/people/vero.marengere/sounds/513410/) | Uploader describes a woman whispering, recorded using a Blue Yeti Pro. Original: 12 s, stereo, 48 kHz, 24-bit WAV, 3.3 MB. This is whispering, not independently established naturally quiet voiced speech; ASMR performance is not spontaneous conversation. | [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/) | [513410_6605732-hq.mp3](https://cdn.freesound.org/previews/513/513410_6605732-hq.mp3), **199,656 bytes** |
| Short utterance: **591283**, `yes.wav`, **awchacon**; [record](https://freesound.org/people/awchacon/sounds/591283/) | Uploader describes a man saying yes. Original: 1.524 s, stereo, 44.1 kHz, 16-bit WAV, 262.7 KB. Clip duration does not establish utterance duration or a spontaneous interjection. | [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/) | [591283_13322965-hq.mp3](https://cdn.freesound.org/previews/591/591283_13322965-hq.mp3), **37,648 bytes** |
| Natural overlap/noisy speech: **690597**, `English pub ambiance.m4a`, **Talitha5**; [record](https://freesound.org/people/Talitha5/sounds/690597/) | Uploader describes English pub voices overlapping with dishes and silverware. Original: 140.223 s, mono, 48 kHz M4A, 1.1 MB. Crowd ambience tests speech response in noise; it does not establish a clean two-person overlap mask or microphone ownership. | [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/) | [690597_4458028-hq.mp3](https://cdn.freesound.org/previews/690/690597_4458028-hq.mp3), **3,159,456 bytes** |

Attribution record for 185227: `nervous laugh.wav by tim.kahn — https://freesound.org/people/tim.kahn/sounds/185227/ — CC BY 4.0, https://creativecommons.org/licenses/by/4.0/`. Later decoding/downmixing/resampling or excerpting must be stated as changes. For CC0 clips retain title, uploader, source and license anyway so their provenance is reproducible. No claim is made that a public preview has identical encoding or boundaries to a corpus derivative.

Two additional CC0 alternatives were resolved, but are not necessary for the minimum four-clip set:

- **93775**, `Female Laugh.wav`, **rdneubauer**: [record](https://freesound.org/people/rdneubauer/sounds/93775/), [official preview](https://cdn.freesound.org/previews/93/93775_1441763-hq.mp3), HTTP 200, **78,370 bytes**. Uploader describes female laughter recorded with two condenser microphones; original 3.286 s. Spontaneity is unspecified.
- **446580**, `Whispers`, **Nicolas4677**: [record](https://freesound.org/people/Nicolas4677/sounds/446580/), [official preview](https://cdn.freesound.org/previews/446/446580_9134393-hq.mp3), HTTP 200, **68,047 bytes**. Uploader describes whispering; original 3.5 s. No transcription, speaker count, quiet-voiced-speech truth or processing provenance is supplied.

## FSD50K: better annotation provenance, current retrieval barrier

[Official FSD50K v1.0](https://zenodo.org/records/4060432) describes human-annotated sound events with weak clip-level labels. Its audio licenses vary per clip, including noncommercial and Sampling+ material, so dataset CC BY attribution is insufficient to approve any particular recording. The official record identifies `dev_clips_info_FSD50K.json` / `eval_clips_info_FSD50K.json` as mappings containing uploader, title, description and clip license; `dev.csv` / `eval.csv` map Freesound IDs to ground-truth labels. Collection metadata retains some finer categories omitted from the 200-class release. The eval split is more exhaustively annotated within that vocabulary, but still supplies no speech-boundary masks.

The record lists [metadata ZIP](https://zenodo.org/records/4060432/files/FSD50K.metadata.zip) at 6.7 MB and [ground-truth ZIP](https://zenodo.org/records/4060432/files/FSD50K.ground_truth.zip) at 334.7 KB. Direct metadata and API-content requests returned HTTP 403 here; the companion release page timed out. No audio archives or mirrors were tried. Consequently this investigation establishes **no FSD50K membership, validated category, annotator agreement or split** for the four candidates above. The concrete next step, when the official metadata becomes accessible, is a bounded in-memory ZIP join of category rows to per-clip license/uploader, rejecting NC, Sampling+ and unclear entries, then resolving individual public clip URLs. Do not fetch the multi-GB audio archives merely to discover IDs.

## Integration recommendation and remaining truth gaps

Keep these as a separate public-category smoke-test manifest, with fields for Freesound ID, title, uploader, source URL, preview URL, license URL, acquisition hash/bytes, provenance level, conversion recipe and explicit limitations. A later authorized downloader should verify its byte cap before decoding. No download helper or dependency is needed for this research note.

The existing feasibility CLI pins one owner project and its source bindings; these external clips cannot be passed off as its canonical references. A future small fixture extension can reuse the detector while recording their separate provenance. Whole-clip probability/event timing and block/reset invariance may be measured; preservation accuracy requires reviewed time labels and listening. Do not derive universal pass/fail probability thresholds from these four recordings. Whispering does not close naturally quiet voiced-speech acceptance; a spoken affirmative does not close spontaneous interjection acceptance; pub crowd noise does not close clean natural-overlap acceptance. None establishes speaker identity or isolation.

## Actual GET correction and Commons alternatives

Later 2026-10-26 acquisition checks supersede the earlier HEAD-only availability inference. The root agent's actual GET of Freesound preview 185227 returned HTTP 403 with an empty body, despite HEAD 200. One explicit user-agent/source-referer diagnostic also returned 403; no audio file was created. This research continuation made **no further Freesound CDN requests**, login attempts or mirror requests. The Freesound candidates remain licensing/category leads, not demonstrated working downloads.

Scope clarification: the final paragraph above describes **broader 85/93 preservation and listening qualification**, not additional requirements imposed on 92. Once actually exercised, representative human-recorded whispering, a short affirmative, laughter and naturally overlapping voices can meet 92's bounded category-smoke coverage with provenance and available-evidence gaps stated. Ticket 92 does not specifically demand spontaneous interjections, quiet voiced phonation rather than whispering, or clean two-person time masks. Clip-level descriptions cannot establish frame-level quality, but their weakness does not by itself disqualify a feasibility smoke case.

The following alternatives are documented on Wikimedia Commons' own file records. The three preferred files total **438,865 bytes** according to repository metadata. No full audio was downloaded or listened to.

| Representative category | First-party metadata, author and license | Exact candidate file endpoint; upstream hash |
| --- | --- | --- |
| Actual whispered word | [Whispering-example.ogg](https://commons.wikimedia.org/wiki/File:Whispering-example.ogg), **Mx. Granger**, own work, 2014-11-15. The author describes a male American English speaker whispering the word whispering. **CC0 1.0**. Repository metadata: **30,283 bytes**, **2.56580498866213 s**, Ogg Vorbis. This describes the manner of speech; it is not merely pronunciation of the word whisper at ordinary volume. | `https://upload.wikimedia.org/wikipedia/commons/e/e8/Whispering-example.ogg`; repository SHA-1 **cc3b98f93353cf6ed8bb453669987afe7dd748bd**. |
| Short affirmative | [En-nz-yes.ogg](https://commons.wikimedia.org/wiki/File:En-nz-yes.ogg), uploader/author **Mattadjwic**; caption credits recording to **Codymator Adjwic**, 2024-07-28. New Zealand English pronunciation of yes, own work. **CC BY 4.0**. Repository metadata: **12,589 bytes**, **0.619705215419501 s**, Ogg Vorbis. Elicited dictionary utterance, not a claimed spontaneous conversational interruption. | `https://upload.wikimedia.org/wikipedia/commons/2/27/En-nz-yes.ogg`; repository SHA-1 **4b92cad9d19926659e4f1470341cc6ca51abf00e**. |
| Human laughter | [Laughter.ogg](https://commons.wikimedia.org/wiki/File:Laughter.ogg), author **ezwa**, uploaded/transferred by **Fæ**, source PDSounds record 137. Description attributes the laugh to amusement while recording a story. Commons records the author's **public-domain dedication**, rather than a CC0 license. Repository metadata: **395,993 bytes**, **16.4832653061224 s**, Ogg Vorbis; source was MP3, so not pristine lossless capture. | `https://upload.wikimedia.org/wikipedia/commons/c/c6/Laughter.ogg`; repository SHA-1 **e46f1ee739d737af4efa4ddf639eb94bc00e7f08**. |

For En-nz-yes attribution retain `En-nz-yes.ogg — Mattadjwic (caption: Codymator Adjwic), https://commons.wikimedia.org/wiki/File:En-nz-yes.ogg — CC BY 4.0, https://creativecommons.org/licenses/by/4.0/`, plus a statement of any later decoding, resampling, downmixing or trimming. Keep the other authors/source links in the manifest too. Upstream SHA-1 is provenance metadata, not a locally verified hash; compute SHA-256 after any successful later acquisition.

### Bounded actual GET observations

One unauthenticated `Range: bytes=0-63` GET was attempted per preferred endpoint, with response-headers-first streaming, a 2,000,000-byte Content-Length ceiling, a maximum 64-byte body read and immediate disposal. **All three returned HTTP 403, `text/plain`, Content-Length 126; zero audio bytes were read.** The earlier US affirmative alternative below returned the same result. Direct Commons page/API access from the shell was also refused, although the web research tool could read the first-party file pages. This identifies an access barrier in this execution environment; it does not establish that the files are absent or globally unavailable. There is no working GET result, magic-byte validation, decode result, local hash or probability result to report. No authentication, CAPTCHA bypass, alternative-host routing or full download was attempted.

An optional short-word source is [En-us-yes.ogg](https://commons.wikimedia.org/wiki/File:En-us-yes.ogg), **Dvortygirl**, own work, approximately 0.8 s/16 KB. It offers **CC BY-SA 3.0 Unported** among several licenses. Attempted endpoint: `https://upload.wikimedia.org/wikipedia/commons/b/b1/En-us-yes.ogg`. If that option were chosen, retain attribution, source and license links, identify changes, and follow the stated same-or-compatible-license condition when distributing adaptations. Keep audio/derivatives outside this MIT source repository with their own license record. The CC BY 4.0 New Zealand recording above avoids needing this optional share-alike source.

No new pub/overlap download was pursued after the access failures. The existing owner active-speaker overlap reference remains a separate source-bound overlap case; do not replace its truth with an invented pub mask. The best next action is to resolve the demonstrated public-file GET access barrier through the supported environment/network path, then acquire only the preferred tiny originals under an explicit byte cap and verify upstream size/hash. The content and license candidates are concrete; successful retrieval and category execution remain pending.

## Successful Commons acquisition and bounded execution

The coordinator resolved Commons retrieval using its documented
[User-Agent policy](https://foundation.wikimedia.org/wiki/Policy:Wikimedia_Foundation_User-Agent_Policy):
the honest application identifier was
`CleanSlateFixtureBot/0.0.1 (https://github.com/Today20092/podcast-bleed-reducer)`.
This supplied the requested bot/contact identity; it did not impersonate a
browser, change credentials or route around an authentication boundary. The
three original endpoints above then downloaded successfully, matching their exact
438,865-byte combined upstream size and individual SHA-1 values. Actual SHA256
values, authors, licenses, URLs and attribution are retained in the repository's
[metadata-only manifest](../../plugins/gate-detector/fixtures/public-categories-v1.json).
Laughter uses the author's public-domain dedication, **not CC0**; the affirmative
retains its CC BY 4.0 attribution and conversion statement. No audio is committed
or distributed with this MIT source repository.

Original OGG and decoded PCM files remain private in
`%LOCALAPPDATA%/CleanSlate/GateDetector92/commons-category-fixtures-550d9196d0c24223b3309600bdc89bda/`,
alongside the acquisition manifest and per-clip decode guard receipts. FFmpeg
9.0.2, one decode/filter thread, selected the first decoded channel and at most
the first 15 seconds **before inference**, resampling to 48 kHz mono float32.
No normalization, probability-based trim or model-driven selection occurred.
The exact launcher hash and command arguments are in the manifest; each decoded
PCM byte count/SHA256 is verified by the harness before model preparation.

Natural overlap is a separate existing owner source: the exact pinned
`test-audio/quick-test-10min/261005-quick-test.cleanslate` hash
`20bb912825964a2ba9b717525477f6af365abb3e434084cf7836e6b5725cbd98`
contains saved reference **7**, active speakers **[1,2]**, range
`[5342409605,5417660656]` at 28,224,000 ticks/second. All three complete source
hashes and reference bindings were verified. The full 2.666208-second range was
read from unchanged track 1, starting at 189.286055 seconds; its float32 excerpt
SHA256 is `8ab04644b26a3e8534fa06d0409f1a24bbb8b398d715e6169ede6c4869686cc5`.
This is a human source-bound development overlap mask, not an independent
sample-level speech audit. Original long-project reference 13 is solo [3]; it
was not used as evidence of natural overlap.

The optional category harness executed 225 runs (15 cases × three rates × five
blocks), retaining all probabilities and exact block/reset invariance. See the
[measured decision](smart-gate-detector-92.md#recorded-category-follow-up) for
raw-response summaries, resources and receipts. All 165 earlier probability
arrays remained exactly unchanged. A same-size one-byte PCM mutation is rejected
with `public fixture PCM identity mismatch`, before model/report work, in a
public CLI red/green regression.

This closes the bounded 92 category-execution gap: synthetic silence/noise,
normal recorded speech, actual recorded whispering, a short spoken affirmative,
human laughter and recorded natural overlap have all been exercised. It does not
close per-frame wanted-speech preservation, natural quiet starts/tails,
spontaneous conversation or 85/93 listening qualification. Whisper and laughter
responses are low across much of those whole clips: that is a **strong
preservation risk**, not a false-negative percentage without time masks. A future
93 experimental prototype must start bypassed and require explicit arming;
these results do not justify enabling production attenuation or tuning away the
room disagreement. No model or detector algorithm changed.
