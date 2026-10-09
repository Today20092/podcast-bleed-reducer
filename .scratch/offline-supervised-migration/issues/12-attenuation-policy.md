# Decide how automatic attenuation handles uncertainty

Type: grilling
Labels: wayfinder:grilling
Status: resolved
Parent: ../map.md
Blocked by: none

## Answer

2026-10-09: Owner confirmed the [consolidated design](../gui-design-summary.md). Earlier questions and recommendations below are interview history; the consolidated design is the final decision. Acoustic-first evaluation with optional Nemotron and less conservative attenuation is folded into existing [build ticket 14](../../offline-supervised-editor/issues/14-independent-release-evaluation.md). This authorizes an evaluation direction, not removal of release gates or a claim of acoustic-only quality. No implementation has been performed for this decision.

## Question

Does the owner want better bleed reduction while preserving the current wanted-speech guarantee, or a deliberate tradeoff allowing uncertain speech to be attenuated?

## Owner feedback

The owner expects supervised examples to teach where speakers are absent and thinks the system need not be so conservative. They request an understandable Mermaid explanation of the mathematics and workflow.

## Current contract and recommendation

2026-10-09 owner decision: allow automatic attenuation of uncertain regions. The owner reports visually obvious speaker-absent spans receiving insufficient attenuation. This reopens the automatic uncertainty-preservation rule. Diagnose the failing mechanism as part of the eventual implementation, but do not treat diagnosis-only as the owner's selected outcome. Determine reduction behavior, trusted-label protection, missing-evidence behavior and revised release metrics in subsequent rounds. No implementation or numerical thresholds are authorized yet.

Solo/overlap/room labels already train independent per-track logistic ownership models using fixed neural speech/activity and acoustic bleed features. The models estimate local ownership beyond labeled intervals. Ambiguous identity, missing evidence and poor acoustic fits preserve uncertain regions; this is distinct from the supervised learning itself. See [algorithm diagram](../gui-feedback.md).

The owner has selected less conservative automatic attenuation. Diagnose why absent-speaker spans remain preserved while evaluating the revised policy. Update the accepted specification and release evidence only after shared-understanding confirmation; retain explicit human speech-label protection and separate manual edits from automatic quality metrics. Do not silently turn missing model evidence into silence or guess speaker identity.

## Later decisions and proposed acceptance questions

2026-10-09 third round: evaluate acoustic-first supervised analysis with optional Nemotron before committing to removal. Train from a few clear representative reference intervals across the podcast, not exhaustive annotation. Use adjustable attenuation with fades rather than complete muting. See [consolidated design](../gui-design-summary.md). Automatic policy revision and real-recording qualification remain pending.

2026-10-09 second round: prioritize effective main attenuation; do not require the owner to focus on uncertainty review now. Preserve explicitly labeled speakers. Attempt acoustically supported attenuation when model identity/evidence is unavailable, rather than automatically preserving every such span. The owner questions whether Nemotron is necessary; removing or making it optional remains unresolved. This reopens required-Nemotron and missing-evidence rules from decision 03 and the accepted specification, but does not yet revise either. Acoustic-only feasibility must be separated from demonstrated quality on the real recording. Reduction amount and label coverage expectations remain open.

What specifically sounds insufficiently reduced, and where? Is the problem mapping, references, poor bleed fit, probability thresholds, temporal regions or fade settings? Which quiet speech, interjections and overlap must remain untouched? What loss, if any, is acceptable? Freeze tuning separately from held-out evaluation and retain baseline comparison and listening approval. No numerical threshold or new classifier is selected by this ticket.
