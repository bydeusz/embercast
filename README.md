# Embercast

A resin slicer for macOS, built for the Creality Halot One Plus. Free and open source, made for personal use.

Embercast is a fork of [mslicer](https://github.com/connorslade/mslicer) by Connor Slade, licensed under GPL-3.0. The file format work follows [UVtools](https://github.com/sn4k3/UVtools) as a reference.

## Status

Embercast exports `.cxdlp` for the Halot One Plus, has a preset for that printer with a Siraya Tech Build profile, places supports with a raft, hollows thick parts with pillars and drain holes, and checks a print for islands, resin traps and suction cups. The first test prints are still to be done.

## Contents

- `mslicer/` holds the slicer: the app, the slicer library, the file formats (including `format/cxdlp_format`) and the tools.
- `profiles/` holds the printer and resin profile for the Halot One Plus with Siraya Tech Build at 50 µm.
- `design/` holds the design tokens and guidelines for the interface, and the script for the temporary icon.
- `testprints/` holds the test models and the scripts that make them.

## Build

Install Rust with rustup, then run `mslicer/dist/build-macos-app.sh`. The app is then in `mslicer/target/Embercast.app`. For development, run `cargo run -p mslicer` in `mslicer/`.

## License

GPL-3.0, see [LICENSE](LICENSE). The copyright notices of mslicer stay in place.
