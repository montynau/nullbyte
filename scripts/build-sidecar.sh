#!/usr/bin/env bash
# Sukuria target-triple sufiksuotą nullbyte-emu binarą, kurio reikalauja tauri-plugin-shell
# sidecar (CLAUDE.md §4/ADR-016, MVP.md P4.0.3/P4.0.5). Kviečiamas iš tauri.conf.json
# `beforeDevCommand`/`beforeBuildCommand` PRIEŠ paleidžiant `cargo run`/`cargo build` nullbyte-app'ui —
# `tauri-build`'o build.rs žingsnis (`copy_binaries`) reikalauja šio failo JAU egzistuojant,
# kitaip visas nullbyte-app build'as žlunga su `std::process::exit(1)` (patikrinta tauri-build
# 2.6.3 šaltinyje: crates/nullbyte-app/../tauri-build klaida "does not exist" nėra minkšta).
#
# P4.0.5 (universal build): PATIKRINTA REALIU `pnpm tauri build --target universal-apple-darwin`
# paleidimu (2026-08-25) — ANKSTESNĖ šio scripto/MVP.md prielaida („kiekvienas externalBin
# turi būti paduotas per DU atskirus triple sufiksuotus failus") buvo KLAIDINGA. Tikras
# klaidos pranešimas: `Failed to copy external binaries: resource path
# binaries/nullbyte-emu-universal-apple-darwin doesn't exist` — Tauri universal build'ui
# ieško VIENO, JAU `lipo`'into binaro su `-universal-apple-darwin` sufiksu, ne dviejų
# per-arch failų. Todėl macOS `release` profilis stato ABU triple'us IR juos sulieja per
# `lipo -create` į trečią, universal-sufiksuotą failą — visi trys lieka `binaries/`, kad
# veiktų IR pavienio host'o `pnpm tauri build`, IR `--target universal-apple-darwin`.
# `debug` profilis (dev ciklas, `pnpm tauri dev`) LIEKA tik hosto triple — greitis svarbiau.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

PROFILE="${1:-debug}"
DEST_DIR="crates/nullbyte-app/binaries"
mkdir -p "$DEST_DIR"

build_for_triple() {
  local triple="$1"
  local dest="$DEST_DIR/nullbyte-emu-$triple"
  local host_triple
  host_triple="$(rustc --print host-tuple)"

  if [ "$PROFILE" = "release" ]; then
    if [ "$triple" = "$host_triple" ]; then
      cargo build --package nullbyte-emu --release
    else
      cargo build --package nullbyte-emu --release --target "$triple"
    fi
    src="target/release/nullbyte-emu"
    [ "$triple" = "$host_triple" ] || src="target/$triple/release/nullbyte-emu"
  else
    cargo build --package nullbyte-emu
    src="target/debug/nullbyte-emu"
  fi

  cp "$src" "$dest"
  echo "nullbyte-emu sidecar paruoštas: $dest"
}

if [ "$PROFILE" = "release" ] && [ "$(uname -s)" = "Darwin" ]; then
  build_for_triple "aarch64-apple-darwin"
  build_for_triple "x86_64-apple-darwin"

  UNIVERSAL_DEST="$DEST_DIR/nullbyte-emu-universal-apple-darwin"
  lipo -create -output "$UNIVERSAL_DEST" \
    "$DEST_DIR/nullbyte-emu-aarch64-apple-darwin" \
    "$DEST_DIR/nullbyte-emu-x86_64-apple-darwin"
  echo "nullbyte-emu universal sidecar paruoštas: $UNIVERSAL_DEST"
else
  build_for_triple "$(rustc --print host-tuple)"
fi
