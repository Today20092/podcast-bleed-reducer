# FUTO payment research and CleanSlate options

Date: 2026-10-23. Ticket: 72. Owner direction: an optional one-time purchase, with the complete app remaining free to use, using purchase wording rather than donation wording. This note records research and copy recommendations; no selected price, payment integration, licensing change or release authorization.

## Likely source and findings

The owner's description closely matches FUTO's [Shifting Perspectives on Purchasing Open Source Software](https://futo.tech/blog/shifting-perspectives-on-open-source), published April 15, 2025 and labeled Opinion.

FUTO reports over 750 anonymous responses, approximately 110 developers and 600 consumers, recruited for a month through its YouTube channel, website, word of mouth and SXSW-related outreach. More than 94% expressed willingness to pay; more than 80% of surveyed customers favored pay-what-you-want and/or one-time purchases. Its hypothetical valuable-tool price-band answers yielded an approximately US$18 mean using band lower bounds; frequent donors averaged about US$27 and represented roughly 10% of respondents. Leading motivations were roadmap clarity (68.3%), usability (63.5%) and funding transparency (59.8%). Reported deterrents included affordability, abandonment and purchasing friction. FUTO describes purchases as supporting present value rather than promising future features, and separately promotes Source First licensing. [Source](https://futo.tech/blog/shifting-perspectives-on-open-source).

Interpretation: this is self-selected stated preference, not observed purchases or a representative sample of podcast editors. The combined one-time/PWYW result does not establish demand for a mandatory unlock. The band-based mean is not a tested optimum. The article does not establish CleanSlate's conversion rate, sustainable revenue or customer time savings.

## Options for CleanSlate

These are recommendations from the evidence, not findings of FUTO's survey:

| Model | Concrete behavior | Implication |
| --- | --- | --- |
| Optional one-time purchase (owner-selected model) | Full existing app remains free to use; Ko-fi provides the purchase route. GitHub Sponsors is an additional support route. | No activation system. Treat US$18–20 as an initial suggested amount to evaluate, not an approved price. |
| Paid official downloads | A one-time purchase provides the official packaged app; source remains available under its license. | Requires a storefront, delivery/recovery policy and a clear decision about existing public release assets and updater access. |
| Mandatory in-app unlock | Payment enables app features or continued use. | Adds purchase verification, offline activation and recovery/support obligations. Do not infer authorization from the owner's tentative one-time-payment idea. |

Recommended next scope: implement the owner-selected optional one-time purchase copy and verified destinations, with GitHub Sponsors alongside Ko-fi. Paid-only downloads and mandatory unlocks remain unselected alternatives. Do not assert a measured number of hours saved or promise lifetime updates/support.

## Concrete copy and product implications

The following are CleanSlate recommendations informed by the research, not a FUTO-designed checkout or entitlement policy:

- Primary action: **Purchase CleanSlate**. Adjacent explanation: **Make an optional one-time payment for CleanSlate as it works today. The complete app is free to use. Your purchase helps sustain maintenance and development; future features are not included as promises.** The payment destination must actually offer a one-time payment; GitHub Sponsors should have separate truthful wording matching its configured options.
- Roadmap: link a short public page that distinguishes shipped features, active work and ideas under investigation. Describe the existing offline import, speaker examples, analysis, gain editing and export workflow. Do not sell ticket 67's plugin concept or ticket 73's updater as delivered features.
- Usability: make the free-use path clear at the purchase prompt, open payments in the user's browser only on explicit action, and avoid interrupting editing or exports. The offline workflow should work without an account, purchase verification or network access.
- Funding transparency: state the intended uses of revenue in plain language, such as maintaining CleanSlate, fixing defects and improving usability. Obtain owner confirmation before publishing spending allocations, targets or claims about how money has already been spent. A short dated progress note can explain completed work without implying guaranteed delivery dates.
- Affordability and purchase friction: show the amount and currency before checkout when configured, describe the payment as optional, and keep the purchase link easy to find again in Help/About. Do not claim platform fees, tax treatment, refunds or payment methods without checking the actual configured checkout.
- Continuity: keep project status and maintenance communication visible. Report material pauses honestly; no promise of indefinite support or automatic updates is established by a purchase.

These recommendations apply the source's themes to the owner's chosen model. They do not require adopting FUTO's license, authentication, telemetry, store or billing infrastructure.

## Existing project constraints

The repository's [MIT license](../../LICENSE) expressly permits selling copies as well as copying, modification and redistribution, conditional on retaining its notices. Charging for an official build therefore does not make the existing MIT code exclusive or prevent redistribution. FUTO's licensing proposal is a separate decision; this note does not recommend adopting it or determine rights over future contributions.

The [stable compatibility policy](../releasing.md#versioning-and-stability) covers existing saved projects, speaker examples, manual edits and preferences. Ticket 73's updater discovery must explicitly account for any future change in package access or purchase entitlements. Preserve existing projects and published release identities; do not quietly add a payment requirement through an update.

## Decisions still needed

Recipient follow-up: the owner supplied `https://ko-fi.com/ayoubab1`; existing repository funding metadata identifies the same Ko-fi account and GitHub `Today20092`. The public `https://github.com/sponsors/Today20092` page was verified and offers one-time/monthly support. Ko-fi is primary. The web tool could not retrieve its page; checkout has not been exercised. This resolves the recipient/ordering question below at the identity level, without claiming a tested payment.

- Exact owner-controlled Ko-fi and GitHub Sponsors URLs, and their ordering. Naming the services does not establish the recipient accounts; no links should be guessed.
- The selected amount/currency; optional payment and continued free full-app use are confirmed.
- Any update/support terms beyond supporting the app as it exists today. No future-feature or lifetime-support promise is selected.
- If testing suggested support amounts, agree how results will be assessed without silently adding analytics or collecting recording data.

Verification: read the primary FUTO article, local LICENSE and release policy. No accounts, checkout, platform fees, tax handling, purchasing flow or conversion results were checked; no implementation or public copy was changed.
