fn main() {
    check_sidecar_exists();
    tauri_build::build()
}

/// `tauri-build` (per `tauri.conf.json` `bundle.externalBin`) reikalauja, kad
/// `crates/nullbyte-app/binaries/nullbyte-emu-<target-triple>` JAU egzistuotų šio build.rs
/// paleidimo metu — kitaip `tauri_build::build()` panikuoja giliai viduje (`copy_binaries()`)
/// su neinformatyviu „does not exist" pranešimu (patikrinta prieš `tauri-build` 2.6.3 šaltinį,
/// MVP.md P4.0.3 pastaba). `nullbyte-app` NEPRIKLAUSO nuo `nullbyte-emu` Cargo priklausomybių
/// grafe (vištos-kiaušinio problema — sidecar'as reikalingas runtime, ne kompiliavimo metu),
/// tad Cargo savaime negarantuoja teisingos statymo tvarkos `cargo build --workspace` metu.
/// `pnpm tauri dev`/`build` tai sprendžia automatiškai per `beforeDevCommand`/
/// `beforeBuildCommand` (žr. tauri.conf.json), CI — per atskirą žingsnį
/// (`.github/workflows/ci.yml`). Šis patikrinimas gaudo LIKUSĮ atvejį — tiesioginį
/// `cargo build/test/clippy --workspace` paleidimą be išankstinio sidecar build'o — ir duoda
/// aiškų, veiksmingą pranešimą vietoj tauri-build vidinio.
fn check_sidecar_exists() {
    let target_triple =
        std::env::var("TARGET").expect("Cargo turėtų nustatyti TARGET build.rs metu");
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("Cargo turėtų nustatyti CARGO_MANIFEST_DIR build.rs metu");
    let sidecar = std::path::Path::new(&manifest_dir)
        .join("binaries")
        .join(format!("nullbyte-emu-{target_triple}"));

    if !sidecar.exists() {
        panic!(
            "\n\nTrūksta nullbyte-emu sidecar binaro: {}\n\
             Paleisk PRIEŠ `cargo build`/`test`/`clippy`:\n  pnpm run build:sidecar\n\
             (release build'ui: pnpm run build:sidecar:release)\n\
             `pnpm tauri dev`/`pnpm tauri build` tai daro automatiškai (beforeDevCommand/beforeBuildCommand).\n\n",
            sidecar.display()
        );
    }
}
