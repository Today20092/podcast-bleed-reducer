# Signing portable updates

CleanSlate updates use a pinned Minisign public key. The private key belongs to
the release owner, outside this repository and outside ordinary CI jobs. Purchase
or sponsorship is never required to check for or install updates.

## First trusted build

1. Install the official [Minisign](https://jedisct1.github.io/minisign/) utility.
2. In your own terminal, run `minisign -G -p CleanSlate.pub -s CleanSlate.key`
   from a secure directory outside the checkout. Choose a strong password when
   prompted. Do not paste the password or private key into chat, shell arguments,
   GitHub issues or source files. Back up the encrypted private key and password
   separately; losing either prevents signing future releases with this identity.
3. Copy **only** the base64 public-key line from `CleanSlate.pub` into
   `assets/update-public-key.txt`. Review that public-only change and rebuild.
   An empty file disables automatic installation; it never means trust all keys.
4. Distribute that reviewed bootstrap build through the established release
   procedure. Existing 1.0.0 has no updater and needs this one manual replacement.

Keep the private key offline except while signing reviewed release descriptors.
The release workflow does not create keys, grant an entitlement, publish a release,
or choose a trust root dynamically. A future rotation requires a release trusted
by the existing key; replacing unsigned metadata cannot rotate trust.

## Sign reviewed artifacts

Build and check all native packages first. For Windows, the packaging script
emits a descriptor binding the complete ZIP hash, inner manifest, version,
platform and resource budgets. Review its source commit and package hashes.
Use Minisign to sign the exact descriptor bytes, retaining its `.minisig` file.
Use `scripts/update-metadata.ps1 -Package <checked-directory> -Archive <checked-zip>
-Minisign <trusted-local-minisign.exe> -SecretKey <private-key-path>` in your own
terminal. It prompts locally for the password and verifies the result against the
pinned public key. The descriptor is `<archive>.update.json`; its signature is
`<archive>.update.json.minisig`. Upload both alongside the unchanged checked ZIP
through the deliberate draft release procedure, then verify the downloaded
descriptor/signature with `minisign -V -m <descriptor> -P <pinned-public-key>`.
Check that the descriptor hash/size match the downloaded ZIP before publication.
Unsigned draft packages remain manual-download only. Never sign unreviewed or
replaced artifacts. Do not run signing with a production private key in CI.

The owner generated the bootstrap key on 2026-10-23; only its public half is in
`assets/update-public-key.txt`. Password entry and private-key backup stay outside
the app, repository and chat. No release was published by setting up this key.

## Disposable acceptance builds

`scripts/gui-dev.ps1 -BuildOnly -UpdaterFixture` uses the explicit
`updater-test-key` Cargo feature. It pins only the public test fixture key and
enables local package/failure-injection routes. The feature causes a compile error
in release profiles; never distribute a fixture executable. The ordinary GUI
build has no fixture overrides and embeds the owner public key.

`scripts/test-updater-package.ps1` accepts preserved old/new fixture executables,
an existing runtime DLL package, a trusted local Minisign executable and an
external **test-only** secret key. It creates unique disposable package roots,
detects a newer release from synthetic GitHub API metadata, verifies the signed
complete package, and exercises restart, startup failures, interrupted swap,
manual recovery, mixed folders, corruption and locks. Its receipt records hashes
and paths. It never uses owner recordings or a production signing key.

Checksums downloaded beside the ZIP detect corruption; only a signature checked
against the pinned bootstrap key authenticates the release publisher. Missing
or invalid signatures disable automatic installation and leave manual download
available. Test keys and synthetic packages are not production trust roots.
