#!/bin/sh
set -e
cd "$(dirname "$0")/.."
cargo build --release --package mslicer
rm -rf target/Embercast.app
cp -R dist/Embercast.app target/Embercast.app
mkdir -p target/Embercast.app/Contents/MacOS
cp target/release/mslicer target/Embercast.app/Contents/MacOS/mslicer
echo "Built $(pwd)/target/Embercast.app"
