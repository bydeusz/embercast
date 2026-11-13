# Embercast (persoonlijk project)

**Embercast** is een eigen resin-slicer voor macOS voor een **Creality Halot One Plus**, met de functies die je kent uit Chitubox Basic/Advanced/Pro, maar dan gratis en open source. Alleen voor eigen gebruik.

De gebruiker is Nederlandstalig: communiceer in het Nederlands.

## Afspraken (belangrijk)

- **Functies nabouwen mag, Chitubox kopiëren niet.** Ideeën als auto-supports, hollowing, drain holes, island-detectie, multi-platform, labels en reparatie zijn generiek. Neem géén code, iconen, logo's, kleuren of de exacte schermindeling van Chitubox over. De look and feel komt uit het eigen design system van de gebruiker (zie "Look and feel").
- De geïnstalleerde Chitubox-app niet decompileren, niet analyseren en geen Pro-functies ontgrendelen. Alleen de eigen printer-/harsinstellingen van de gebruiker aflezen is prima (al gedaan, zie hieronder).
- **Basis: [mslicer](https://github.com/connorslade/mslicer)** (Rust, GPL-3.0). Code mag gebruikt en aangepast worden. Bij delen moet het project ook open source blijven. Embercast is een fork: laat de GPL-licentie en de copyright-vermeldingen van mslicer staan en noem mslicer in een About-scherm en in de README.
- **Referentie voor bestandsformaten: [UVtools](https://github.com/sn4k3/UVtools)** (C#, AGPL-3.0). Vooral `UVtools.Core/FileFormats/CrealityCXDLPFile.cs`.

## Printer en profiel

- Creality Halot One Plus (Creality-modelcode `CL-79`)
- Resolutie 4320 × 2560 px, bouwvolume 172,8 × 102,4 × 160 mm, pixel 40 µm, niet gespiegeld
- **Bestandsformaat: `.cxdlp`** (magic `CXSW3DV2`, UVtools schrijft standaard versie 3, ondersteunt 2 en 3). mslicer kan dit nog **niet** exporteren: dat is stap 1.
- Werkend profiel van de gebruiker staat in `profiles/halot-one-plus_siraya-build-50um.json` (afgelezen uit Chitubox): 0,05 mm laag, 3,2 s belichting, 6 bodemlagen à 35 s, 4 s licht-uit, lift 6 mm @ 60 mm/min, retract 150 (bodem) / 180 mm/min, PWM 255, geen anti-aliasing.

## Look and feel: DS Dark Fantasy World

De interface volgt het design system **"DS: Dark Fantasy world"** van de gebruiker (intern "Tintsmith Design System"): https://claude.ai/artifact/WpgD4e13JADTohi4jEi2Qk. Een kopie staat offline in `design/`:

- `design/tokens.json`: alle tokens (kleuren, typografie, spacing, radius, borders, schaduwen). **Bron van waarheid voor waarden**; nooit kleuren verzinnen.
- `design/DESIGN_SYSTEM.md`: de richtlijnen (sfeer, typografie, randen, hover/focus, iconen, toon).

**Vertalen naar mslicer.** mslicer gebruikt egui (Rust), geen HTML/React. De React-componenten en CSS uit het design system zijn dus niet direct bruikbaar; vertaal de tokens naar egui:

- Maak één thema-module (bijv. `mslicer/src/interface/theme.rs`) die `egui::Visuals`/`egui::Style` vult vanuit de tokens, en gebruik in de rest van de UI alleen die namen.
- **Kleuren** (één thema: donker): achtergrond `bg-base` #08080A, panelen `surface-card` #121217, verhoogd `surface-card-raised` #1A1A21, tekst `text-primary` #F4EEDE / `text-secondary` #C9BCA0, accent `accent` #D18B2C (hover #E6AB4C, press #AD6D1D), gevaar crimson #9E2B25, succes patina #4E6E57. Dunne goudkleurige randen (hairline) op rust.
- **Hoeken scherp**: radius 2 / 4 / 6 px, nooit zachte ronde knoppen.
- **Fonts** (open licentie, OFL): Cinzel alleen voor titels/kopjes, Barlow / Barlow Semi Condensed voor alle UI, labels en getallen (leesbaarheid gaat voor sfeer), EB Garamond alleen voor langere teksten (uitleg, about). Bundel de TTF-bestanden in de app.
- **Labels en knoppen** in hoofdletters met ruime letterafstand; geen emoji; iconen als dunne lijniconen (Lucide-stijl) in goud of bot-wit.
- **Beweging** rustig, niets dat stuitert.
- **3D-weergave**: bouwplaat en achtergrond in ink-tinten; geselecteerd model in goud; problemen (islands, resin traps, zuignappen) in crimson; in orde in patina.
- Het merk "Tintsmith", zijn embleem en de fictieve game "Wild Reign" uit het design system zijn **niet** het merk van deze slicer: gebruik alleen de stijl. De slicer heet **Embercast**; een eigen logo/embleem moet nog ontworpen worden (gebruik tot die tijd de naam als woordmerk in Cinzel, goud).
- Let op contrast: sommige combinaties (zoals `text-muted` op `surface-card`) zitten net onder de WCAG-grens; gebruik voor belangrijke informatie `text-secondary` of `text-primary`.

## Stappenplan

0. **Omgeving**: Rust installeren via rustup (vraag de gebruiker eerst), `git clone https://github.com/connorslade/mslicer` in `./mslicer`, en `cargo run -r -p mslicer` laten draaien op de Mac. In mslicer is de standaardprinter een Elegoo Saturn 3 Ultra; de Halot One Plus wordt in stap 2 toegevoegd.
1. **CXDLP-export** (hoogste prioriteit, zonder dit kan er niet geprint worden):
   - Nieuwe crate `format/cxdlp_format` naar het voorbeeld van `format/goo_format` (header, layers, preview, encoding).
   - Formaat volgen uit UVtools `CrealityCXDLPFile.cs` (header, SlicerInfo/SlicerInfoV3, previews, layer-definities met lijn-encoding, footer, checksum).
   - Aansluiten op de export-keuze in de UI en in de CLI (`slicer`).
   - **Verifiëren**: exporteer een testmodel, open het in UVtools (`print-properties`, `print-issues`, of de GUI) en vergelijk met een `.cxdlp` die Chitubox maakt van hetzelfde model. Pas daarna een kleine testprint.
2. **Printer-preset** Halot One Plus en het Siraya Build-profiel toevoegen (`mslicer/src/core/config/printers.rs`, slice-config).
3. **Supports verbeteren**: mslicer heeft basis-supportgeneratie (`mslicer/src/task/mesh/generate_supports.rs`). Uitbreiden met lichte/middel/zware presets, raft/basisplaat en handmatig plaatsen/verwijderen.
4. **Hollowing + drain holes** met instelbare wanddikte en infill.
5. **Analyse**: island-detectie (bestaat al deels), resin traps / zuignappen, overhangs, laagvoorvertoning.
6. **Interface in de Dark Fantasy-stijl**: thema-module en fonts zoals beschreven onder "Look and feel", en de app hernoemen naar Embercast (venstertitel, `dist/mslicer.app` → `Embercast.app`, bundle-id, Info.plist, icoon). Mag ook eerder, zodra de export werkt.
7. **Later**: auto-orient, auto-layout, meerdere bouwplaten, tekstlabels, modelreparatie, meetgereedschap.

## Werkwijze

- Kleine stappen, na elke stap compileren en testen.
- Nooit een bestand naar de printer zonder eerst in UVtools te controleren.
- Commits met duidelijke berichten; geen force-push.
