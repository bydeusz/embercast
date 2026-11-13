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

**3. Testprint `testprints/embercast-test-2.cxdlp` (pas na een geslaagde testprint 1).**
- Wat het is: hetzelfde testmodel, maar 5 mm opgetild op 117 automatische supports (preset Medium) met een raft van 1 mm.
- Printtijd volgens UVtools ongeveer 1 uur en 31 minuten, ongeveer 3,4 ml hars, 330 lagen.
- Gecontroleerd met UVtools: geen zwevende supports en geen harsvallen. UVtools meldt wel de onderkant van het plaatje als "eiland" met overhangende stukjes. Dat doet UVtools bij elke platte onderkant die op supportpuntjes rust.

Waar je op moet letten:
1. Blijft de raft aan de bouwplaat zitten, en is hij met een schraper los te krijgen (de rand loopt schuin af)?
2. Staan alle supports er, of zijn er kolommen afgebroken of los gekomen?
3. Hangt het plaatje aan de supports, of is het (deels) van de supports losgescheurd en op de FEP blijven plakken?
4. Gaan de supports goed los van de onderkant van het plaatje, en hoe zien de contactpunten eruit (putjes, bultjes)?

**4. Testprint `testprints/embercast-test-3.cxdlp` (pas na een geslaagde testprint 2).**
- Wat het is: een kubus van 20 mm, hol gemaakt met een wand van 2 mm, pilaren om de 4 mm en 2 afvoergaten van 2,5 mm in de bodem. De kubus staat 5 mm hoog op 82 supports (Medium) met een raft.
- Printtijd volgens UVtools ongeveer 2 uur en 14 minuten, ongeveer 5,3 ml hars (massief zou 8,9 ml zijn), 500 lagen.
- Gecontroleerd met UVtools: geen harsvallen en geen zuignappen. UVtools meldt het plafond van de holte als 16 overhangende vakjes van 3 × 3 mm tussen de pilaren. Dat is bij deze maat normaal.

Waar je op moet letten:
1. Is de kubus heel gebleven, zonder scheuren of gaten in de wanden?
2. Komt er hars uit de twee gaatjes in de onderkant als je hem na het printen kantelt? Spoel de holte goed na, ook bij het uitharden.
3. Zaag of breek hem na het uitharden eventueel door: is de wand ongeveer 2 mm, staan de pilaren er, en is het plafond dicht en glad genoeg?

Ik werk intussen verder aan de stappen die hier niet van afhangen.

## Stand van zaken

| Stap | Onderdeel | Status |
|---|---|---|
| 0 | Omgeving (Rust, mslicer, eerste build) | klaar |
| 1 | CXDLP-export | gebouwd en met UVtools gecontroleerd, wacht op testprint |
| 2 | Printer-preset Halot One Plus + Siraya Build | klaar |
| 3 | Supports verbeteren | gebouwd, wacht op testprint 2 |
| 4 | Hollowing + drain holes | gebouwd, wacht op testprint 3 |
| 5 | Analyse (islands, resin traps, overhangs, laagvoorvertoning) | klaar |
| 6 | Interface in Dark Fantasy-stijl, hernoemen naar Embercast | klaar (veldlabels nog niet in hoofdletters) |
| 7 | Later: auto-orient, auto-layout, meerdere platen, labels, reparatie, meten | bezig |

## Wat af is

- Stap 1: crate `mslicer/format/cxdlp_format` (schrijven en inlezen van CXDLP v3, met 26 tests). Aangesloten op de formatenlijst, de CLI (`slicer ... uitvoer.cxdlp`), het Save-menu van de app en het openen van gesliceerde bestanden.
- Controle: UVtools leest het bestand, het controlegetal klopt, en `UVtoolsCmd compare` zegt "Files matches" tegen de `.cxdlp` die UVtools zelf maakt van dezelfde lagen. De lijndata van alle lagen is byte voor byte gelijk.
- Testmodel: `testprints/make_test_model.py` maakt `embercast-test-1.stl`. Het printbestand maak je opnieuw met het commando onder "Hoe maak je het testbestand opnieuw".
- Stap 2: printer `Creality HALOT-ONE PLUS` in de printerlijst van de app. Harsprofielen staan in `mslicer/common/src/slice/profiles.rs`, met als eerste "Siraya Tech Build 50 µm". In het paneel Slice Config kies je het profiel bij "Resin Profile". Een test vergelijkt het profiel met `profiles/halot-one-plus_siraya-build-50um.json`. De standaardinstellingen van de app en van de CLI zijn nu deze printer met dit profiel.

- Stap 3, supports:
  - Presets Light, Medium en Heavy met elk een eigen tipdikte (0,3 / 0,4 / 0,6 mm) en lijfdikte (0,8 / 1,2 / 1,6 mm). De keuzelijst toont de naam, en tipdiameter, tiplengte, lijfdiameter en vrije ruimte zijn aan te passen.
  - Raft: hoogte en rand uit de instellingen worden nu gebruikt. De onderkant is breder dan de bovenkant, zodat een schraper eronder kan. Hoogte 0 betekent geen raft.
  - Supports bewegen mee met hun model. Hoger zetten maakt de kolommen langer. Draaien of schalen wist de supports, want dan passen ze niet meer.
  - Supports en supportinstellingen worden opgeslagen in het projectbestand (projectversie 17). Oudere projecten openen gewoon.
  - Automatisch plaatsen: alleen nog binnen het gebied onder het model, standaard om de 2,5 mm (was 50 mm). Geen supports meer naar vlakken die binnenin het model liggen, bijvoorbeeld bij overlappende delen.
  - Handmatig plaatsen (klikken op het model) en verwijderen (selecteren, Delete) bestonden al in mslicer.
  - Hulpprogramma `mslicer/target/release/supports` (crate `tools`): plaatst automatisch supports onder een STL en schrijft model, supports en raft samen weg als één STL. Zo kan ik supports zonder de app via de slicer en UVtools controleren.

- Stap 4, hollowing (`mslicer/slicer/src/post_process/hollow.rs`):
  - Werkt op de gesneden lagen, als eerste nabewerking. Een grof voxelraster (hooguit 0,25 mm) over het verlichte deel, de exacte afstand tot de buitenkant, en alles wat verder weg ligt dan de wanddikte wordt leeg. De wand is nooit dunner dan ingesteld (hooguit één voxel dikker).
  - Dunne delen zoals supports en de raft blijven vanzelf massief.
  - Pilaren (optioneel): vierkante staanders met instelbare afstand en breedte. Die sluiten geen hars in, een rooster van wanden zou dat wel doen.
  - Afvoergaten: automatisch op de laagste laag van de holte (het dichtst bij de bouwplaat), recht omlaag door de bodemwand. Een gat stopt bij de eerste laag die grotendeels leeg is, zodat supports eronder heel blijven.
  - In de app: paneel Slice Config, Post Processors, "Hollowing". Undo werkt, en de instellingen staan in het projectbestand (versie 18).
  - CLI: `--hollow <wand mm> --drain-holes <aantal> --drain-hole-diameter <mm> --pillar-spacing <mm>`.

- Stap 5, analyse:
  - Na het slicen staat bovenin de knop "Check Print". Die zoekt eilanden (bestond al in mslicer), harsvallen en zuignappen (nieuw, `mslicer/slicer/src/post_process/cavities.rs`).
  - Harsval: lege ruimte zonder uitweg. Zuignap: lege ruimte die alleen naar de FEP open is. De bouwplaat telt als dicht als de print op laag 0 begint.
  - In de laagvoorvertoning en op de laagschuif: eiland donkerrood (crimson-500), harsval lichtrood (crimson-300), zuignap amber (gold-500). Onder "Analysis, Print Check" staat per probleem in woorden wat er is, met een link naar de eerste laag.
  - Overhangkleuring in de 3D-weergave staat nu standaard aan (30°), in te stellen in het paneel Workspace.
  - De laagvoorvertoning bestond al in mslicer.
  - CLI: `--analyze` drukt de gevonden holtes af.

- Stap 6, interface en naam:
  - Thema-module `mslicer/mslicer/src/interface/theme.rs` is de enige plek met kleuren en lettertypes, allemaal uit `design/tokens.json`: inktkleurige vlakken, dunne gouden lijnen, hoeken van 2 en 4 px, goud voor links en selectie, crimson voor problemen.
  - Lettertypes in `mslicer/mslicer/assets/fonts/` met hun OFL-licentie: Cinzel (titels, woordmerk), Barlow (alle tekst en getallen), Barlow Semi Condensed (knoppen en labels), EB Garamond (uitleg in het About-scherm).
  - Menu's, de Slice-knop en de tabbladtitels staan in hoofdletters met ruime letterafstand (`theme::label`).
  - 3D-weergave: achtergrond `bg-base`, bouwplaatraster in ink-300, geselecteerde vlakken goud, overhang en buiten het volume crimson.
  - Naam: venstertitel, woordmerk linksboven, About-scherm (met verwijzing naar mslicer en UVtools), Help-menu, instellingenmap (`~/Library/Application Support/embercast`) en de tekst in projectbestanden. De update-controle tegen mslicer-versies staat standaard uit.
  - macOS-app: `mslicer/dist/Embercast.app` (bundel-ID `io.github.bydeusz.embercast`). Bouwen met `mslicer/dist/build-macos-app.sh`; de app staat daarna in `mslicer/target/Embercast.app`.
  - Tijdelijk icoon: een gouden Cinzel-E op een inkt-tegel, gemaakt met `design/icon/make_icon.swift` (`swift design/icon/make_icon.swift <Cinzel.ttf> <uit.png>`).

## Nu bezig

Stap 7: auto-orient, auto-layout, meerdere bouwplaten, tekstlabels, modelreparatie, meetgereedschap.

## Volgende stap

Na stap 7: de uitslagen van de testprints verwerken.

## Hoe maak je het testbestand opnieuw

Vanuit de hoofdmap, na `cargo build -r -p slicer` in `mslicer/`. De CLI gebruikt standaard de Halot One Plus met het Siraya-profiel:

```
python3 testprints/make_test_model.py testprints/embercast-test-1.stl
mslicer/target/release/slicer --mesh testprints/embercast-test-1.stl testprints/embercast-test-1.cxdlp
/Applications/UVtools.app/Contents/MacOS/UVtoolsCmd print-issues testprints/embercast-test-1.cxdlp
```

Testprint 3:

```
python3 testprints/make_test_model.py testprints/cube-20mm.stl cube
mslicer/target/release/supports --preset medium testprints/cube-20mm.stl testprints/embercast-test-3.stl
mslicer/target/release/slicer --mesh testprints/embercast-test-3.stl --hollow 2.0 --drain-holes 2 \
  --drain-hole-diameter 2.5 --pillar-spacing 4 testprints/embercast-test-3.cxdlp
```

Testprint 2 (na `cargo build -r -p tools --bin supports`):

```
mslicer/target/release/supports --preset medium testprints/embercast-test-1.stl testprints/embercast-test-2.stl
mslicer/target/release/slicer --mesh testprints/embercast-test-2.stl testprints/embercast-test-2.cxdlp
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
- 2026-11-13: `SliceConfig::default()` is nu de Halot One Plus met het Siraya-profiel (was een Elegoo Saturn). Je bestaande mslicer-instellingen van stap 0 bewaren nog de oude standaard; die verdwijnen vanzelf bij het hernoemen naar Embercast in stap 6, of met "Reset Config" in het paneel Workspace.
- 2026-11-13: het inlezen van `.cxdlp` controleert vóór elk blok of er genoeg bytes zijn, zodat een kapot bestand een foutmelding geeft en de app niet laat crashen.
- 2026-11-13: supports worden opgeslagen ten opzichte van de positie van het model (`tools/src/supports/set.rs`). Bij een andere schaal of draaiing vervallen ze, in plaats van dat ik ze probeer mee te draaien.
- 2026-11-13: hollowing gebeurt op de gesneden lagen en niet op het 3D-model. Zo blijft het oppervlak van het model precies zoals het was, en kunnen gaten en holtes geen kapotte mesh opleveren. Nadeel: hollowing geldt voor alles op de plaat, niet per model, en je ziet de holte pas na het slicen in de laagvoorvertoning.
- 2026-11-13: harsvallen en zuignappen worden op een raster van ongeveer 0,1 mm gezocht (grover bij hele grote prints, hooguit 40 miljoen voxels). Holtes kleiner dan 0,2 mm³ worden niet gemeld.
- 2026-11-13: preset bepaalt de vorm van een support, de dichtheid staat apart onder "Auto Placement". Zo blijven de twee los te regelen.

## Bekende beperkingen

- In de laagafbeelding staat de F op zijn kop ten opzichte van het bovenaanzicht van het model. mslicer doet dat ook bij de andere formaten. De Chitubox-vergelijking of de testprint moet uitwijzen of dit goed is op de Halot.
- CXDLP kent geen hoogte en belichting per laag. Overgangslagen en variabele laaghoogte werken daarom niet; in de app is CXDLP uitgeschakeld bij variabele laaghoogte.

- Het inlezen van projectbestanden (uit mslicer) begrenst nog niet het aantal modellen en meshes in een bestand. Een gemanipuleerd projectbestand kan de app dus nog laten vastlopen. Voor de nieuwe supportdelen is dat wel begrensd.
- UVtools meldt bij een platte onderkant op supports altijd een "eiland". Extra supports zetten op plekken die in de lagen echt los hangen, hoort bij stap 5.
- De veldlabels in de panelen ("Slice Mode", "Printer" en zo) staan nog in gewone schrijfwijze. egui kent geen letterafstand voor de hele app, dus elke tekst moet apart; dat zijn er honderden.
- Er is nog geen eigen logo of embleem. Het icoon is een tijdelijke monogram-E.
- De programmanaam in de bundel is nog `mslicer` (`Contents/MacOS/mslicer`). macOS toont overal "Embercast", maar in Terminal en logbestanden zie je de oude naam.
- Afvoergaten worden alleen automatisch geplaatst, recht omlaag. Zelf een plek voor een gat aanklikken kan nog niet.
- Een model met holte dat direct op de bouwplaat staat, krijgt gaten die tegen de plaat uitkomen. Die zijn dan dicht. Zet holle modellen op supports.
- CXDLP kent één terugtreksnelheid voor alle lagen. Het profiel heeft 150 mm/min voor de bodemlagen en 180 mm/min daarna; in het bestand komt 180 mm/min (3 mm/s).
- CXDLP slaat snelheden op in hele mm/s, bodembelichting en liftafstand in hele seconden en millimeters, en de normale belichting in tienden van een seconde.
