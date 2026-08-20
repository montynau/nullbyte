# CLAUDE.md — Nullbyte

> Šis failas yra instrukcija Claude Code, dirbančiam šioje repozitorijoje.
> Perskaityk jį **visą** prieš rašydamas pirmą kodo eilutę.
> Jei kažkas prieštarauja šiam failui — laimi šis failas, nebent vartotojas pasako kitaip.

---

## 1. Kas yra Nullbyte

Nullbyte — daugiaplatformis retro žaidimų emuliavimo frontend'as, sukurtas **macOS ir Linux**.
Jis pats **neemuliuoja** konsolių — jis įkelia **libretro core'us** (tas pačias `.dylib` / `.so`
bibliotekas, kurias naudoja RetroArch) ir suteikia joms modernų, greitą, gražų UI.

Idėjinis palyginimas:

| | RetroArch | OpenEmu | **Nullbyte** |
|---|---|---|---|
| Core API | libretro | sava `.oecoreplugin` API | **libretro** |
| Platformos | viskas | tik macOS | **macOS + Linux** |
| UI | savas (menu driver) | AppKit / SwiftUI | **WebView (Svelte 5)** |
| Metaduomenys | thumbnails repo | OpenVGDB | **ScreenScraper API** |
| Fokusas | maksimalus lankstumas | grožis | **grožis + modernus stack'as** |

Pagrindinis produktinis skirtumas: **video gameplay preview** bibliotekoje (hover ant žaidimo →
groja trumpas gameplay įrašas, kaip Steam / Epic Games Store).

---

## 2. Tech stack — tikslios versijos

**Nekeisk šių pasirinkimų be aiškaus vartotojo sutikimo.** Jie apsvarstyti ir užfiksuoti.

### Backend (Rust)

| Priklausomybė | Versija | Paskirtis |
|---|---|---|
| `tauri` | `2.11.x` | Aplikacijos apvalkalas, langai, IPC |
| `tauri-build` | `2.x` | Build script |
| `libloading` | `0.8` | Dinaminis libretro core'ų įkėlimas (`dlopen`) |
| `wgpu` | `26.x` | GPU vaizdo atvaizdavimas (Metal macOS / Vulkan Linux) |
| `raw-window-handle` | `0.6` | Tauri lango handle → wgpu `Surface` |
| `cpal` | `0.16` | Garso išvestis |
| `rubato` | `0.16` | Garso resampling (core sample rate → device rate) |
| `rtrb` | `0.3` | Lock-free ring buffer garsui |
| `rusqlite` | `0.32` + `bundled` | SQLite duomenų bazė |
| `gilrs` | `0.11` | Gamepad įvestis (macOS + Linux) |
| `reqwest` | `0.12` + `json`, `stream` | ScreenScraper HTTP |
| `serde` / `serde_json` | `1.x` | Serializacija |
| `tokio` | `1.x` + `rt-multi-thread`, `fs` | Async runtime (scraping, atsisiuntimai) |
| `thiserror` | `2.x` | Klaidų tipai |
| `tracing` / `tracing-subscriber` | `0.1` / `0.3` | Logging |
| `md-5`, `sha1`, `crc32fast` | — | ROM hash'ai ScreenScraper'iui |
| `walkdir` | `2.x` | ROM katalogų skenavimas |
| `zip`, `sevenz-rust` | — | Suarchyvuotų ROM'ų skaitymas |

> `bundled` feature `rusqlite` — **privaloma**, kad nereikėtų sistemos SQLite ir build'as veiktų
> vienodai macOS ir Linux.

### Frontend

| Priklausomybė | Versija | Paskirtis |
|---|---|---|
| `svelte` | `5.x` (runes) | UI framework |
| `@sveltejs/kit` | `2.x` | Routing, build |
| `@sveltejs/adapter-static` | `3.x` | **Privaloma** — Tauri nepalaiko SSR |
| `typescript` | `5.x` | Tipai |
| `tailwindcss` | `4.x` | Stiliai |
| `shadcn-svelte` | naujausia | UI komponentai (kopijuojami į projektą) |
| `bits-ui` | — | shadcn-svelte priklausomybė (headless primitives) |
| `lucide-svelte` | — | Ikonos |
| `@tauri-apps/api` | `2.x` | `invoke`, `Channel`, `event` |

### Įrankiai

- **Paketų valdyklė:** `pnpm` (ne npm, ne yarn)
- **Rust toolchain:** stable, MSRV 1.82+
- **Formatavimas:** `cargo fmt` + `prettier` (su `prettier-plugin-svelte`, `prettier-plugin-tailwindcss`)
- **Lint:** `cargo clippy -- -D warnings` + `eslint` + `svelte-check`

---

## 3. Architektūra

### 3.1 Sluoksniai

> **Nuo ADR-016 (P4.0.x) tai DU procesai**, ne vienas — žr. §3.4 pilnam kontekstui.
> Diagrama žemiau atnaujinta atitinkamai (2026-08-20).

```
┌───────────────────────────────────────────────────────────────────────────┐
│  nullbyte-app (Tauri tėvo procesas)                                        │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────┐              │
│  │  Svelte 5 UI (WebView)                                   │              │
│  │  biblioteka · nustatymai · scraping · žaidimo meniu      │              │
│  └───────────────────────┬─────────────────────────────────┘              │
│                          │  Tauri IPC (invoke / events / Channel)          │
│  ┌───────────────────────▼─────────────────────────────────┐              │
│  │  Rust — commands sluoksnis (crates/nullbyte-app/src/commands/) │        │
│  │  plonas: validacija → kviečia domeno modulius ARBA        │              │
│  │  siunčia EmuCommand vaikui per proceso IPC (žr. žemiau)   │              │
│  └───────┬───────────────────────────────────────┬──────────┘              │
│          ▼                                       ▼                        │
│  ┌───────────────┐  ┌──────────┐  ┌──────────┐   │                        │
│  │      db/      │  │ scraper/ │  │ library/ │   │                        │
│  │    rusqlite   │  │  ScreenS.│  │ skener.  │   │                        │
│  │    SQLite     │  │   HTTP   │  │ + hash   │   │                        │
│  └───────────────┘  └──────────┘  └──────────┘   │                        │
└────────────────────────────────────────────────────┼──────────────────────┘
                                                       │ IPC (plona riba — TIK
                                                       │ EmuCommand/EmuStatus,
                                                       │ NIEKADA vaizdas/garsas)
┌──────────────────────────────────────────────────────▼──────────────────────┐
│  nullbyte-emu (vaiko procesas, winit)                                       │
│                                                                              │
│  ┌────────┐ ┌─────────┐   ┌───────────┐   ┌──────────┐  ┌──────────┐       │
│  │ core/  │ │ video/  │   │  audio/   │   │  input/  │  │  winit   │       │
│  │libretro│ │  wgpu   │   │   cpal    │   │  gilrs + │  │  event   │       │
│  │  FFI   │ │ render  │   │  + resamp │   │  klaviat.│  │  loop    │       │
│  └───┬────┘ └────▲────┘   └─────▲─────┘   └────┬─────┘  └────┬─────┘       │
│      │           │              │               │             │            │
│      │  video    │  audio       │               │  input      │            │
│      └───────────┴──────────────┘               └─────────────┘            │
│         (lock-free buferiai — VISKAS ŠIAME procese, žr. §3.4)               │
└──────────────────────────────────────────────────────────────────────────┘
```

> `crates/nullbyte-core` (bendra logika: `core/`, `video/`, `audio/`, `input/`, `error.rs`)
> naudojama IR `nullbyte-emu` (vykdo), IR `nullbyte-app` (dalinasi `EmuCommand`/`EmuStatus`
> tipais IPC'ui) — schemoje ji pavaizduota tik `nullbyte-emu` viduje, nes ten VYKDOMA.

### 3.2 Gijų (threads) modelis — KRITIŠKAI SVARBU

```
┌──────────────────┐   ┌──────────────────┐   ┌──────────────────┐
│  Main / UI gija  │   │  Emuliavimo gija │   │  Garso gija      │
│  (Tauri event    │   │  (dedikuota,     │   │  (cpal callback, │
│   loop, WebView) │   │   real-time)     │   │   real-time)     │
│                  │   │                  │   │                  │
│  · IPC komandos  │   │  · retro_run()   │   │  · pull iš ring  │
│  · lango įvykiai │   │  · frame pacing  │   │    buffer        │
│  · wgpu present  │◄──┤  · rašo į video  │◄──┤  · resampling    │
│                  │   │    triple buffer │   │                  │
│                  │   │  · rašo į audio  ├──►│                  │
│                  │   │    ring buffer   │   │                  │
└──────────────────┘   └──────────────────┘   └──────────────────┘
```

**Taisyklės, kurių NEGALIMA laužyti:**

1. **Visi `retro_*` kvietimai — tik iš emuliavimo gijos.** libretro core'ai naudoja globalų
   būvį (global state) ir nėra thread-safe.
2. **Vienu metu procese gali būti įkeltas tik VIENAS core.** Norint pakeisti core'ą — reikia
   `retro_unload_game()` → `retro_deinit()` → `Library::close()`, ir tik tada įkelti kitą.
   Kai kurie core'ai net po `dlclose` palieka nešvarų globalų būvį — jei pastebi keistą elgesį
   perjungiant core'us, sprendimas yra **atskiras child procesas** vienam core'ui (žr. §10).
3. **Garso gijoje NEGALIMA:** alokuoti atminties, imti `Mutex`, kviesti `println!`, daryti I/O.
   Tik lock-free ring buffer skaitymas.
4. **Emuliavimo gijoje NEGALIMA:** blokuoti ilgiau nei frame biudžetas (~16.6 ms), kviesti
   Tauri IPC sinchroniškai, daryti tinklo užklausų.

### 3.3 libretro callback'ų problema

libretro callback'ai yra **paprasti C funkcijų rodyklės be `user_data` parametro**:

```c
typedef void (*retro_video_refresh_t)(const void *data, unsigned width,
                                      unsigned height, size_t pitch);
```

Nėra kur perduoti `&mut self`. Todėl:

- Naudok **`thread_local!` + `RefCell`** arba globalų `OnceLock<Mutex<...>>` su būviu.
- **Pirmenybė `thread_local!`**, nes visi kvietimai vyksta iš vienos gijos → nereikia sinchronizacijos.
- **Nenaudok `static mut`** — Rust 2024 tai yra `deny`-by-default (`static_mut_refs`).

Rekomenduojamas šablonas:

```rust
thread_local! {
    static CTX: RefCell<Option<EmuContext>> = const { RefCell::new(None) };
}

unsafe extern "C" fn video_refresh_cb(
    data: *const c_void, width: u32, height: u32, pitch: usize,
) {
    if data.is_null() { return; }               // dupe frame — core prašo pakartoti kadrą
    CTX.with_borrow_mut(|ctx| {
        if let Some(ctx) = ctx.as_mut() {
            ctx.push_frame(data, width, height, pitch);
        }
    });
}
```

### 3.4 Proceso architektūra — `nullbyte-emu` vaiko procesas

Žr. **ADR-016** (MVP.md §14) pilnam kontekstui ir sprendimo priežastims. Santrauka:

- `nullbyte-app` (Tauri tėvas) paleidžia `nullbyte-emu` kaip **vaiko procesą** (Tauri
  `externalBin` sidecar) kiekvienam žaidimo paleidimui.
- `nullbyte-emu` turi SAVO winit langą, SAVO wgpu `Surface`, SAVO cpal `AudioOutput`, SAVO
  `gilrs` gamepad pump'ą — visi §3.2/§3.3 taisyklės galioja TOJE PAČIOJE prasmėje, tik dabar
  „procesas" vietoj „Tauri aplikacija".
- **Nei vaizdas, nei garsas nekerta proceso ribos.** Per IPC keliauja TIK lengvos valdymo
  žinutės (`Load`/`Pause`/`Resume`/`Stop`/`SaveState`/`SetFastForward` ir būvio pranešimai
  atgal) — `video::frame_buffer` ir `audio::ring` lieka algoritmiškai NEPAKITĘ, veikia tarp
  dviejų gijų VIENAME (`nullbyte-emu`) procese, kaip ir anksčiau.
- **Klaviatūra ir gamepad'as gyvena `nullbyte-emu`** — jokio IPC nereikia įvesčiai (winit
  langas gauna klaviatūrą tiesiogiai; `gilrs` pollina nepriklausomai nuo proceso/lango).
- Core'o perjungimas = senas vaiko procesas baigiamas, naujas paleidžiamas švariu būviu —
  R4 (core'ų globalus būvis) išspręsta STRUKTŪRIŠKAI, ne apeinant (žr. rizikų registrą §13).

---

## 4. Katalogų struktūra

> **Cargo workspace, trys crate'ai — nuo P4.3/ADR-016 (2026-08-20).** Emuliavimo langas
> (vaizdas + garsas + gamepad + klaviatūra) veikia **atskirame vaiko procese**
> (`nullbyte-emu`), ne Tauri procese — žr. §3.4 ir ADR-016 (MVP.md §14). Priežastis:
> klaviatūros įvestis (Tauri `Window` be webview neturi jokio klaviatūros event'ų API) ir
> R4 rizikos (core'ų globalus būvis) sprendimas viename architektūriniame žingsnyje.

```
nullbyte/
├── CLAUDE.md                      # šis failas
├── README.md
├── MVP.md                         # darbų planas — SEK JO EILIŠKUMO
├── Cargo.toml                     # workspace root — [workspace] members = ["crates/*"]
├── package.json
├── pnpm-lock.yaml
├── svelte.config.js               # adapter-static, ssr = false
├── vite.config.ts
├── tsconfig.json
├── .env.example                   # ScreenScraper dev credentials pavyzdys
│
├── crates/
│   ├── nullbyte-core/              # BENDRA emuliavimo logika — naudoja IR nullbyte-emu
│   │   │                           # (vykdo), IR nullbyte-app (dalinasi IPC žinučių tipais)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs           # AppError (thiserror) — bendras visiems trims crate'ams
│   │       │
│   │       ├── core/              # libretro sluoksnis (nepakitęs nuo P1.x)
│   │       │   ├── mod.rs
│   │       │   ├── ffi.rs         # libretro.h tipai, konstantos, struct'ai
│   │       │   ├── loader.rs      # libloading, simbolių gavimas, CoreHandle
│   │       │   ├── callbacks.rs   # unsafe extern "C" callback'ai + thread_local CTX
│   │       │   ├── environment.rs # retro_environment komandų apdorojimas
│   │       │   ├── runner.rs      # emuliavimo gija, frame loop, audio-driven pacing
│   │       │   ├── savestate.rs   # retro_serialize / retro_unserialize
│   │       │   └── info.rs        # .info failų parsinimas (core → sistemos, plėtiniai)
│   │       │
│   │       ├── video/
│   │       │   ├── mod.rs
│   │       │   ├── renderer.rs    # wgpu device/surface/pipeline — DABAR winit::window::Window
│   │       │   ├── frame_buffer.rs # triple buffer — LIEKA VIENAME (nullbyte-emu) procese
│   │       │   ├── pixel_format.rs # RGB565 / XRGB8888 / 0RGB1555 → RGBA8
│   │       │   └── shaders/
│   │       │       ├── blit.wgsl  # centruotas quad, aspect ratio / integer scaling
│   │       │       └── crt.wgsl   # post-MVP: CRT / scanlines
│   │       │
│   │       ├── audio/
│   │       │   ├── mod.rs
│   │       │   ├── output.rs      # cpal stream setup
│   │       │   ├── ring.rs        # rtrb producer/consumer — LIEKA VIENAME procese
│   │       │   └── resampler.rs   # rubato + dynamic rate control (P3.3/P3.4)
│   │       │
│   │       └── input/
│   │           ├── mod.rs
│   │           ├── gamepad.rs     # gilrs event pump — gyvena nullbyte-emu (low-latency)
│   │           ├── keyboard.rs    # winit KeyboardInput — DABAR realiai gaunama (ADR-016)
│   │           └── mapping.rs     # fizinis mygtukas/klavišas → RETRO_DEVICE_ID_JOYPAD_*
│   │
│   ├── nullbyte-emu/               # VAIKO procesas — emuliatoriaus langas, vykdo core'ą
│   │   ├── Cargo.toml              # priklauso nuo nullbyte-core
│   │   └── src/
│   │       ├── main.rs             # winit event loop, ActivationPolicy::Accessory (§10)
│   │       ├── ipc.rs              # IPC serveris — priima EmuCommand iš tėvo, siunčia būvį
│   │       └── orphan_guard.rs     # tėvo pipe stebėjimas — EOF → savaiminis išsijungimas
│   │
│   └── nullbyte-app/                # TĖVO procesas — Tauri, UI, DB, scraper, biblioteka
│       ├── Cargo.toml               # priklauso nuo nullbyte-core (dalinasi IPC tipais)
│       ├── build.rs
│       ├── tauri.conf.json          # bundle.externalBin → nullbyte-emu binaras (sidecar)
│       ├── capabilities/
│       │   └── default.json         # Tauri v2 permissions
│       ├── icons/
│       ├── migrations/              # SQL migracijos, numeruotos
│       │   ├── 001_initial.sql
│       │   └── 002_....sql
│       └── src/
│           ├── main.rs              # tik `nullbyte_app::run()`
│           ├── lib.rs               # Tauri builder, state, komandų registracija
│           ├── state.rs             # AppState — DABAR laiko vaiko proceso handle + IPC
│           │                        # klientą, NE Renderer/EmuThread/AudioOutput tiesiogiai
│           ├── paths.rs             # XDG / macOS katalogų sprendimas
│           │
│           ├── db/
│           │   ├── mod.rs
│           │   ├── migrations.rs    # user_version pagrįstos migracijos
│           │   ├── models.rs        # Game, SaveState, Platform, Setting struct'ai
│           │   ├── games.rs         # CRUD + paieška + filtravimas
│           │   └── settings.rs      # key/value nustatymai
│           │
│           ├── library/
│           │   ├── mod.rs
│           │   ├── scanner.rs       # ROM katalogų skenavimas, plėtinių atpažinimas
│           │   ├── hasher.rs        # CRC32 / MD5 / SHA1 (archyvams — vidinio failo hash)
│           │   └── archive.rs       # .zip / .7z skaitymas
│           │
│           ├── scraper/
│           │   ├── mod.rs
│           │   ├── screenscraper.rs # API v2 klientas
│           │   ├── types.rs         # JSON atsako struct'ai
│           │   ├── media.rs         # viršelių / video atsisiuntimas į cache
│           │   └── rate_limit.rs    # kvotų laikymasis, backoff
│           │
│           └── commands/            # PLONAS sluoksnis — jokios logikos
│               ├── mod.rs
│               ├── library.rs       # scan_roms, list_games, get_game, ...
│               ├── emulator.rs      # start_game/pause/resume/stop — paleidžia/valdo
│               │                    # nullbyte-emu vaiko procesą per IPC
│               ├── scraper.rs       # scrape_game, scrape_library, scrape_progress
│               └── settings.rs      # get_settings, set_setting, list_cores
│
├── src/                           # Svelte frontend
│   ├── app.html
│   ├── app.css                    # Tailwind v4 @import + tema (CSS kintamieji)
│   ├── lib/
│   │   ├── components/
│   │   │   ├── ui/                # shadcn-svelte (generuojami CLI — NEREDAGUOK ranka be reikalo)
│   │   │   ├── library/
│   │   │   │   ├── GameCard.svelte
│   │   │   │   ├── GameGrid.svelte
│   │   │   │   ├── VideoPreview.svelte
│   │   │   │   └── PlatformFilter.svelte
│   │   │   ├── layout/
│   │   │   │   ├── Sidebar.svelte
│   │   │   │   ├── TopBar.svelte
│   │   │   │   └── CommandPalette.svelte
│   │   │   └── settings/
│   │   │       ├── CoresPanel.svelte
│   │   │       ├── PathsPanel.svelte
│   │   │       ├── InputPanel.svelte
│   │   │       └── ScraperPanel.svelte
│   │   ├── stores/                # Svelte 5 runes (.svelte.ts failai)
│   │   │   ├── library.svelte.ts
│   │   │   ├── emulator.svelte.ts
│   │   │   └── settings.svelte.ts
│   │   ├── api/
│   │   │   ├── index.ts           # tipizuoti invoke wrapper'iai
│   │   │   └── events.ts          # Tauri event listener'iai
│   │   ├── types/
│   │   │   └── index.ts           # BENDRI tipai su Rust (žr. §7.3)
│   │   └── utils/
│   │       ├── format.ts
│   │       └── platforms.ts       # platformų metaduomenys, spalvos, ikonos
│   └── routes/
│       ├── +layout.svelte
│       ├── +layout.ts             # export const ssr = false; prerender = true
│       ├── +page.svelte           # biblioteka
│       ├── game/[id]/+page.svelte # žaidimo detalės
│       └── settings/+page.svelte
│
└── static/
```

> **Istorinė pastaba:** iki P4.3/ADR-016 visas Rust kodas gyveno viename `src-tauri/`
> crate'e (žr. git istoriją P0.1–P4.1 commit'uose) — vaizdas/garsas/emuliavimas veikė TAME
> PAČIAME procese kaip Tauri UI, per `tauri::window::Window` be webview (ADR-005). Ta
> architektūra susidūrė su dviem neišsprendžiamomis kliūtimis: (1) tokia `Window` neturi
> jokio klaviatūros event'ų API Tauri v2 (patikrinta prieš `tauri` 2.11.5 šaltinį — tik
> `on_window_event`/`on_menu_event`, jokio klaviatūros varianto `WindowEvent` enum'e), (2) R4
> rizika (core'ų globalus būvis neleidžia perjungti core'o be restarto) neturėjo sprendimo
> iki po-MVP. ADR-016 sprendžia abi vienu žingsniu — žr. §3.4 ir MVP.md §14.

---

## 5. Komandos

> **Nuo P4.0.1/ADR-016 (2026-08-20):** repo tapo Cargo workspace'u (žr. §4) — `cargo`
> komandos žemiau paleidžiamos IŠ REPO ŠAKNIES su `--workspace`, BE `--manifest-path`
> (workspace root `Cargo.toml` automatiškai apima visus tris crate'us). `pnpm tauri dev`/
> `build` komandų TIKSLUS iškvietimas (kaip nurodyti, kad `tauri.conf.json` dabar
> `crates/nullbyte-app/`, ne `src-tauri/`) **DAR NEPATIKRINTAS realiu build'u** — tai
> P4.0.1/P4.0.5 darbas. Žemiau — geriausia žinoma prielaida (Tauri CLI `--config`), pažymėta
> aiškiai; nepasikliauk ja aklai, kol P4.0.5 acceptance to nepatvirtins.

```bash
# Setup (vieną kartą)
pnpm install
rustup target add aarch64-apple-darwin x86_64-apple-darwin   # tik macOS

# Kūrimas
pnpm tauri dev                     # TIKSLUS iškvietimas po workspace split'o — NEPATIKRINTA,
                                    # žr. pastabą aukščiau (galimai reikės --config nuorodos į
                                    # crates/nullbyte-app/tauri.conf.json — P4.0.1/P4.0.5)
pnpm dev                           # tik frontend (be Tauri — daugumai UI darbų greičiau,
                                    # nepaveikta workspace split'o, frontend'as lieka repo šaknyje)

# Kokybė — PALEISK PRIEŠ KIEKVIENĄ COMMIT (iš repo šaknies)
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm check                         # svelte-check + tsc
pnpm lint                          # eslint + prettier --check
pnpm format                        # prettier --write

# Build
pnpm tauri build                   # dabartinei platformai — ta pati NEPATIKRINTA pastaba
pnpm tauri build --target universal-apple-darwin   # macOS universal binary
```

---

## 6. Rust konvencijos

### 6.1 Bendra

- **Jokių `unwrap()` / `expect()` produkciniame kelyje.** Leidžiama tik testuose ir
  `lib.rs` starto sekoje, kur klaida = programa negali veikti.
- Visos viešos funkcijos grąžina `Result<T, AppError>`.
- `AppError` — vienas `thiserror` enum'as; jis serializuojasi į UI kaip
  `{ kind: string, message: string }`.
- Moduliai bendrauja per **savo tipus**, ne per `serde_json::Value`.
- Kiekvienas `pub fn` turi doc komentarą (`///`), jei nėra visiškai akivaizdus.

### 6.2 `unsafe` taisyklės

- `unsafe` blokai — **tik** `core/ffi.rs`, `core/loader.rs`, `core/callbacks.rs`.
- Virš kiekvieno `unsafe` bloko — komentaras `// SAFETY: ...` paaiškinantis, kodėl tai saugu.
- Visos FFI struct'ai — `#[repr(C)]`.
- Rodyklių dereferencing'as — tik po `is_null()` patikros.
- Visi FFI tipai turi būti tikslūs pagal `libretro.h`. Jei abejoji dėl tipo dydžio —
  parašyk `const _: () = assert!(size_of::<X>() == N);` compile-time patikrą.

### 6.3 Tauri komandos

```rust
#[tauri::command]
pub async fn list_games(
    state: tauri::State<'_, AppState>,
    filter: GameFilter,
) -> Result<Vec<Game>, AppError> {
    state.db.games().list(&filter).await
}
```

- Komandos yra **plonos**: validacija → domeno modulis → grąžina.
- Ilgai trunkančios operacijos (skenavimas, scraping) turi būti **async** ir siųsti progresą
  per `tauri::ipc::Channel<Progress>`, ne blokuoti IPC.
- Visos komandos registruojamos `lib.rs` viename `generate_handler![]`.

### 6.4 Logging

- `tracing` visur. `tracing::info!` gyvenimo ciklo įvykiams, `debug!` detalėms,
  `warn!`/`error!` problemoms.
- **Nieko nelogink emuliavimo loop'e per kadrą** — tik pirmą kartą arba kas N sekundžių.

---

## 7. Frontend konvencijos

### 7.1 Svelte 5 runes

Naudok **tik runes sintaksę**, ne senąją Svelte 4:

```svelte
<script lang="ts">
  let { game }: { game: Game } = $props();
  let hovered = $state(false);
  let coverUrl = $derived(game.coverPath ?? placeholderFor(game.platform));

  $effect(() => {
    if (hovered) startPreview();
    return () => stopPreview();
  });
</script>
```

- `$state` vietoj `let` reaktyvumui
- `$derived` vietoj `$:`
- `$props()` vietoj `export let`
- `$effect` vietoj `onMount` + `$:` kombinacijų (`onMount` vis dar OK vienkartiniam setup'ui)
- Store'ai — `.svelte.ts` failuose su `$state` klasėse arba objektuose

### 7.2 SvelteKit + Tauri

`src/routes/+layout.ts` **privalo** turėti:

```ts
export const ssr = false;
export const prerender = true;
```

Tauri serviruoja statinius failus — jokio serverio nėra. Jokių `+page.server.ts`, jokių
form actions, jokio `fetch` į savo backend'ą — viskas per `invoke`.

### 7.3 Tipų sinchronizacija Rust ↔ TypeScript

Rust struct'ai, kurie kerta IPC ribą, **privalo** turėti:

```rust
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Game { /* ... */ }
```

TypeScript pusėje — rankiniu būdu atitinkantis interface `src/lib/types/index.ts`.
**Kai keiti Rust struct'ą, kuris kerta IPC — TUOJ PAT atnaujink TS tipą.** Tai dažniausia
tylių bug'ų priežastis šiame projekte.

> Post-MVP galima automatizuoti su `ts-rs` crate'u. MVP metu — rankiniu būdu, bet drausmingai.

### 7.4 UI komponentai

- shadcn-svelte komponentai pridedami per CLI: `pnpm dlx shadcn-svelte@latest add button dialog ...`
- Jie atsiranda `src/lib/components/ui/` — tai **tavo kodas**, gali redaguoti, bet
  atskirk savo pakeitimus nuo generuoto kodo komentarais.
- Sava logika — **niekada** ne `ui/` kataloge, o `library/`, `layout/`, `settings/`.

### 7.5 Dizaino kryptis

- **Tamsi tema kaip numatytoji.** Šviesi — post-MVP.
- Estetika: modernus, tankus, klaviatūra valdomas (kaip Linear / Arc / Raycast),
  ne „gaming RGB".
- Žaidimų grid'as — viršeliai dominuoja, tekstas minimalus.
- Hover → video preview su 300 ms delsa (kad neblyksėtų slenkant pele).
- Visos spalvos per Tailwind v4 CSS kintamuosius `app.css` — jokių hardcode'intų hex reikšmių
  komponentuose.

---

## 8. Domeno žinios — libretro

### 8.1 Simbolių sąrašas, kuriuos reikia gauti iš core

```
retro_api_version()               -> u32     (privalo grąžinti 1)
retro_set_environment(cb)
retro_set_video_refresh(cb)
retro_set_audio_sample(cb)
retro_set_audio_sample_batch(cb)
retro_set_input_poll(cb)
retro_set_input_state(cb)
retro_init()
retro_deinit()
retro_get_system_info(*mut retro_system_info)
retro_get_system_av_info(*mut retro_system_av_info)
retro_set_controller_port_device(port, device)
retro_load_game(*const retro_game_info) -> bool
retro_unload_game()
retro_run()
retro_reset()
retro_serialize_size()            -> usize
retro_serialize(*mut c_void, usize)   -> bool
retro_unserialize(*const c_void, usize) -> bool
retro_get_memory_data(id)         -> *mut c_void   (SRAM išsaugojimui)
retro_get_memory_size(id)         -> usize
retro_get_region()                -> u32
```

### 8.2 Privalomas iškvietimo eiliškumas

```
1. Library::new(path)
2. retro_api_version()            → tikrink == 1, kitaip atmesk core'ą
3. retro_set_environment(cb)      → PIRMAS callback, core čia klausia savybių
4. retro_set_video_refresh(cb)
5. retro_set_input_poll(cb)
6. retro_set_input_state(cb)
7. retro_set_audio_sample(cb)
8. retro_set_audio_sample_batch(cb)
9. retro_init()
10. retro_get_system_info()       → palaikomi plėtiniai, ar reikia pilno kelio
11. retro_load_game(&game_info)
12. retro_get_system_av_info()    → TIK PO load_game! Čia gauni tikslų fps ir sample_rate
13. loop { retro_run() }
14. retro_unload_game() → retro_deinit() → drop(Library)
```

> **Dažna klaida:** kviesti `retro_get_system_av_info()` prieš `retro_load_game()`.
> Kai kurie core'ai (pvz. Mednafen) grąžina neteisingus duomenis, nes AV info priklauso nuo ROM'o.

### 8.3 `retro_environment` komandos, kurias PRIVALOMA apdoroti MVP metu

| Komanda | ID | Ką daryti |
|---|---|---|
| `SET_PIXEL_FORMAT` | 10 | Įsimink formatą. Palaikom: `0RGB1555`(0), `XRGB8888`(1), `RGB565`(2). Grąžink `true`. |
| `GET_SYSTEM_DIRECTORY` | 9 | Grąžink kelią iki BIOS katalogo (`CString`, gyvuoja visą sesiją). |
| `GET_SAVE_DIRECTORY` | 31 | Grąžink kelią iki SRAM katalogo. |
| `GET_CAN_DUPE` | 3 | Grąžink `true` (palaikom `data == NULL` kadrus). |
| `SET_VARIABLES` / `SET_CORE_OPTIONS*` | 16 / 53,54,55,67,68,69 | MVP: išsaugok, grąžink `true`. UI — post-MVP. |
| `GET_VARIABLE` | 15 | Grąžink numatytąją reikšmę arba `NULL`. |
| `GET_VARIABLE_UPDATE` | 17 | Grąžink `false`. |
| `SET_MESSAGE` | 6 | Persiųsk į UI kaip toast. |
| `GET_LOG_INTERFACE` | 27 | Duok savo log callback → `tracing`. **Labai padeda debug'inant.** |
| `SHUTDOWN` | 7 | Sustabdyk emuliavimą švariai. |
| `SET_PERFORMANCE_LEVEL` | 8 | Ignoruok, grąžink `true`. |
| `GET_LANGUAGE` | 39 | Grąžink `RETRO_LANGUAGE_ENGLISH` (0). |
| `GET_INPUT_BITMASKS` | 51 \| EXPERIMENTAL | Grąžink `true` — greitesnis input polling. |
| `SET_HW_RENDER` | 14 | **SĄMONINGAI grąžink `false` MVP metu** (`environment.rs` `_ =>` numatytoji šaka — jokios atskiros logikos NEREIKIA rašyti). Core'ai, prašantys HW render (Mupen64Plus-Next, ParaLLEl N64, Dolphin, PPSSPP), arba nepasileis, arba kris į nenaudojamą fallback'ą — tai ŽINOMAS, dokumentuotas apribojimas (žr. MVP.md §15 v0.2 „Hardware-rendered core'ų palaikymas" ir README platformų lentelę), NE praleista klaida. |

> `EXPERIMENTAL = 0x10000` (bitų žymė, pridedama prie bazinio ID). Visos ID reikšmės čia
> patikrintos prieš tikrą `libretro.h` (RetroArch/master) — ankstesnė šios lentelės versija
> turėjo klaidingas `GET_CAN_DUPE`/`SHUTDOWN`/`GET_INPUT_BITMASKS` reikšmes.

Visoms kitoms komandoms grąžink `false`. **Niekada nepanikuok (`panic!`) environment callback'e** —
tai kirstų per FFI ribą ir yra UB. Naudok `catch_unwind` arba tiesiog venk panic'ų.

### 8.4 Vaizdo formatai

Core paduoda `pitch` **baitais**, ne pikseliais. Eilutės gali turėti padding'ą.

```rust
// Teisinga:
for y in 0..height {
    let row = unsafe { data.add(y * pitch) };  // pitch baitais!
    // konvertuok `width` pikselių iš `row`
}
```

Konversijos, kurių reikia:
- `RGB565` (2 baitai/px) → RGBA8 — dažniausias
- `XRGB8888` (4 baitai/px) → RGBA8 — reikia BGRA→RGBA swizzle
- `0RGB1555` (2 baitai/px) → RGBA8 — seni core'ai

**Optimizacija (post-MVP):** konvertuok GPU pusėje shader'yje, o ne CPU.

### 8.5 Timing

- `retro_system_av_info.timing.fps` — tikrasis kadrų dažnis, **ne 60.0**
  (SNES: 60.098, NES NTSC: 60.0988, GB: 59.727, PAL sistemos: ~50.0).
- `retro_system_av_info.timing.sample_rate` — core garso dažnis
  (SNES: 32040.0, Genesis: 44100.0, GBA: 32768.0).
- **MVP sinchronizacijos strategija: audio-driven.** Garso plokštė yra laikrodis;
  emuliavimo gija generuoja kadrus tokiu greičiu, kad garso ring buffer'is liktų ~50% pilnas.
  Tai eliminuoja traškesius ir yra paprasčiau nei VRR/vsync derinimas.

### 8.6 Garso resampling ir dynamic rate control

```
core_rate (32040) → rubato → device_rate (48000)
```

Ratio korekcija pagal buffer occupancy:

```rust
let occupancy = ring.slots_filled() as f64 / ring.capacity() as f64;  // 0.0..1.0
let deviation = (occupancy - 0.5) * 2.0;            // -1.0..1.0
let ratio = base_ratio * (1.0 + MAX_DELTA * deviation);  // MAX_DELTA ≈ 0.005
```

Tai standartinė RetroArch technika. Be jos girdėsi periodinius traškesius.

### 8.7 Save states

- `retro_serialize_size()` gali grąžinti **skirtingą dydį** skirtingais momentais —
  visada kviesk prieš kiekvieną išsaugojimą.
- Save state'ai **nėra suderinami** tarp core versijų. Saugok metaduomenyse core pavadinimą
  ir versiją; įkeliant — perspėk vartotoją, jei nesutampa.
- `retro_serialize` / `retro_unserialize` **privalo** būti kviečiami iš emuliavimo gijos,
  tarp `retro_run()` kvietimų, ne jų viduryje.

### 8.8 SRAM

Atskirai nuo save state'ų: `retro_get_memory_data(RETRO_MEMORY_SAVE_RAM = 0)` +
`retro_get_memory_size(0)`. Išsaugok į `.srm` failą uždarant žaidimą ir kas ~30 s.
Įkelk po `retro_load_game()`.

---

## 9. Domeno žinios — ScreenScraper

### 9.1 API

**Endpoint:** `https://www.screenscraper.fr/api2/jeuInfos.php`

**Parametrai:**

| Parametras | Privalomas | Aprašymas |
|---|---|---|
| `devid` | taip | Developer ID (registruojamas ScreenScraper svetainėje) |
| `devpassword` | taip | Developer slaptažodis |
| `softname` | taip | `Nullbyte` + versija |
| `output` | taip | `json` |
| `ssid` | ne | Vartotojo login (be jo — labai maža kvota) |
| `sspassword` | ne | Vartotojo slaptažodis |
| `crc` | ne* | ROM CRC32 (hex) |
| `md5` | ne* | ROM MD5 |
| `sha1` | ne* | ROM SHA1 |
| `romnom` | ne* | ROM failo pavadinimas (fallback, jei hash nerado) |
| `romtaille` | ne | ROM dydis baitais — labai pagerina tikslumą |
| `systemeid` | ne | ScreenScraper platformos ID |

\* Bent vienas iš `crc`/`md5`/`sha1`/`romnom` privalomas.

**Strategija:** pirma bandyk hash'ais (`crc` + `md5` + `sha1` + `romtaille`), tik nepavykus —
`romnom` + `systemeid`.

> Suarchyvuotiems ROM'ams (`.zip`, `.7z`) hash'uok **vidinį failą**, ne archyvą.

### 9.2 Media tipai

| Tipas | Reikšmė | Naudojimas Nullbyte |
|---|---|---|
| `box-2D` | 2D viršelis | pagrindinis GameCard vaizdas |
| `box-3D` | 3D viršelis | alternatyva nustatymuose |
| `ss` | screenshot | detalių puslapis |
| `sstitle` | title screen | fallback |
| `wheel` | logotipas su permatomu fonu | overlay ant hero |
| `video` | gameplay video | **hover preview** |
| `video-normalized` | normalizuotas video | **pirmenybė** — mažesnis, vienodesnis |
| `screenmarquee` | marquee | post-MVP |

Regionų prioritetas (numatytasis): `wor` → `eu` → `us` → `jp` → `ss`.
Kalbų prioritetas aprašymams: `en` → `lt` (jei bus) → pirmas prieinamas.

### 9.3 Kvotos ir rate limiting — PRIVALOMA

ScreenScraper griežtai riboja:
- Užklausų kiekį per dieną (priklauso nuo vartotojo lygio; anonimams — beveik nulis)
- Vienalaikių gijų skaičių (`maxthreads` grąžinamas atsake `ssuser.maxthreads`)

**Taisyklės kode:**
1. **Visada cache'uok** — prieš užklausą tikrink SQLite. Sėkmingi ir nesėkmingi rezultatai
   cache'uojami (nesėkmingi — su TTL, pvz. 7 dienos).
2. **Gerbk `maxthreads`** — semaforas, ne daugiau nei serveris leidžia (numatytoji 1).
3. **Exponential backoff** ties HTTP 429/430 ir ties `API closed` atsaku.
4. **Niekada neskenuok visos bibliotekos automatiškai** — tik vartotojui paspaudus.
5. Dev credentials — iš `.env` / nustatymų, **niekada nehardcode'ink į repo**.

### 9.4 Media cache

```
{app_data}/media/
├── covers/{game_id}.{ext}
├── screenshots/{game_id}.{ext}
├── wheels/{game_id}.png
└── videos/{game_id}.mp4
```

DB laiko tik **santykinius kelius**, ne absoliučius — kad veiktų perkėlus profilį.

---

## 10. Spąstai (gotchas)

### Rust / FFI
- **`static mut` uždrausta** Rust 2024 edition'e → naudok `thread_local!` arba `OnceLock`.
- **Panic per FFI ribą = UB.** Callback'uose venk `unwrap()`, indeksavimo be patikros, `assert!`.
- **`CString` gyvavimo trukmė:** jei atiduodi core'ui `*const c_char` (pvz. system directory),
  `CString` privalo gyventi tiek pat, kiek core. Laikyk ją struct'e, ne lokaliai.
- **`dlclose` ir globalus būvis:** kai kurie core'ai neatstato globalaus būvio, jei bandai
  perjungti core'ą TAME PAČIAME procese. **IŠSPRĘSTA nuo ADR-016** (P4.3): kiekvienas žaidimo
  paleidimas = naujas `nullbyte-emu` vaiko procesas, senas užbaigiamas prieš tai — nešvarus
  būvis tiesiog dingsta kartu su procesu. Nebereikia „neleisk perjungti be restarto" apribojimo.

### wgpu / winit (`nullbyte-emu` vaiko procesas)
- **Emuliatoriaus langas — winit, ne Tauri `Window`.** Nuo ADR-016 emuliatoriaus vaizdas
  veikia atskirame vaiko procese (`nullbyte-emu`) su SAVO winit event loop'u, ne Tauri
  `Window` be webview. Priežastis: Tauri `Window` (patikrinta prieš `tauri` 2.11.5 šaltinį)
  neturi JOKIO klaviatūros event'ų API — tik `on_window_event`/`on_menu_event`, jokio
  klaviatūros varianto `WindowEvent` enum'e (žr. §3.4, ADR-016).
- **macOS Dock:** winit numatytai naudoja `ActivationPolicy::Regular` — vaiko procesas
  atsirastų Dock'e kaip ANTRA programa. Naudok
  `EventLoopBuilderExtMacOS::with_activation_policy(ActivationPolicy::Accessory)`.
- **Linux/Wayland:** wgpu Vulkan backend'as gali reikalauti `WAYLAND_DISPLAY` handling'o.
  Testuok ir X11, ir Wayland. Jei Wayland problemiškas — leisk force'inti X11 per `WINIT_UNIX_BACKEND=x11`.
- **`Surface` turi būti kuriamas VAIKO PROCESO main gijoje** (macOS reikalavimas — `NSView`
  prieinamas tik main thread; dabar tai `nullbyte-emu` proceso main thread, ne Tauri).
  Emuliavimo gija tik **rašo pikselius į buferį**; `queue.write_texture` + `present` daromas
  vaiko proceso main gijoje.
- **macOS:** wgpu Metal backend'as reikalauja, kad lango dydžio keitimas ir surface
  rekonfigūracija vyktų main thread'e.

### Proceso architektūra (`nullbyte-emu` ↔ `nullbyte-app`, ADR-016)
- **Našlaičių (orphan) procesai:** jei `nullbyte-app` (tėvas) staiga krenta, `nullbyte-emu`
  (vaikas) Unix'e NEMIRŠTA automatiškai kartu — liktų veikiantis fone. **Nenaudok PID
  pollinimo** (nepatikima, race'inama). Vietoj to: tėvas laiko atvirą pipe'ą į vaiką kaip
  gyvumo signalą; vaikas jį skaito fone atskiroje gijoje — kai tėvas (net ir netikėtai)
  baigiasi, OS uždaro pipe'ą ir vaikas gauna EOF → švariai išsijungia pats.
- **IPC riba turi likti PLONA.** Per ją keliauja tik valdymo žinutės (`EmuCommand`-like) ir
  būvio pranešimai — NIEKADA vaizdo kadrai ar audio sample'ai (tam nėra reikalo, nes vaikas
  turi savo langą/garso įrenginį — žr. §3.4). Jei prireikia siųsti kadrą per IPC (pvz.
  bibliotekos preview'ui) — tai atskiras, retas, apgalvotas atvejis, ne bendra taisyklė.

### Tauri IPC
- **Nesiųsk kadrų per `invoke`.** JSON serializacija 60 kartų per sekundę užmuš aplikaciją.
- Dideliems binariniams duomenims (jei visgi reikės) — `tauri::ipc::Response::new(bytes)`
  arba `Channel<&[u8]>`, ne `serde_json`.
- Viršeliai ir video į UI paduodami per `asset:` protokolą (`convertFileSrc`), ne base64.
  Nepamiršk `assetProtocol.scope` `tauri.conf.json`.

### SQLite
- `rusqlite` `Connection` **nėra `Sync`**. Naudok `Mutex<Connection>` arba
  `r2d2_sqlite` pool'ą. MVP: vienas `Mutex<Connection>` `AppState` viduje — pakanka.
- Įjunk `PRAGMA journal_mode = WAL;` ir `PRAGMA foreign_keys = ON;` prie kiekvieno prisijungimo.
- Bibliotekos skenavimas — **viena transakcija** visiems įrašams, ne po vieną `INSERT`.

### Svelte
- Video preview: `<video>` elementas su `muted`, `loop`, `playsinline` ir `preload="none"`.
  Be `muted` naršyklė neleis autoplay.
- Grid'as su 1000+ žaidimų — **privaloma virtualizacija** (pvz. `@tanstack/svelte-virtual`),
  kitaip DOM'as žlugs.
- `$effect` grąžina cleanup funkciją — naudok ją video sustabdymui, kitaip liks groti fone.

---

## 11. Ko NEDARYTI

1. ❌ **Nerašyk savo emuliavimo kodo.** Jokių CPU emuliatorių, jokių PPU implementacijų.
   Nullbyte yra frontend'as. Visa emuliacija — libretro core'uose.
2. ❌ **Neplatink libretro core'ų repozitorijoje.** `crates/nullbyte-core/cores/` yra `.gitignore`.
   Vartotojas atsisiunčia core'us pats arba per built-in downloader'į (post-MVP).
3. ❌ **Nepridėk ROM atsisiuntimo, torrent'ų, ROM nuorodų ar bet kokio ROM šaltinio.**
   Nullbyte niekada nedistribuoja žaidimų. README aiškiai tai sako.
4. ❌ **Nekeisk stack'o** (Tauri/Svelte/wgpu/cpal/rusqlite/ScreenScraper) be vartotojo sutikimo.
5. ❌ **Nedaryk Windows palaikymo MVP metu.** Tik macOS + Linux. Kodas turi būti
   platformai neutralus, bet netestuok ir nedebug'ink Windows.
6. ❌ **Nenaudok Svelte 4 sintaksės** (`export let`, `$:`, senų store'ų).
7. ❌ **Nedėk verslo logikos į `commands/`** ir nedėk Tauri priklausomybių į domeno modulius.
8. ❌ **Nekurk naujų priklausomybių** be įrašo į MVP.md sprendimų žurnalą.
9. ❌ **Necommit'ink** `.env`, ScreenScraper credentials, ROM'ų, core'ų, `target/`, `node_modules/`.
10. ❌ **Nepraleisk `clippy`.** `-D warnings` yra privaloma; jei warning'as nepagrįstas —
    `#[allow(...)]` su komentaru kodėl.

---

## 12. Definition of Done

Užduotis laikoma baigta tik kai **visi** punktai įvykdyti:

- [ ] Kodas kompiliuojasi be warning'ų (`cargo clippy --workspace --all-targets -- -D warnings`,
      `pnpm check`) — žr. §5 pastabą dėl workspace komandų nuo P4.0.1
- [ ] `cargo fmt --all` ir `prettier` pritaikyti
- [ ] Nauja logika turi bent vieną testą (`cargo test --workspace`) — išskyrus grynai UI komponentus
- [ ] Jei keitei IPC struct'ą — TS tipas `src/lib/types/index.ts` atnaujintas
- [ ] Jei pridėjai priklausomybę arba architektūrinį sprendimą — naujas ADR įrašas MVP.md §14
- [ ] Jei keitei DB schemą — nauja migracija `crates/nullbyte-app/migrations/`, ne senos redagavimas
- [ ] Patikrinta rankiniu būdu (`pnpm tauri dev`) — bent viename iš macOS/Linux
- [ ] MVP.md atitinkama užduotis pažymėta `[x]`

---

## 13. Git konvencijos

- Šakos: `feat/...`, `fix/...`, `refactor/...`, `docs/...`
- Commit'ai — Conventional Commits:
  ```
  feat(core): add libretro environment callback handling
  fix(audio): correct ring buffer underrun on startup
  docs(mvp): mark phase 2 complete
  ```
- Vienas commit'as = vienas logiškas pakeitimas. Nemaišyk formatavimo su logika.
- **Necommit'ink į `main` tiesiogiai**, jei vartotojas nepaprašė.

---

## 14. Kai nežinai ką daryti

1. Pažiūrėk **MVP.md** — ten yra užduočių eiliškumas. Imk pirmą nepažymėtą.
2. Jei užduotis neaiški — paklausk vartotojo, **nespėliok** architektūrinių sprendimų.
3. Jei reikia libretro detalių — autoritetingas šaltinis yra
   `RetroArch/libretro-common/include/libretro.h` (GitHub) ir https://docs.libretro.com.
4. Jei kažkas neveikia po 2–3 bandymų — sustok, paaiškink ką bandei, ir paklausk.
   **Nekartok to paties bandymo ciklu.**

---

## 15. Nuorodos

- libretro API header: https://github.com/libretro/RetroArch/blob/master/libretro-common/include/libretro.h
- libretro dokumentacija: https://docs.libretro.com/development/cores/developing-cores/
- Tauri v2: https://v2.tauri.app
- Svelte 5: https://svelte.dev/docs/svelte/overview
- shadcn-svelte: https://shadcn-svelte.com
- wgpu: https://wgpu.rs
- ScreenScraper: https://www.screenscraper.fr
- Referencinė Rust libretro frontend implementacija: https://www.retroreversing.com/CreateALibRetroFrontEndInRust
