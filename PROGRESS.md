# Voortgang Embercast

## WACHT OP JOU

**1. Chitubox-vergelijking (5 minuten, graag vóór de print).**
Open `testprints/embercast-test-1.stl` in Chitubox, kies je Halot One Plus met het Siraya Build-profiel, slice en sla op als `testprints/chitubox-test-1.cxdlp`. Daarmee kan ik zonder print nagaan of mijn bestand niet gespiegeld is en of alle instellingen gelijk staan aan wat Chitubox schrijft.

**2. Testprint `testprints/embercast-test-1.cxdlp`.**
- Wat het is: een plaatje van 30 × 20 × 1,5 mm met daarop een kubus van 10 mm, de letter F (3 mm hoog) en drie dunne pinnetjes van 1,0, 0,6 en 0,4 mm.
- Het model staat direct op de bouwplaat, zonder supports. Dat is de bedoeling.
- Printtijd volgens UVtools ongeveer 1 uur en 6 minuten, ongeveer 2,1 ml hars. 230 lagen van 0,05 mm, 6 bodemlagen van 35 s, daarna 3,2 s.
- Gecontroleerd met UVtools: geen problemen gevonden, instellingen gelijk aan je Chitubox-profiel.

Waar je op moet letten:
1. Start de print gewoon vanaf de printer? Of geeft de printer een foutmelding bij het openen van het bestand?
2. Blijft het plaatje aan de bouwplaat zitten?
3. Zet het onderdeel na afloop op het plaatje neer en kijk van boven: lees je de F normaal, of in spiegelbeeld?
4. Meet de kubus: is hij 10,0 × 10,0 mm, en is het geheel 11,5 mm hoog?
5. Welke van de drie pinnetjes zijn er (heel) uitgekomen?
6. Toont de printer een voorbeeldplaatje? (Bij deze print is het zwart, dat is nog verwacht.)

Ik werk intussen verder aan de stappen die hier niet van afhangen.

## Stand van zaken

| Stap | Onderdeel | Status |
|---|---|---|
| 0 | Omgeving (Rust, mslicer, eerste build) | klaar |
| 1 | CXDLP-export | gebouwd en met UVtools gecontroleerd, wacht op testprint |
| 2 | Printer-preset Halot One Plus + Siraya Build | open |
| 3 | Supports verbeteren | open |
| 4 | Hollowing + drain holes | open |
| 5 | Analyse (islands, resin traps, overhangs, laagvoorvertoning) | open |
| 6 | Interface in Dark Fantasy-stijl, hernoemen naar Embercast | open |
| 7 | Later: auto-orient, auto-layout, meerdere platen, labels, reparatie, meten | open |

## Wat af is

- Stap 1: crate `mslicer/format/cxdlp_format` (schrijven en inlezen van CXDLP v3, met 26 tests). Aangesloten op de formatenlijst, de CLI (`slicer ... uitvoer.cxdlp`), het Save-menu van de app en het openen van gesliceerde bestanden.
- Controle: UVtools leest het bestand, het controlegetal klopt, en `UVtoolsCmd compare` zegt "Files matches" tegen de `.cxdlp` die UVtools zelf maakt van dezelfde lagen. De lijndata van alle lagen is byte voor byte gelijk.
- Testmodel: `testprints/make_test_model.py` maakt `embercast-test-1.stl`. Het printbestand maak je opnieuw met het commando onder "Hoe maak je het testbestand opnieuw".

## Nu bezig

Stap 2: printer-preset Halot One Plus en het Siraya Build-profiel.

## Volgende stap

Stap 3: supports (presets licht/middel/zwaar, raft, handmatig plaatsen en verwijderen).

## Hoe maak je het testbestand opnieuw

Vanuit de hoofdmap, na `cargo build -r -p slicer` in `mslicer/`:

```
python3 testprints/make_test_model.py testprints/embercast-test-1.stl
mslicer/target/release/slicer --platform-resolution "4320, 2560" --platform-size "172.8, 102.4, 160" \
  --layer-height 0.05 --first-layers 6 --transition-layers 0 \
  --exposure-time 3.2 --exposure-delay 4 --exposure-pwm 100 --lift-distance 6 --lift-speed 60 --retract-speed 180 \
  --first-exposure-time 35 --first-exposure-delay 4 --first-exposure-pwm 100 --first-lift-distance 6 \
  --first-lift-speed 60 --first-retract-speed 150 \
  --mesh testprints/embercast-test-1.stl testprints/embercast-test-1.cxdlp
/Applications/UVtools.app/Contents/MacOS/UVtoolsCmd print-issues testprints/embercast-test-1.cxdlp
```

## Werkafspraken voor een nieuwe sessie

- Alle werk staat op branch `feat/embercast`. `main` blijft gelijk aan GitHub. Niet pushen zonder dat de gebruiker erom vraagt.
- Rust: `. "$HOME/.cargo/env"` als `cargo` niet gevonden wordt. Bouwen en testen vanuit `mslicer/`: `cargo build` en `cargo test`.
- UVtools 7.1.2 staat in `/Applications/UVtools.app`. Command-line: `/Applications/UVtools.app/Contents/MacOS/UVtoolsCmd` (bijv. `print-properties`, `print-issues`, `compare`, `convert`).
- Testbestanden voor de printer gaan naar `testprints/`. Elk bestand eerst door UVtools.

## Beslissingen

- 2026-11-13: mslicer staat als gewone code in `mslicer/` (squash van de subtree). Updates van mslicer later met de hand overnemen.
- 2026-11-13: UVtools zelf gedownload van GitHub (v7.1.2, Apple Silicon), omdat Homebrew het pakket heeft uitgeschakeld.
- 2026-11-13: CXDLP versie 3, zoals UVtools standaard schrijft. Printermodel `CL-79` wordt bepaald uit de resolutie 4320 × 2560.

- 2026-11-13: in het blok `SlicerInfoV3` schrijf ik dezelfde waarden als UVtools (onder meer twee aan/uit-vlaggen op 1, alle compensatiewaarden 0). UVtools-bestanden printen bewezen op Halot-printers.
- 2026-11-13: het oppervlak per laag (dat de printer bij elke laag krijgt) is het pixeloppervlak van de grootste vorm met de gaten erin meegeteld. UVtools meet langs de rand en komt ongeveer 1% lager uit. Wat de printer met dit getal doet is niet bekend.
- 2026-11-13: de CLI zet modellen nu net als de app een halve laag omlaag, zodat elke laag in het midden wordt gesneden. Zonder dat bleef laag 0 van de theepot leeg.
- 2026-11-13: het inlezen van `.cxdlp` controleert vóór elk blok of er genoeg bytes zijn, zodat een kapot bestand een foutmelding geeft en de app niet laat crashen.

## Bekende beperkingen

- In de laagafbeelding staat de F op zijn kop ten opzichte van het bovenaanzicht van het model. mslicer doet dat ook bij de andere formaten. De Chitubox-vergelijking of de testprint moet uitwijzen of dit goed is op de Halot.
- CXDLP kent geen hoogte en belichting per laag. Overgangslagen en variabele laaghoogte werken daarom niet; in de app is CXDLP uitgeschakeld bij variabele laaghoogte.

- CXDLP kent één terugtreksnelheid voor alle lagen. Het profiel heeft 150 mm/min voor de bodemlagen en 180 mm/min daarna; in het bestand komt 180 mm/min (3 mm/s).
- CXDLP slaat snelheden op in hele mm/s, bodembelichting en liftafstand in hele seconden en millimeters, en de normale belichting in tienden van een seconde.
