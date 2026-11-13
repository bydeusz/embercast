# Voortgang Embercast

## WACHT OP JOU

Niets op dit moment.

## Stand van zaken

| Stap | Onderdeel | Status |
|---|---|---|
| 0 | Omgeving (Rust, mslicer, eerste build) | klaar |
| 1 | CXDLP-export | bezig |
| 2 | Printer-preset Halot One Plus + Siraya Build | open |
| 3 | Supports verbeteren | open |
| 4 | Hollowing + drain holes | open |
| 5 | Analyse (islands, resin traps, overhangs, laagvoorvertoning) | open |
| 6 | Interface in Dark Fantasy-stijl, hernoemen naar Embercast | open |
| 7 | Later: auto-orient, auto-layout, meerdere platen, labels, reparatie, meten | open |

## Nu bezig

Stap 1: nieuwe crate `mslicer/format/cxdlp_format`, naar het voorbeeld van `goo_format`, met de opbouw uit UVtools `CrealityCXDLPFile.cs`.

## Volgende stap

De crate aansluiten op `common::slice::format`, `slicer::util` (export en inladen), de CLI en de exportkeuze in de UI. Daarna exporteren en controleren met UVtools.

## Werkafspraken voor een nieuwe sessie

- Alle werk staat op branch `feat/embercast`. `main` blijft gelijk aan GitHub. Niet pushen zonder dat de gebruiker erom vraagt.
- Rust: `. "$HOME/.cargo/env"` als `cargo` niet gevonden wordt. Bouwen en testen vanuit `mslicer/`: `cargo build` en `cargo test`.
- UVtools 7.1.2 staat in `/Applications/UVtools.app`. Command-line: `/Applications/UVtools.app/Contents/MacOS/UVtoolsCmd` (bijv. `print-properties`, `print-issues`, `compare`, `convert`).
- Testbestanden voor de printer gaan naar `testprints/`. Elk bestand eerst door UVtools.

## Beslissingen

- 2026-11-13: mslicer staat als gewone code in `mslicer/` (squash van de subtree). Updates van mslicer later met de hand overnemen.
- 2026-11-13: UVtools zelf gedownload van GitHub (v7.1.2, Apple Silicon), omdat Homebrew het pakket heeft uitgeschakeld.
- 2026-11-13: CXDLP versie 3, zoals UVtools standaard schrijft. Printermodel `CL-79` wordt bepaald uit de resolutie 4320 × 2560.

## Bekende beperkingen

- CXDLP kent één terugtreksnelheid voor alle lagen. Het profiel heeft 150 mm/min voor de bodemlagen en 180 mm/min daarna; in het bestand komt 180 mm/min (3 mm/s).
- CXDLP slaat snelheden op in hele mm/s, bodembelichting en liftafstand in hele seconden en millimeters, en de normale belichting in tienden van een seconde.
