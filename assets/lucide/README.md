# Lucide outline assets

Source: https://github.com/lucide-icons/lucide

Pinned upstream revision: `20aef31db93fe4d9f398dbb15b120d99c48eef95`.

The nine SVG files are unchanged upstream assets from `icons/`, retrieved on 2026-10-21 and 2026-10-22. They use a 24×24 viewBox, no fill, a two-unit stroke and rounded caps/joins. CleanSlate embeds only this subset, renders it through Iced's native SVG widget and supplies theme/state tint. The meter-placement marker is separate application artwork. No JavaScript, font, CDN or runtime download is used.

`LICENSE` retains the complete upstream ISC notice and the MIT notice for Feather-derived icons. The application embeds this text in About; packaging copies it to `licenses/lucide.txt` and lists it in `THIRD-PARTY-NOTICES.txt`.

Mappings: `arrow-right-to-line` → Catch up; `lock-keyhole` / `lock-keyhole-open` → Lock; `fold-horizontal` → Center playhead; `audio-lines` → Meters; `skip-back` → Previous; `play` / `pause` → transport (Pause only for an active reference loop); `square` → Stop. Existing shortcuts, selected state and command semantics are retained; icon-only controls retain descriptive hints.
