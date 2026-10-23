# Make project sponsorship visible in the GUI and repository

Status: claimed
Assignee: Codex
Implementation readiness: needs-info (savings stage; optional purchase stage accepted)
Type: feature
Blocked by: none
Related: 63, 71

## Owner request — 2026-10-22

Create a clear, compelling way to support CleanSlate through the application, README and GitHub project. Explain that it is free and open source and sustained by users. Offer the owner's Ko-fi, Stripe and/or GitHub Sponsors destinations. Include an eventual explanation of time saved compared with manually reducing podcast microphone bleed.

The owner requested work on this ticket on 2026-10-23. Preparation is claimed; implementation readiness still needs verified recipient links. This does not authorize account creation, payment collection, or publication of unverified recipient links. No dependency on the updater ticket.

## Scope and proposed experience

- Add an accessible Support CleanSlate action using the existing registered-action/menu conventions, reachable after startup from Help/About. Keep a concise, visible support entry in the application without competing with editing controls.
- Show a brief startup support message with an immediate Continue action and a persistent Don't show again preference. Start with first launch and explicit reopening from Help; avoid a repeated modal on every launch. Proposed copy: “CleanSlate is free and open source. Support from users helps fund development and maintenance. If it saves you time, please consider a donation or becoming a sponsor.” Final wording and presentation belong to GUI review.
- Present verified available choices with clear one-time donation or recurring sponsorship labels. Open the selected provider's hosted page in the normal browser only after a user action; keep payment details outside the app. Omit unavailable providers rather than shipping placeholders or broken buttons. Supporting the project never unlocks core features or interrupts analysis, playback, saving or export.
- Add a concise README support section and configure GitHub's repository funding entry using the same verified destinations. Include relevant About/help/project documentation so users can find support again. Explain what support funds without promising delivery dates or implying a tax deduction.
- Include time-saved feedback in this ticket as a later delivery stage. Prefer factual local usage totals initially, such as recording duration successfully processed. Distinguish recording duration, processing elapsed time and estimated manual work avoided. Do not equate a two-hour recording with two hours saved.
- Before showing a savings estimate, define the manual-editing baseline and formula from measured comparison or a user-supplied estimate. Show assumptions and label the result “Estimated”; use a range if appropriate. Exclude failed/cancelled jobs and avoid inflating totals through repeated analysis or exports of the same session. Define reset, project reopening and deduplication behavior. Keep these counters local; no new telemetry is needed. If the baseline is unavailable, show only factual totals and retain the estimate as explicitly unfinished scope.

## Information needed before implementation readiness

- Exact owner-controlled Ko-fi, Stripe hosted payment/donation and GitHub Sponsors URLs, which are active, and preferred primary destination. These are not present in the request; do not invent account handles or checkout links.
- For the savings stage: evidence or owner-supplied manual-editing assumptions, calculation and counting/reset rules. The basic support entry need not wait for this stage once recipient links are supplied; completion must distinguish stages rather than silently dropping savings feedback.

## Acceptance

- [x] Verified destinations are recorded here; GUI, README, GitHub funding configuration and relevant help use consistent links and recipient identity. Owner-supplied Ko-fi plus existing funding metadata; public Sponsors page verified. Checkout not exercised.
- [ ] Startup copy communicates free/open-source status and voluntary user support; Continue, dismissal, persisted suppression and Help reopening work with keyboard navigation, compact layouts and supported themes.
- [x] Unavailable/offline/browser-launch failure states leave the editor usable with clear feedback. No startup network request is required to show the message, and no payment data is handled by CleanSlate. Browser failure injection passed; real checkout remains untested.
- [x] No donation gate or repeated prompt interrupts working sessions. The registered support action and any shortcut follow the existing command contract. Full-app free use and optional purchase wording are the accepted model.
- [ ] Test preference persistence and the meaningful support-flow failure paths. Check destination consistency without making a payment; review rendered README/funding configuration and the GUI through the standard GUI development workflow.
- [ ] Savings-stage baseline/formula, local storage and counting rules are documented and covered by small tests for repeat processing, failed/cancelled jobs, reopening and reset. Unsupported numeric savings claims are absent.
- [ ] Record implementation, checks and remaining limits here; do not mark the entire ticket done while its savings stage remains outstanding.

## Results

2026-10-23 owner acceptance: the development GUI was launched through the standard script using a separate project copy. The owner reviewed the purchase flow, accepted its appearance and requested a development commit. This accepts the shown UI; unperformed native matrix/checkout checks and the savings stage remain explicitly open.

2026-10-23: Optional purchase stage implemented and software-verified on development. Final `scripts/check.ps1 -Local` passed 359 tests (20 ignored), strict Clippy, formatting and script fixtures. Standard GUI BuildOnly and executable package self-check passed. The [delivery receipt](../../../docs/research/support-purchase-delivery-2026-10-23.md) records source/executable paths and hashes, preference/async-command fixes, destinations and remaining checks. Native visual/checkout review is unperformed; the savings stage is still unfinished. Keep this ticket claimed rather than marking the whole feature done.

2026-10-22: Created on owner request and linked from the active roadmap/index. Specification only; recipient verification, UI implementation and savings evidence are outstanding.

## Owner follow-up — 2026-10-23

- Work on this ticket is requested. Use Ko-fi and GitHub Sponsors as the intended support destinations; exact owner-controlled URLs and the primary destination are still needed. Stripe is not part of this requested first delivery.
- Research FUTO's writing/research about paying for open-source software as a baseline. The owner is considering a one-time-payment model; no price or mandatory payment requirement has been accepted.
- The original voluntary-donation/no-feature-gate brief is therefore provisional where it conflicts with this payment-model exploration. Do not ship purchase wording, a licensing gate, or unsupported time-saved claims before the model is settled.
- Research and preparation can continue independently of recipient links. Preserve the existing MIT license and public package identity; no account creation, checkout, payment collection or release publication is requested.

## Prepared implementation boundary — 2026-10-23

### Accepted model and destinations

The owner selected voluntary one-time support with the complete app remaining free, then explicitly requested **purchase** wording rather than donation wording. Use “Purchase CleanSlate” with optional-payment disclosure. Apply FUTO's practical recommendations through a visible public roadmap, usable nonblocking flow and truthful intended funding uses; no revenue totals, savings figures, future-feature promises or telemetry are authorized.

Primary destination: owner-supplied `https://ko-fi.com/ayoubab1`. Secondary: `https://github.com/sponsors/Today20092`, resolved from the existing `.github/FUNDING.yml` (`github: [Today20092]`, `ko_fi: ayoubab1`) and verified against the public Sponsors page, which offers one-time/monthly support. The normal profile is not substituted for checkout. Ko-fi's public page could not be retrieved by the web tool; the exact owner-supplied URL and existing funding configuration agree, but checkout operation has not been tested.

### Implementation in progress

The optional purchase dialog, first-launch dismissal preference, registered commands, Menu/About access, browser failure recovery, roadmap link and README/help copy are implemented in the working tree. Existing GitHub funding entries already match and require no edit. The shared command dispatch now returns its asynchronous task through mouse and keyboard callers so browser-launch results reach the dialog. Software checks and GUI build are underway. Time-saved counters/estimates remain unfinished scope; do not close the entire ticket.

The [FUTO payment-model research](../../../docs/research/futo-payment-model-2026-10-23.md) identifies the likely requested article: [Shifting Perspectives on Purchasing Open Source Software](https://futo.tech/blog/shifting-perspectives-on-open-source). Its self-selected survey supports investigating one-time/pay-what-you-want contributions, not a validated CleanSlate purchase price or conversion rate. The owner is being asked to choose voluntary support, paid official downloads or a mandatory unlock; no option is accepted yet. The latter two also affect ticket 73's future package access and entitlement policy.

Existing integration points were inspected: `src/shortcuts.rs` owns command definitions and optional bindings; `src/button_action.rs` requires registered actions or audited exceptions; `src/app.rs` owns menu/dialog input, `AppLink`, browser dispatch and startup preferences; `src/appearance.rs` merges preference JSON and saves through `project::atomic_save`. Reuse those boundaries rather than introducing a payment client or separate settings store.

Once the payment model and exact links are settled, the change must reach command registration/dispatch and hints, menu/About/first-launch presentation and keyboard focus, persisted startup dismissal, browser failure feedback, README, GitHub funding configuration and help. Test missing/invalid preferences, preservation of unrelated preferences, failed saves, repeat startup/dismissal, command contexts and browser failure. Review compact layouts and both themes through the documented GUI workflow.

The accepted optional-purchase decision above supersedes earlier pending-model wording. Time-saved estimates remain a separate unfinished stage; a software-payment survey supplies neither a manual-editing baseline nor deduplication/counting rules.
