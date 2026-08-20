#!/usr/bin/env bash
# Sukuria target-triple sufiksuotą nullbyte-emu binarą, kurio reikalauja tauri-plugin-shell
# sidecar (CLAUDE.md §4/ADR-016, MVP.md P4.0.3/P4.0.5). Kviečiamas iš tauri.conf.json
# `beforeDevCommand`/`beforeBuildCommand` PRIEŠ paleidžiant `cargo run`/`cargo build` nullbyte-app'ui —
# `tauri-build`'o build.rs žingsnis (`copy_binaries`) reikalauja šio failo JAU egzistuojant,
# kitaip visas nullbyte-app build'as žlunga su `std::process::exit(1)` (patikrinta tauri-build
# 2.6.3 šaltinyje: crates/nullbyte-app/../tauri-build klaida "does not exist" nėra minkšta).
#
# NEAPIMA multi-arch/universal build'o (P4.0.5) — tik dabartinio hosto triple.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

PROFILE="${1:-debug}"
TARGET_TRIPLE="$(rustc --print host-tuple)"
DEST_DIR="crates/nullbyte-app/binaries"
DEST="$DEST_DIR/nullbyte-emu-$TARGET_TRIPLE"

if [ "$PROFILE" = "release" ]; then
  cargo build --package nullbyte-emu --release
  SRC="target/release/nullbyte-emu"
else
  cargo build --package nullbyte-emu
  SRC="target/debug/nullbyte-emu"
fi

mkdir -p "$DEST_DIR"
cp "$SRC" "$DEST"
echo "nullbyte-emu sidecar paruoštas: $DEST"
