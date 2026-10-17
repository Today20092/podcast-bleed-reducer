# Embedded assets

## Silero VAD model

`silero_vad.onnx` is the Silero VAD v5 model copied from the validated Python
reference environment. It is distributed by the
[`snakers4/silero-vad`](https://github.com/snakers4/silero-vad) project under
the MIT license.

- Size: 2,327,524 bytes
- SHA-256: `1A153A22F4509E292A94E67D6F9B85E8DEB25B4988682B7E174C65279D8788E3`

## CleanSlate identity

The owner approved **Open channel** on 2026-10-17: a mint audio signal inside an open C, on a charcoal tile. Mint is `#9BE2BA`; charcoal is `#181B24`.

- `cleanslate.svg` is the editable icon master and Linux package icon.
- `cleanslate.ico` embeds nine Windows sizes (16–256px) in the executable.
- `cleanslate.rgba` is 64×64, 8-bit RGBA in sRGB, embedded in the Iced window without a runtime decoder or external file lookup.
- `cleanslate.icns` provides 16–1024px images for macOS app bundles.
- `cleanslate.png` is the 1024px transparent-corner master export.
- `github-avatar.png` is a 512px square export with solid charcoal corners for square/circular crops. It does not replace the repository owner's personal avatar.
- `cleanslate-logo-light.svg` and `cleanslate-logo-dark.svg` are outlined wordmarks for pale and dark backgrounds respectively. Segoe UI Bold lettering has been converted to paths; rendering needs no installed font.

Use `node scripts/generate-icons.mjs` with ImageMagick installed to regenerate icon exports after editing the master SVG. The generator checks ICO sizes, raw pixel length and ICNS PNG dimensions. ImageMagick and Node are authoring tools only, not application/build dependencies. Preserve proportions and leave at least one stroke width of clear space around standalone logos.
