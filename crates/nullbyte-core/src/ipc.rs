//! `nullbyte-emu` ↔ `nullbyte-app` IPC protokolo bendri tipai (CLAUDE.md §3.4/§10, ADR-016,
//! MVP.md P4.0.3). Transportas: NDJSON per stdin (`EmuCommand`, tėvas → vaikas) / stdout
//! (`EmuStatus`, vaikas → tėvas) — žr. `crates/nullbyte-emu/src/ipc.rs` (serveris) ir
//! `crates/nullbyte-app/src/ipc.rs` (klientas) realiam skaitymo/rašymo loop'ui.
//!
//! ## Protokolo versijos handshake
//!
//! Pati PIRMA eilutė ABIEM kryptimis PRIVALO būti [`IpcHello`] (NE `EmuCommand`/`EmuStatus`)
//! — apsauga nuo pasenusio sidecar binaro (build grandinė paprastai jį perstato prieš
//! kiekvieną `pnpm tauri dev`/`build`, žr. MVP.md P4.0.3 priešdarbio pastabą, bet rizika
//! nenulinė — pvz. rankiniu būdu paleidus seną `target/debug/nullbyte-emu` tiesiogiai).
//! Be handshake'o toks neatitikimas pasireikštų kaip nesuprantamos NDJSON parse klaidos giliai
//! protokolo viduryje, ne kaip aiškus „versijos nesutampa" pranešimas iškart prisijungus.
//!
//! `IpcHello` yra SĄMONINGAI ATSKIRAS tipas, ne `EmuCommand`/`EmuStatus` variantas — protokolo
//! lygmuo ([]versija") ir žaidimo valdymo/būvio lygmuo yra skirtingi rūpesčiai, o vienas
//! bendras tipas abiem kryptimis (ne du beveik identiški) reiškia, kad negalima atsitiktinai
//! pakeisti tik vienos pusės handshake formos.

use serde::{Deserialize, Serialize};

use crate::core::loader::LoadedGameInfo;
use crate::error::CoreError;

/// Didinamas KIEKVIENĄ kartą, kai keičiasi `EmuCommand`/`EmuStatus`/`IpcHello` laido formatas
/// (nauja privaloma reikšmė, pašalintas variantas, pervardytas laukas) — ne kiekvieną kartą,
/// kai pridedamas naujas OPCIONALUS/atgal-suderinamas variantas.
pub const IPC_PROTOCOL_VERSION: u32 = 1;

/// Protokolo versijos handshake — pirma žinutė, kurią kiekviena pusė siunčia SAVO, ir pirma
/// žinutė, kurią kiekviena pusė TIKISI gauti iš kitos. Abi pusės naudoja TĄ PATĮ tipą (žr.
/// modulio doc), tad `serde` laukų pakeitimas automatiškai galioja abiem kryptimis vienu metu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IpcHello {
    pub protocol_version: u32,
}

impl IpcHello {
    /// Handshake žinutė su ŠIO build'o protokolo versija — siunčiama kaip pati pirma eilutė.
    pub fn current() -> Self {
        Self {
            protocol_version: IPC_PROTOCOL_VERSION,
        }
    }

    /// `true`, jei gautas handshake atitinka ŠIO build'o protokolo versiją. Nesutapimas —
    /// nesuderinamas sidecar binaras (žr. modulio doc); caller'is turėtų nutraukti ryšį su
    /// aiškiu pranešimu, NE bandyti tęsti su likusiu protokolu.
    pub fn is_compatible(&self) -> bool {
        self.protocol_version == IPC_PROTOCOL_VERSION
    }
}

/// Būvio pranešimai, siunčiami iš `nullbyte-emu` (vaikas) į `nullbyte-app` (tėvas) per stdout
/// (NDJSON, viena žinutė per eilutę — žr. modulio doc). `Error` neša [`CoreError`]
/// STRUKTŪRIŠKAI (ne suplokštintą `{kind, message}` eilutę) — leidžia tėvui atkurti visus
/// laukus (`path`/`expected`/`actual`/`bios_file` ir pan.) ir teisingai apgaubti į
/// `AppError::Core`, kurio `kind()` deleguoja į `CoreError::kind()` (žr.
/// `nullbyte-app::error::AppError` doc). Suplokštinimas vyksta TIK ties Tauri → frontend riba,
/// ne čia — priešingu atveju P4.0.1 metu pridėti konkretūs `CoreError` variantai (CoreLoad/
/// ApiVersion/RomLoad/MissingBios/UnsupportedPixelFormat) taptų bevertė taksonomija, mirštanti
/// ties šia IPC riba (P9.1/P9.3 reikalauja UI galėti šakotis pagal konkretų klaidos tipą).
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EmuStatus {
    /// `EmuCommand::Load` sėkmingai užbaigtas — core'as ir ROM'as įkelti.
    Loaded(LoadedGameInfo),
    /// Bet kokia `CoreError` — core'o įkėlimo, ROM'o įkėlimo, save state ir pan. klaida.
    /// Gija LIEKA gyva (žr. `core::runner::handle_load` doc) — šis pranešimas NEREIŠKIA
    /// proceso pabaigos, tik VIENOS operacijos nesėkmę.
    Error(CoreError),
    /// Periodinė sveikatos informacija (post-MVP HUD/diagnostikai). `audio_buffer_occupancy`
    /// — tas pats `[0.0, 1.0]` occupancy, kurį `core::runner` jau naudoja audio-driven pacing'ui
    /// viduje (CLAUDE.md §8.5/§8.6, `audio::ring::AudioConsumer::occupancy()`); kiti laukai
    /// (frame timing ir pan.) pridedami tada, kai atsiras realus vartotojas UI pusėje —
    /// NEsuprojektuoti iš anksto (žr. CLAUDE.md „Ko nedaryti" §11 dėl spekuliatyvių abstrakcijų).
    Stats { audio_buffer_occupancy: f64 },
    /// `EmuCommand::Stop` užbaigtas švariai — core'as atlaisvintas (`unload_game` →
    /// `deinit` → `drop(Library)`, žr. CLAUDE.md §8.2 žingsnis 14).
    Stopped,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_roundtrips_and_detects_mismatch() {
        let json = serde_json::to_string(&IpcHello::current()).unwrap();
        let parsed: IpcHello = serde_json::from_str(&json).unwrap();
        assert!(parsed.is_compatible());

        let stale = IpcHello {
            protocol_version: IPC_PROTOCOL_VERSION + 1,
        };
        assert!(!stale.is_compatible());
    }

    #[test]
    fn emu_status_error_carries_structured_core_error() {
        let status = EmuStatus::Error(CoreError::ApiVersion {
            path: std::path::PathBuf::from("/cores/old.dylib"),
            expected: 1,
            actual: 2,
        });
        let json = serde_json::to_string(&status).unwrap();
        let restored: EmuStatus = serde_json::from_str(&json).unwrap();
        match restored {
            EmuStatus::Error(err) => {
                assert_eq!(err.kind(), "api_version");
            }
            other => panic!("tikėtasi EmuStatus::Error, gauta {other:?}"),
        }
    }

    #[test]
    fn emu_status_loaded_roundtrips() {
        let status = EmuStatus::Loaded(LoadedGameInfo {
            fps: 60.098,
            sample_rate: 32040.0,
            base_width: 256,
            base_height: 224,
            max_width: 256,
            max_height: 239,
            aspect_ratio: -1.0,
        });
        let json = serde_json::to_string(&status).unwrap();
        let restored: EmuStatus = serde_json::from_str(&json).unwrap();
        match restored {
            EmuStatus::Loaded(info) => assert!((info.fps - 60.098).abs() < 1e-9),
            other => panic!("tikėtasi EmuStatus::Loaded, gauta {other:?}"),
        }
    }
}
