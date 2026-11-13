# Tintsmith Design System

A **dark high-fantasy** design system for *Tintsmith* — a fictional game studio brand. It captures the mood of a AAA dark-fantasy RPG marketing world: near-black grounds lit by forged gold and blood crimson, engraved Roman-capital display type, literary serif prose, and weathered procedural textures.

> **Provenance & IP note.** The aesthetic *direction* was briefed against the visual genre of the CD PROJEKT RED *Witcher 3* website (`https://www.thewitcher.com/pl/en/witcher3`) — used only as a **mood reference** for the dark-fantasy category (dark theme `#090909`, gold accents, ornate serif type, evocative lore copy). **No Witcher/CD PROJEKT RED intellectual property is reproduced here** — no wolf medallion, no "Witcher" wordmark, no character art, no screenshots. The Tintsmith emblem, wordmark, the fictional game *Wild Reign*, and all copy are original.

---

## What's inside

| Area | Path | Notes |
|---|---|---|
| Global entry | `styles.css` | `@import` list only — consumers link this one file |
| Tokens | `tokens.json` | colors, typography, spacing, effects, base, fonts |
| Foundation cards | `guidelines/*.html` | Type / Colors / Spacing / Brand specimens (Design System tab) |
| Components | `components/src/<group>/` | `core`, `forms`, `navigation`, `feedback` |
| UI kit | `ui_kits/wild-reign/` | Interactive game-landing recreation |
| Brand assets | `assets/` | `tintsmith-mark.svg` emblem |
| Agent skill | `assets/notes/SKILL.from-standalone.md` | Claude Code–compatible skill front-matter |

### Components
- **core** — `Button`, `Badge`, `Card`, `Ornament`
- **forms** — `Input`, `Select`, `Checkbox`, `Switch`
- **navigation** — `Tabs`
- **feedback** — `Dialog`

Mount in card/kit HTML via `const { Button } = window.TintsmithDesignSystem_c946a4` after loading `components/bundle.js`.

---

## CONTENT FUNDAMENTALS — voice & tone

The voice is **epic, atmospheric, and weathered** — a narrator who has seen the realm and speaks plainly about its dangers.

- **Tone:** cinematic and grave, never jokey. Evokes destiny, contracts, blood, and consequence. Occasional dry, world-weary wit ("The world doesn't need a hero. It needs a professional.").
- **Person:** primarily **second person** to pull the player in ("Walk the path…", "Ready yourself for combat"). Slips to third person for lore/character description.
- **Casing:** Display headings are **engraved uppercase** (Cinzel) or Title Case. UI labels, eyebrows, and buttons are **UPPERCASE, heavily tracked** (0.18–0.24em). Body prose is sentence case.
- **Sentences:** confident, declarative, image-rich. Favor concrete nouns of the world (marsh, blade, raven, contract, realm) over generic marketing words.
- **Eyebrows / kickers:** short tracked phrases that frame a section ("Beyond the Game", "Join the Watch", "Chapter I · The Path").
- **Emoji:** **never.** This brand does not use emoji.
- **Punctuation:** em dashes for dramatic pause; the middle dot `·` as a tracked separator in labels.

**Examples**
- Hero: *"Walk the path of a monster slayer for hire through a morally grey realm where every contract draws blood."*
- CTA: `BUY NOW` · `WATCH TRAILER` · `EMBARK ON THE PATH`
- Section header: *"More From the Realm"* with eyebrow *"BEYOND THE GAME"*.
- Newsletter success: *"The raven is away. Watch the skies."*

---

## VISUAL FOUNDATIONS

**Color vibe.** Warm gold and blood-red embers against cold near-black and steel. Nothing is pure `#000` or pure `#fff` — grounds are ink (`#08080A`), text is parchment/bone (`#F4EEDE`). Imagery skews **dark, desaturated, and cool** with a warm key light (mist, dusk, torchlight). Gold is the single hero accent; crimson is reserved for brand emphasis and danger; a muted **patina** green marks success.

**Type.** Three families:
- *Display* — **Cinzel** (engraved Roman caps), weights 400–700, tracked `0.04em`. Titles, heroes, character names.
- *Serif* — **EB Garamond**, the literary workhorse for lore and long-form body, line-height 1.7.
- *Sans* — **Barlow** / **Barlow Semi Condensed** for UI: tracked uppercase labels, buttons, eyebrows, metadata.

**Spacing.** 8px base rhythm (`--space-*`). Generous vertical section padding (`--section-y`, clamps 4→8rem). Content max 1280px; prose max ~64ch.

**Backgrounds.** Never flat black. Body composites an **ember wash** (warm radial glow up top) + **vignette** (darkened edges), with an optional **ash grain** SVG-noise texture on media. Full-bleed hero imagery with strong **protection gradients** (top-down fade to ink) so display type stays legible. No bluish-purple gradients.

**Borders & frames.** Thin **gold hairlines** (`--border-hairline`, `rgba(gold, .16)`) at rest; stronger gold on emphasis. Ornate cards add a second inset gold rule for a "framed" feel. Corner ticks mark media placeholders.

**Corner radii.** Deliberately **sharp** — forged metal and stone, not soft plastic. `sm 2px`, `md 4px`, `lg 6px`. Pills only for toggles/pager dots.

**Cards.** Dark raised surface (`--surface-card` `#121217`), 1px gold hairline, deep cold shadow. `ornate` variant = double gold rule. `media` variant = image header with a bottom protection gradient. Hover (interactive): border brightens, shadow deepens, 3px lift.

**Shadows.** Deep, cold, black — `sm → xl`. Elevation comes from darkness, not blur color. Two **glow** tokens add warmth: `--glow-gold` (focus, hero accents) and `--glow-blood`.

**Elevation & light.** Light reads as gold ember glow from above; UI lifts toward the light on hover.

**Motion.** Slow and weighted — **nothing bounces.** `--ease-out` (cinematic decel) and `--ease-in-out`; durations `fast 140ms → cinematic 900ms`. Transitions favor fades, gentle 1–3px lifts, and color shifts. Slide entrances should resolve to the visible state for print/reduced-motion.

**Hover / press.**
- *Hover:* primary buttons gain a gold glow + 1px rise; outline/ghost fill with `--accent-soft` and brighten to gold; links shift gold-400.
- *Press:* colors deepen (gold-600 / crimson-600); no scale-down theatrics.
- *Focus:* 2px offset ring in `--focus-ring` (soft gold) — `--focus-shadow`.

**Transparency & blur.** Used sparingly: sticky nav uses a translucent ink fill + `blur(8px)` once scrolled; modal scrims use `rgba(5,5,6,.78)` + `blur(3px)`. Otherwise surfaces are opaque.

**Layout rules.** Sticky transparent nav over the hero that solidifies on scroll. Section headers use the **ornament divider** (gold hairline broken by a centered diamond or a tracked label). Editorial card grids auto-fit. Generous gutters (`--gutter`).

---

## ICONOGRAPHY

- **Approach:** minimal, **line (stroke) icons** at ~1.6px stroke weight, rounded caps/joins — a feather/Lucide register. Icons are monochrome and inherit `currentColor` (usually gold or bone). They are sparse: navigation chevrons, arrows, play, close, globe, check.
- **Recommended set:** **[Lucide](https://lucide.dev)** — closest match to the in-house glyphs. Link from CDN for consumers:
  `<script src="https://unpkg.com/lucide@latest"></script>` then `lucide.createIcons()`, or use the React package. Set stroke width to `1.6`–`2` and color via `currentColor`.
- **In-house glyphs:** a few tiny inline SVGs ship inside components (chevron in `Select`, close in `Dialog`, checkmark in `Checkbox`, play/arrow in the UI kit). These match the Lucide stroke style so the two are interchangeable.
- **Emoji:** **never used.** Do not introduce emoji as icons.
- **Unicode:** the middle dot `·` is used as a label separator; the diamond ornament is drawn with CSS, not a glyph font. No icon webfont is bundled.
- **Brand mark:** `assets/tintsmith-mark.svg` — an **original** forged-crest emblem (lozenge + crossguard/blade "T"). Use it for the logo lock-up; do not stretch or recolor outside the gold ramp.

---

## SUBSTITUTIONS & CAVEATS
- **Fonts** are nearest-match open-license Google Fonts (Cinzel / EB Garamond / Barlow), loaded from the Google Fonts CDN — **not** bundled binaries, and **not** the reference site's bespoke faces. Provide true webfont files to replace them.
- **Imagery** is procedural placeholder only (no licensed art). Swap in real key art where `Placeholder` appears.
- Generated artifacts (`components/bundle.js`, `_ds_manifest.json`, `_adherence.oxlintrc.json`) are written by the compiler — do not edit by hand.

## Using this system

- Link `styles.css`, or load `tokens.css` and then `components/bundle.css`. `tokens.css` is generated from `tokens.json`: every color, spacing step, radius, border width, shadow and glow, the layout lengths, the font sizes, line heights, letter spacing and weights, and the four families as `--font-display`, `--font-serif`, `--font-sans`, `--font-label`.
- `components/bundle.css` holds what stays CSS: the Google Fonts import, `--focus-shadow`, motion (`--ease-out`, `--ease-in-out`, `--dur-fast` to `--dur-cinematic`), the textures (`--ember-wash`, `--vignette`, `--grain`), the base element styles and the utilities (`.t-eyebrow`, `.t-display-1`, `.t-lead`, `.ornament`, `.container`).
- Components come from `components/bundle.js` as `window.TintsmithDesignSystem_c946a4`, with React 18 on the page. Each has its guide and live preview under `components/<Comp>/`, and its source under `components/src/<group>/`.
- A few pairs sit under the WCAG floors and are kept as the brand has them; their token notes say so: `text-muted` on `surface-card` (4.46:1), and the control borders `border-steel`, `border-strong`, steel-500 and steel-600 (under 3:1). Keep meaning in words and in the focus state, not in those borders alone.
