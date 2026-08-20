# MVP.md — Nullbyte darbų planas

> Šis failas yra **vykdomasis planas**. Užduotys eina **griežtu eiliškumu** — kiekviena fazė
> remiasi ankstesne.
>
> **Claude Code: kiekvienos sesijos pradžioje** perskaityk §0.1, rask pirmą nepažymėtą užduotį,
> ir dirbk tik su ja. Baigęs — pažymėk `[x]` ir atnaujink §12 progreso lentelę.

---

## 0. Kaip naudotis šiuo failu

### 0.1 Sesijos pradžios rutina

1. Perskaityk `CLAUDE.md` (bent §2, §3, §10, §11, §12)
2. Atsidaryk šį failą, rask **pirmą** užduotį be `[x]`
3. Patikrink jos **Priklausomybės** — ar visos jos jau `[x]`? Jei ne — grįžk atgal
4. Įvykdyk užduotį
5. Patikrink **Acceptance criteria** — visus punktus
6. Paleisk `Definition of Done` patikras (`CLAUDE.md` §12)
7. Pažymėk `[x]`, atnaujink §12 progreso lentelę, commit'ink

### 0.2 Užduoties anatomija

```
### PX.Y — Užduoties pavadinimas          [ ]
Priklausomybės: PX.(Y-1)
Failai: crates/<crate>/src/kelias/i/faila.rs  (crate = nullbyte-core | nullbyte-app | nullbyte-emu — žr. CLAUDE.md §4)
Ką daryti:
  - konkretūs žingsniai
Acceptance:
  - [ ] patikrinamas rezultatas
```

### 0.3 Ženklai

| Ženklas | Reikšmė |
|---|---|
| `[ ]` | Nepradėta |
| `[~]` | Vykdoma |
| `[x]` | Baigta ir patikrinta |
| `[!]` | Užblokuota — žr. komentarą po užduotimi |
| 🔴 | Didelė rizika / sudėtinga |
| 🟡 | Vidutinė rizika |

---

## 1. MVP apibrėžimas

### 1.1 Tikslas vienu sakiniu

> Nullbyte v0.1 leidžia vartotojui nuskenuoti ROM katalogą, pamatyti gražią biblioteką su
> viršeliais ir gameplay video, ir paleisti bet kurį žaidimą su tinkamu vaizdu, garsu ir
> gamepad valdymu — macOS ir Linux.

### 1.2 MVP apimtis — ĮEINA

| Sritis | Kas įeina |
|---|---|
| **Core'ai** | Bet koks libretro core iš disko. Aptikimas, įkėlimas, sistemos↔core mapping'as |
| **Emuliavimas** | Paleidimas, pauzė, tęsimas, reset, sustabdymas |
| **Vaizdas** | wgpu atvaizdavimas atskirame lange. RGB565 / XRGB8888 / 0RGB1555. Aspect ratio, integer scaling, fullscreen |
| **Garsas** | cpal išvestis, resampling, dynamic rate control, audio-driven sync |
| **Įvestis** | Gamepad (gilrs) + klaviatūra. Numatytasis mapping'as + perrišimas UI |
| **Biblioteka** | ROM katalogų skenavimas, `.zip`/`.7z`, hash'avimas, SQLite saugojimas |
| **Metaduomenys** | ScreenScraper: pavadinimas, viršelis, screenshot, aprašymas, data, žanras, **video** |
| **UI** | Biblioteka (grid + hover video), žaidimo detalės, nustatymai, paieška, filtrai |
| **Išsaugojimai** | Save states (4 slot'ai + quick save/load), automatinis SRAM |
| **Platformos** | macOS (universal) + Linux (x86-64) |

### 1.3 MVP apimtis — NEĮEINA

| Neįeina | Kodėl / kada |
|---|---|
| Windows | v0.3 — kodas neutralus, bet netestuojam |
| Core options UI | v0.2 — MVP naudoja core numatytąsias reikšmes |
| Shader'iai (CRT, xBRZ) | v0.2 — MVP tik nearest/linear filtras |
| Netplay | v0.2 |
| Rewind | v0.2 |
| RetroAchievements | v0.2 |
| Core downloader | v0.3 — MVP vartotojas deda core'us ranka |
| Playlist'ai / kolekcijos | v0.3 |
| Šviesi tema | v0.3 |
| Lokalizacija | v0.3 |
| Cheat'ai | v0.3 |
| Disk control (multi-disk PS1) | v0.2 |
| Vibracija / rumble | v0.2 |
| Automatinis atnaujinimas | v0.2 |

### 1.4 Sėkmės kriterijai

MVP laikomas baigtu, kai **visi** šie scenarijai veikia be crash'ų:

1. ✅ Švari instaliacija → pridedu ROM katalogą su 100+ žaidimų → skenavimas baigiasi
   per < 60 s → visi žaidimai matomi su teisingomis platformomis
2. ✅ Paspaudžiu „Scrape library" → viršeliai ir video atsisiunčia → hover ant kortelės
   groja gameplay video
3. ✅ Paleidžiu SNES žaidimą → vaizdas teisingas (be spalvų iškraipymų, be plyšimų) →
   garsas be traškesių 10+ minučių → gamepad reaguoja be juntamos delsos
4. ✅ Save state slot 1 → išjungiu žaidimą → paleidžiu vėl → load state → tęsiu iš to paties taško
5. ✅ Žaidžiu RPG 5 min → uždarau → paleidžiu → SRAM išsaugotas, in-game save veikia
6. ✅ Perjungiu tarp 4 skirtingų platformų žaidimų iš eilės → visi paleidžia teisingai
7. ✅ Tas pats veikia ir macOS (Apple Silicon), ir Linux (Wayland arba X11)

---

## 2. Faza 0 — Pamatai

**Tikslas:** veikiantis „Hello world" Tauri + Svelte projektas su visais įrankiais.
**Rizika:** 🟢 maža. **Įvertis:** 1 diena.

### P0.1 — Sukurti Tauri v2 + SvelteKit projektą `[x]`

**Priklausomybės:** —
**Failai:** visas root

**Ką daryti:**
```bash
pnpm create tauri-app@latest nullbyte --template svelte-ts --manager pnpm
cd nullbyte
pnpm install
```
- Patikrink, kad `@sveltejs/adapter-static` naudojamas `svelte.config.js`
- `src/routes/+layout.ts`: `export const ssr = false; export const prerender = true;`
- `crates/nullbyte-app/tauri.conf.json`: `productName: "Nullbyte"`, `identifier: "fr.nullbyte.app"`,
  `version: "0.1.0"` (kelias atnaujintas P4.0.1 workspace split'o — žr. ADR-016)

**Acceptance:**
- [x] `pnpm tauri dev` atidaro langą su Svelte puslapiu
- [x] `pnpm tauri build` sukuria binarą be klaidų

---

### P0.2 — Tailwind v4 + shadcn-svelte `[x]`

**Priklausomybės:** P0.1
**Failai:** `src/app.css`, `components.json`, `src/lib/components/ui/`

**Ką daryti:**
```bash
pnpm add -D tailwindcss @tailwindcss/vite
pnpm dlx shadcn-svelte@latest init
pnpm dlx shadcn-svelte@latest add button card dialog input select tabs \
  scroll-area separator badge tooltip skeleton sonner command sheet slider switch
```
- `src/app.css`: `@import "tailwindcss";` + tamsi tema kaip numatytoji (`:root` = dark reikšmės)
- Tema: neutralus pilkas pagrindas, vienas akcento atspalvis (siūlomas: `oklch` violetinis/žydras)

**Acceptance:**
- [x] `<Button>` iš shadcn renderinasi
- [x] Tamsi tema aktyvi be `class="dark"` perjungimo
- [x] `pnpm check` be klaidų

---

### P0.3 — Rust priklausomybės ir modulių griaučiai `[x]`

**Priklausomybės:** P0.1
**Failai (originalūs, iki P4.0.1):** `src-tauri/Cargo.toml`, `src-tauri/src/**/mod.rs`
**Failai (nuo P4.0.1, ADR-016):** `Cargo.toml` (workspace root), `crates/nullbyte-core/Cargo.toml`,
`crates/nullbyte-core/src/**/mod.rs`, `crates/nullbyte-app/Cargo.toml`,
`crates/nullbyte-app/src/**/mod.rs`

**Ką daryti:**
- Į `Cargo.toml` sudėk visas priklausomybes iš `CLAUDE.md` §2
- Sukurk tuščius modulius pagal `CLAUDE.md` §4 struktūrą (kiekvienas `mod.rs` su `//!` doc)
- `error.rs`: `AppError` enum su `thiserror`, `impl serde::Serialize` (kad kirstų IPC) —
  nuo P4.0.1 gyvena `crates/nullbyte-core/src/error.rs` (bendras visiems trims crate'ams)
- `paths.rs`: funkcijos `data_dir()`, `cores_dir()`, `system_dir()`, `saves_dir()`,
  `states_dir()`, `media_dir()`, `db_path()` — su `directories` crate arba rankiniu būdu
  pagal `CLAUDE.md` (macOS: `~/Library/Application Support/Nullbyte`, Linux: XDG) —
  nuo P4.0.1 gyvena `crates/nullbyte-app/src/paths.rs`
- `state.rs`: `AppState` struct (kol kas tuščias) — nuo P4.0.1 gyvena
  `crates/nullbyte-app/src/state.rs`

**Acceptance:**
- [x] `cargo build` be klaidų ir be warning'ų
- [x] `cargo clippy -- -D warnings` praeina
- [x] `paths::data_dir()` grąžina teisingą kelią abiejose platformose

---

### P0.4 — Įrankiai, lint, CI `[x]`

**Priklausomybės:** P0.2, P0.3
**Failai:** `.gitignore`, `.prettierrc`, `eslint.config.js`, `rustfmt.toml`, `.github/workflows/ci.yml`, `.env.example`

**Ką daryti:**
- `.gitignore`: `target/`, `node_modules/`, `build/`, `.svelte-kit/`, `.env`,
  `crates/nullbyte-core/cores/`, `crates/nullbyte-core/roms/`, `crates/nullbyte-core/bios/`,
  `crates/nullbyte-app/gen/`, `*.srm`, `*.state` (keliai atnaujinti P4.0.1 — `cores/roms/bios`
  testų fixture'ai gyvena `nullbyte-core` (`CARGO_MANIFEST_DIR`-pagrįsti testų keliai
  `core/loader.rs`/`environment.rs`/`info.rs`/`runner.rs`), workspace `target/` vienas
  bendras visiems trims crate'ams prie repo šaknies)
- `.env.example` su `SCREENSCRAPER_DEV_ID=` ir `SCREENSCRAPER_DEV_PASSWORD=`
- `package.json` skriptai: `check`, `lint`, `format`
- GitHub Actions: matrix `[macos-latest, ubuntu-latest]` → clippy + test + svelte-check

**Acceptance:**
- [ ] `pnpm lint` ir `pnpm format` veikia
- [ ] CI žalias abiejose platformose
- [ ] `.env` nepatenka į git

---

### P0.5 — Logging ir dev tooling `[x]`

**Priklausomybės:** P0.3
**Failai:** `crates/nullbyte-app/src/lib.rs`

**Ką daryti:**
- `tracing-subscriber` inicializacija su `EnvFilter` (`RUST_LOG=nullbyte=debug`)
- Log failas į `data_dir()/logs/nullbyte.log` su rotacija
- Tauri komanda `get_app_info()` → versija, keliai, platforma (naudinga debug'inant)

**Acceptance:**
- [x] `RUST_LOG=debug pnpm tauri dev` rodo log'us konsolėje
- [x] Log failas kuriamas

---

## 3. Faza 1 — libretro sluoksnis 🔴

**Tikslas:** įkelti core'ą, užkrauti ROM'ą, sukti `retro_run()` loop'ą — **be vaizdo ir garso**.
Tik įrodymas, kad FFI veikia.
**Rizika:** 🔴 didžiausia visame projekte. **Įvertis:** 3–5 dienos.

> Čia laimima arba pralaimima. Neik toliau, kol P1.7 neveikia patikimai.

### P1.1 — `libretro.h` FFI tipai `[x]`

**Priklausomybės:** P0.3
**Failai:** `crates/nullbyte-core/src/core/ffi.rs`

**Ką daryti:**
- Perrašyk į Rust `#[repr(C)]` struct'us:
  `retro_system_info`, `retro_system_av_info`, `retro_game_geometry`,
  `retro_system_timing`, `retro_game_info`, `retro_variable`, `retro_message`,
  `retro_log_callback`
- Konstantos: `RETRO_API_VERSION`, `RETRO_ENVIRONMENT_*` (bent tos iš `CLAUDE.md` §8.3),
  `RETRO_PIXEL_FORMAT_*`, `RETRO_DEVICE_JOYPAD`, `RETRO_DEVICE_ID_JOYPAD_*`,
  `RETRO_MEMORY_SAVE_RAM`
- Callback tipų aliasai (`type RetroVideoRefreshT = unsafe extern "C" fn(...)`)
- Compile-time dydžių patikros: `const _: () = assert!(size_of::<retro_system_info>() == N);`

**Acceptance:**
- [x] Visi tipai atitinka `libretro.h` (patikrink prieš originalą eilutė po eilutės)
- [x] `cargo build` praeina
- [x] Nė vieno `static mut`

---

### P1.2 — Core įkėlimas per `libloading` `[x]`

**Priklausomybės:** P1.1
**Failai:** `crates/nullbyte-core/src/core/loader.rs`

**Ką daryti:**
- `struct CoreHandle { lib: Library, symbols: CoreSymbols, path: PathBuf }`
- `CoreSymbols` — visi 22 simboliai iš `CLAUDE.md` §8.1 kaip `RawSymbol` (ne `Symbol<'_>`,
  kad išvengtum lifetime pragaro — bet tada `lib` privalo gyventi ilgiau; dokumentuok SAFETY)
- `CoreHandle::load(path) -> Result<Self, AppError>`:
  - `Library::new(path)`
  - gauk visus simbolius; trūkstamas simbolis → aiški klaida su simbolio pavadinimu
  - `retro_api_version()` → jei != 1, atmesk
- `Drop for CoreHandle` — dokumentuok, kad `retro_deinit` privalo būti iškviestas PRIEŠ drop

**Acceptance:**
- [x] Testas: įkelia realų core'ą (pvz. `snes9x_libretro`) ir grąžina `api_version == 1`
- [x] Testas: neegzistuojantis failas → aiški `AppError`, ne panic
- [x] Testas: ne-libretro biblioteka (pvz. `libz`) → aiški klaida apie trūkstamą simbolį

---

### P1.3 — Core metaduomenys ir `.info` parsinimas `[x]`

**Priklausomybės:** P1.2
**Failai:** `crates/nullbyte-core/src/core/info.rs`

**Ką daryti:**
- `retro_get_system_info()` → `CoreInfo { name, version, valid_extensions, need_fullpath, block_extract }`
- Papildomai: jei šalia core'o yra `.info` failas (libretro standartas), parsink jį —
  ten yra `systemname`, `manufacturer`, `categories`, `database`
- `scan_cores_dir(path) -> Vec<CoreInfo>` — aptinka visus `*_libretro.dylib` / `*_libretro.so`
- Mapping'as plėtinys → core'ai (`.sfc` → `[snes9x, bsnes]`)

**Acceptance:**
- [x] Nuskaito katalogą su 5+ core'ais ir grąžina teisingus pavadinimus ir plėtinius
- [x] `.info` failo nebuvimas nesulaužo skenavimo
- [x] Testas su fixture katalogu

---

### P1.4 — Callback'ai ir `thread_local` kontekstas 🔴 `[x]`

**Priklausomybės:** P1.2
**Failai:** `crates/nullbyte-core/src/core/callbacks.rs`

**Ką daryti:**
- `thread_local! { static CTX: RefCell<Option<EmuContext>> }`
- `EmuContext` laiko: pixel format, geometry, video buferį, audio ring producer,
  input būvį, system/save dir `CString`'us, log callback
- Implementuok visus 6 callback'us pagal `CLAUDE.md` §3.3 šabloną
- `video_refresh_cb`: `data.is_null()` → dupe frame, praleisk
- `audio_sample_batch_cb`: `(*const i16, frames)` → stumk į ring buffer
- `input_state_cb`: grąžink iš `EmuContext.input_state`
- **Visuose callback'uose:** jokių `unwrap()`, jokio alokavimo kur įmanoma išvengti

**Acceptance:**
- [x] Visi callback'ai `unsafe extern "C"`, visi su `// SAFETY:` komentaru
- [x] `data == NULL` kadras neuždaro programos
- [x] `cargo clippy -- -D warnings` praeina

---

### P1.5 — `retro_environment` apdorojimas 🔴 `[x]`

**Priklausomybės:** P1.4
**Failai:** `crates/nullbyte-core/src/core/environment.rs`

**Ką daryti:**
- Implementuok visas komandas iš `CLAUDE.md` §8.3 lentelės
- `GET_LOG_INTERFACE` — **implementuok pirmiausia**, tai duos core'o log'us į `tracing`
  ir sutaupys daug debug'inimo laiko
- Nežinomos komandos → `tracing::debug!` su ID ir `return false`
- `GET_SYSTEM_DIRECTORY` / `GET_SAVE_DIRECTORY` — `CString` saugoma `EmuContext`, ne lokaliai

**Acceptance:**
- [x] Snes9x core inicializuojasi be klaidų log'e
- [x] core paprašo system directory ir gauna teisingą kelią (Beetle PSX neturime — patikrinta
      unit testu su konfigūruotu keliu; realiu core'u toks užklausimas retro_init() metu
      nefiksuotas nei snes9x, nei genesis_plus_gx, dauguma core'ų to prašo tik load_game metu)
- [x] Nežinoma komanda logginama, bet nesulaužo veikimo
- [x] Pixel format teisingai įsimenamas (patikrink log'e)

---

### P1.6 — ROM įkėlimas `[x]`

**Priklausomybės:** P1.5
**Failai:** `crates/nullbyte-core/src/core/loader.rs`, `crates/nullbyte-core/src/archive.rs`

**Ką daryti:**
- `load_game(rom_path)`:
  - jei `need_fullpath == true` → paduok kelią, `data = NULL`
  - jei `false` → įkelk failą į atmintį, paduok `data` + `size`
  - archyvams (`.zip`/`.7z`): išpakuok pirmą tinkamą plėtinį į atmintį
    (jei `need_fullpath` — išpakuok į temp failą)
- `retro_get_system_av_info()` **po** `load_game` → įsimink `fps`, `sample_rate`, `geometry`
- `unload_game()` + `deinit()` teisinga tvarka

**Acceptance:**
- [x] SNES `.sfc` įkeliamas, `av_info.timing.fps` teisingas (testas priima ir NTSC ≈60.098,
      ir PAL ≈50.0 — `read_dir` tvarka neapibrėžta, priklauso kurį ROM'ą iš `roms/snes/`
      pataikys; svarbu, kad reikšmė tikra aparatūrinė, ne apvalinta 60/50)
- [x] `.zip` su ROM'u viduje įkeliamas (NES core'o neturime — patikrinta su `.sfc` `.zip`
      viduje per snes9x; archive.rs logika nepriklauso nuo konsolės tipo)
- [x] PS1 core su `need_fullpath` gauna kelią, ne buferį (realiai patikrinta:
      `mednafen_psx_libretro.dylib` + tikras BIOS + realus PS1 žaidimas per `.zip`)
- [x] Blogas ROM → `AppError`, ne crash (neegzistuojantis kelias → `AppError::Io`;
      pastaba: snes9x pačio ROM header validaciją daro labai atlaidžiai — šiukšlių baitai
      su `.sfc` plėtiniu realiai BUVO priimti kaip validus LoROM, tad testas orientuotas į
      mūsų pačių I/O klaidos apdorojimą, ne core'o header patikrą)

---

### P1.7 — Emuliavimo gija ir headless loop 🔴 `[x]`

**Priklausomybės:** P1.6
**Failai:** `crates/nullbyte-core/src/core/runner.rs`

**Ką daryti:**
- `EmuThread` — dedikuota gija su komandų kanalu (`crossbeam-channel` arba `std::sync::mpsc`):
  ```rust
  enum EmuCommand { Load{core, rom}, Run, Pause, Resume, Reset, Stop,
                    SaveState(u8), LoadState(u8), SetInput(InputState) }
  ```
- Loop: `recv_timeout` komandoms → `retro_run()` → frame pacing
- Frame pacing MVP: `spin_sleep` iki `1.0 / av_info.timing.fps`
  (bus pakeista audio-driven sinchronizacija P3.4)
- FPS skaitiklis → `tracing::info!` kas 5 s
- Švarus sustabdymas: `unload_game` → `deinit` → drop

**Acceptance:** (visi patikrinti realiu 60s paleidimu, du kartus, release build,
`snes9x` + `Super Punch-Out!!.sfc` iš `roms/snes/`)
- [x] Paleidžia SNES ROM'ą ir 60 sekundžių sukasi be crash'o — 2/2 paleidimai švarūs
- [x] Log rodo ~FPS (±1) — šis konkretus ROM'as pasirodė PAL (50.0 Hz, ne NTSC 60.098);
      `measured_fps` svyravo 49.98–50.03 per abu paleidimus (±0.03, gerokai tiksliau nei ±1)
- [x] `Stop` komanda sustabdo švariai, be memory leak — RSS stebėta kas ~9s per visą 60s:
      31296→31312 KB (16 KB svyravimas iš ~31 MB), procesas išnyksta iš karto po `Stop`
- [x] Video callback kviečiamas ~60 k./s (skaitliukas log'e) — `video_fps` kiekvieną kartą
      tiksliai sutapo su `measured_fps` (šiam core'ui/ROM'ui 1 video kadras = 1 retro_run())
- [x] Audio callback duoda ~32040 sample/s SNES atveju — 31679–32055/s, centruota tiksliai
      ant core'o paties praneštos `sample_rate = 32040.0`

> **Milestone M1:** čia turi būti aišku, kad libretro integracija veikia.
> Jei ne — sustok ir spręsk, prieš eidamas į Fazę 2.

---

## 4. Faza 2 — Vaizdas 🔴

**Tikslas:** matomas žaidimo vaizdas lange.
**Rizika:** 🔴 didelė (wgpu + Tauri + platformų skirtumai). **Įvertis:** 3–4 dienos.

### P2.1 — Pikselių formatų konversija `[x]`

**Priklausomybės:** P1.4
**Failai:** `crates/nullbyte-core/src/video/pixel_format.rs`

**Ką daryti:**
- `convert_to_rgba8(src: &[u8], format: PixelFormat, width, height, pitch) -> Vec<u8>`
- Palaikyk `RGB565`, `XRGB8888`, `0RGB1555`
- **Gerbk `pitch`** (baitais, ne pikseliais — dažniausia klaida)
- Optimizuok: pre-alokuotas išvesties buferis, ne `Vec` per kadrą

**Acceptance:**
- [x] Unit testai visiems 3 formatams su žinomomis reikšmėmis (raudona/žalia/mėlyna/balta/juoda)
- [x] Testas su `pitch > width * bpp` (padding'as) duoda teisingą rezultatą
- [x] Benchmark: 256×224 RGB565 konversija < 0.5 ms — patikrinta `--release` (debug build'e
      naudojamas atlaidesnis 5 ms limitas, nes debug optimizacijos išjungtos)

---

### P2.2 — Triple buffer tarp gijų `[x]`

**Priklausomybės:** P2.1
**Failai:** `crates/nullbyte-core/src/video/frame_buffer.rs`

**Ką daryti:**
- Trys buferiai + atominis indeksas: emu gija rašo į „write", UI gija skaito „read"
- Emu gija niekada nelaukia; UI gija visada gauna naujausią pilną kadrą
- Kadras neša metaduomenis: `width`, `height`, `generation`

**Acceptance:**
- [x] Testas: 2 gijos, 10 000 kadrų, jokio data race (`cargo test --release`) — 340/10000
      kadrų pamatyta (tikėtasi — triple buffer dizainu praleidžia tarpinius), paskutinis
      (10000-asis) kadras patvirtintai pagautas, visų pamatytų kadrų baitai nuoseklūs
      (nė vieno „suplėšyto" skaitymo)
- [x] Emu gija niekada neblokuojasi — `write_frame` ilgiausiai truko 8.375µs per 10 000
      kvietimų (limitas teste 5ms)

---

### P2.3 — Emuliatoriaus langas ir wgpu surface 🔴 `[x]`

**Priklausomybės:** P0.3, P2.2
**Failai:** `crates/nullbyte-core/src/video/renderer.rs`, `crates/nullbyte-emu/src/main.rs`

**Ką daryti:**
- Sukurk **atskirą Tauri `Window` be webview** emuliatoriui
  (`tauri::window::WindowBuilder`, ne `WebviewWindowBuilder`)
- Iš jo gauk `raw_window_handle()` → `wgpu::Instance::create_surface()`
- **Surface kūrimas ir `present()` — TIK main/UI gijoje** (macOS reikalavimas)
- Resize handling: rekonfigūruok surface lango dydžio įvykyje

> **Jei šis kelias nepavyksta** (žr. §13 riziką R2) — fallback: siųsk kadrus per
> `tauri::ipc::Channel<&[u8]>` į `<canvas>` su WebGL2. Veikia iki ~640×480@60.
> Šis fallback'as **nėra numatytasis** — bandyk native langą pirma.

**Acceptance:**
- [x] Atsidaro antras langas, wgpu inicializuojasi be klaidų — patikrinta realiu `pnpm tauri dev`
      paleidimu (log: „wgpu adapteris pasirinktas" adapter=Apple M1 backend=Metal, „wgpu Surface
      sukonfigūruotas" 1600×1200 Bgra8UnormSrgb)
- [x] Veikia macOS (Metal) — patikrinta (žr. aukščiau)
- [!] Veikia Linux X11 (Vulkan) — **NEPATIKRINTA**, šioje sesijoje nėra Linux mašinos su
      ekranu. Kodas naudoja tik standartines, platformai neutralias Tauri (`raw-window-handle`)
      ir wgpu API — jokių macOS-specifinių hack'ų. CI (P0.4) tikrina `cargo build`/`clippy` ant
      `ubuntu-latest`, bet CI runner'yje nėra display serverio/GPU, tad realaus wgpu Surface
      veikimo nepatikrina. Reikia patikrinti realioje Linux mašinoje prieš MVP išleidimą.
- [!] Veikia Linux Wayland arba yra dokumentuotas apėjimas — **NEPATIKRINTA** (ta pati priežastis)
- [x] Lango dydžio keitimas nesulaužo surface'o — patikrinta AppleScript resize (900×700 →
      1800×1400 su Retina scale), log: „wgpu Surface rekonfigūruotas (resize)", jokio crash'o

> **Atnaujinta P4.3/ADR-016 (2026-08-20):** čia sukurtas Tauri `Window` be webview
> pasirodė NETURINTIS jokio klaviatūros event'ų API — kliūtis, atrasta tik P4.2 metu.
> ADR-016 perkelia šį langą (ir visą `renderer.rs`/`video::frame_buffer` logiką, iš esmės
> nepakitusią) į atskirą `nullbyte-emu` vaiko procesą su winit langu vietoj Tauri `Window`.
> Šio task'o rezultatai (acceptance įrodymai aukščiau) LIEKA teisingi kaip wgpu pipeline'o
> patikra — keičiasi tik lango KŪRĖJAS (winit, ne Tauri), ne pati wgpu logika.

---

### P2.4 — Blit pipeline ir shader'is 🔴 `[x]`

**Priklausomybės:** P2.3
**Failai:** `crates/nullbyte-core/src/video/renderer.rs`, `crates/nullbyte-core/src/video/shaders/blit.wgsl`

**Ką daryti:**
- `wgpu::Texture` (RGBA8) → `queue.write_texture()` iš triple buffer
- Full-screen triangle vertex shader + sampled texture fragment shader
- Sampler: `Nearest` (numatytasis, pixel-perfect) ir `Linear` (nustatymuose)
- Render loop susietas su lango redraw įvykiu

**Acceptance:**
- [x] **Matomas SNES žaidimo vaizdas** — pirmas tikras vizualus rezultatas. Patikrinta realiu
      `pnpm tauri dev` paleidimu su snes9x + „Super Mario World.sfc" (laikinas hook
      `lib.rs::setup()`, pašalintas po verifikacijos): pilnas srautas emu gija →
      `pixel_format::convert_to_rgba8_into` → triple buffer → `frame pump` gija →
      `Renderer::upload_frame` + `render()` veikė stabiliai 15+ s (trys 5s statistikos log'ai,
      `measured_fps`/`video_fps` ~49.87–50.01, jokio crash'o). Ekrano nuotrauka
      (`screencapture`) parodė pilną, aiškų SNES titulinį ekraną
- [x] Spalvos teisingos — nuotraukoje matomas teisingas SMW logotipas (raudona/geltona/žalia/
      mėlyna), dangaus/debesų gradientas, Mario+Yoshi sprite'ai teisingomis spalvomis, jokios
      kanalų sumaišties (nėra BGR/RGB swizzle klaidos) ir jokios korupcijos/juodo ekrano
- [x] Nėra tearing'o — `PresentMode::AutoVsync` sukonfigūruotas `Renderer::new()` (žr.
      `video/renderer.rs`); vizualiai stabilus, be blyksėjimo vaizdas nuotraukoje ir ekrane
      stebint tiesiogiai

> **Milestone M2 pasiektas:** žaidimas matomas ekrane (2026-08-20).

---

### P2.5 — Aspect ratio, scaling, fullscreen 🔴 `[x]`

**Priklausomybės:** P2.4
**Failai:** `crates/nullbyte-core/src/video/renderer.rs`

**Ką daryti:**
- Gerbk `av_info.geometry.aspect_ratio` (jei 0 → `base_width / base_height`)
- Letterbox / pillarbox su juodais kraštais
- Integer scaling režimas (nustatymuose)
- Fullscreen perjungimas: `F11` ir `Cmd+Ctrl+F` (macOS)
- `Esc` išeina iš fullscreen

**Acceptance:**
- [x] SNES 4:3 vaizdas neištemptas 16:9 lange — patikrinta realiu `pnpm tauri dev` paleidimu,
      langas rankiniu būdu pakeistas į 1600×900 (16:9), core'as/ROM'as (snes9x + Super Mario
      World) sugeneruoja `aspect_ratio` per `LoadedGameInfo` → `VideoFrameData` → `Renderer`.
      Ekrano nuotrauka rodo simetriškas pillarbox juodas juostas abiejose pusėse, vaizdas
      NEIŠTEMPTAS. **Rasta ir ištaisyta reali klaida verifikacijos metu:** pirma
      implementacija (NDC pozicijos skaliavimas ant P2.4 „pilno ekrano trikampio") davė
      ASIMETRIŠKĄ juostą (tik vienoje pusėje) — trikampio „perteklinis" kraštas kirpamas prie
      FIKSUOTOS clip ribos, ne prie sutrauktos, todėl scale'inimas nesimetriškas. Sprendimas —
      pakeista į tikrą 4-kampų quad'ą (6 viršūnės, 2 trikampiai), kurio kampai visada tiksliai
      `(±scale.x, ±scale.y)`. Antra ekrano nuotrauka po pataisymo patvirtina simetriją
- [x] Integer scaling duoda ryškius pikselius be interpoliacijos artefaktų — patikrinta
      realiu paleidimu, `ScaleMode::Integer` perjungtas per laikiną verifikacijos hook'ą
      (pašalintas po patikros). Ekrano nuotrauka rodo aiškiai mažesnį, sveikojo daugiklio
      dydžio, centruotą vaizdą su juoda juosta visose 4 pusėse (skirtingai nuo `Fit`, kuris
      užpildo pilną aukštį) — pikseliai aštrūs, jokio blur'o (Nearest sampler + sveikasis
      daugiklis pagal RAW core'o pikselių dydį, sąmoningai NEPRITAIKANT aspect_ratio, kad
      kiekvienas šaltinio pikselis atitiktų tikslų NxN bloką ekrane)
- [x] Fullscreen veikia macOS — patikrinta realiu paleidimu per naują
      `commands::emulator::toggle_emulator_fullscreen` (Tauri `Window::set_fullscreen`).
      Ekrano nuotrauka rodo langą, užimantį visą ekraną, su tebeveikiančiu Integer scaling
      (aspect/scale skaičiavimas teisingai persiskaičiuoja po `resize` įvykio, kurį
      fullscreen sukelia). [!] Linux — NEPATIKRINTA (nėra Linux mašinos šioje sesijoje, ta
      pati priežastis kaip P2.3). **Klavišų susiejimas (`F11`, `Cmd+Ctrl+F`, `Esc`) SĄMONINGAI
      NEĮGYVENDINTAS šioje užduotyje** — Tauri `Window` be webview nesiunčia klaviatūros
      `WindowEvent`'ų šioje API versijoje (patikrinta `tauri::app::WindowEvent` šaltinyje),
      o CLAUDE.md pati numato klaviatūros įvesties sluoksnį kaip atskirą P4.2
      (`input/keyboard.rs`) — nešokama į vėlesnę fazę. `toggle_emulator_fullscreen` komanda
      užregistruota ir paruošta, kad P4.2/P7.x UI galėtų ją kviesti tiesiogiai per `invoke`.

---

## 5. Faza 3 — Garsas 🔴

**Tikslas:** garsas be traškesių, sinchronizuotas su vaizdu.
**Rizika:** 🔴 didelė (real-time constraints). **Įvertis:** 2–3 dienos.

### P3.1 — cpal išvesties srautas `[x]`

**Priklausomybės:** P0.3
**Failai:** `crates/nullbyte-core/src/audio/output.rs`

**Ką daryti:**
- Numatytasis įrenginys, `f32` arba `i16` formatas, stereo
- Buferio dydis: taikyk ~40–60 ms latency
- Klaidos callback → `tracing::error!` (ne panic)
- Įrenginio dingimas (ausinių atjungimas) → atkūrimas, ne crash

**Acceptance:**
- [x] Sinusoidė 440 Hz groja švariai 30 s — patikrinta 2x realiu paleidimu
      (`cargo test --release plays_440hz_sine_for_30_seconds -- --ignored --nocapture`),
      vartotojas patvirtino girdėjęs švarų toną abu kartus (MacBook Air Speakers,
      48000 Hz, stereo, F32, 50ms tikslinis latency)
- [x] Ausinių atjungimas/prijungimas neuždaro programos — patikrinta realiai: vartotojas
      fiziškai atjungė/prijungė garso įrenginį per 30s testo langą, programa toliau veikė
      be crash'o (`is_device_lost()` liko `false` visą laiką — CoreAudio šiuo atveju
      persijungimą sutvarkė skaidriai, klaidos callback'as nesuveikė, bet svarbiausia: jokio
      panic'o/crash'o). Klaidos callback'as (kai jis realiai suveikia) veikia NE real-time
      audio gijoje — `tracing::error!` ten saugus (CLAUDE.md §3.2 taisyklė #3 galioja tik
      duomenų callback'ui)
- [x] Veikia macOS (CoreAudio) — patikrinta aukščiau. [!] Linux (ALSA/PipeWire) —
      NEPATIKRINTA (nėra Linux mašinos šioje sesijoje, ta pati priežastis kaip P2.3/P2.5)

---

### P3.2 — Lock-free ring buffer `[x]`

**Priklausomybės:** P3.1
**Failai:** `crates/nullbyte-core/src/audio/ring.rs`

**Ką daryti:**
- `rtrb::RingBuffer<i16>` — producer emu gijoje, consumer cpal callback'e
- Talpa ≈ 4× buferio dydis
- Underrun → užpildyk tyla + `tracing::warn!` (throttled, ne per kadrą)
- Overrun → mesk seniausius sample'us
- `occupancy()` metodas rate control'ui

**Acceptance:**
- [x] Jokio alokavimo cpal callback'e — patikrinta kodo peržiūra: `AudioConsumer::fill()`
      (kviečiamas iš `audio/output.rs` `sample_source` callback'o) naudoja tik
      `rtrb::Consumer::read_chunk` (grąžina jau egzistuojančias `&[i16]` slices per
      `IntoIterator`) ir paprastą `for` ciklą — jokio `Vec`/`String`/`format!`. Throttled
      logging PERKELTAS Į KITĄ (ne real-time) gijos kontekstą pagal projektavimą: callback'e
      tik atomiškai didinamas skaitliukas (`AtomicU64`), `tracing::warn!` bus kviečiamas iš
      `core::runner` periodinio 5s statistikos log'o (P1.7 šabloną pratęsiant), kai
      `AudioConsumer`/`AudioProducer` bus sujungti į `runner.rs` (P3.4)
- [x] Underrun/overrun nesulaužo srauto — patikrinta 60s soak testu (žr. žemiau):
      `underrun=34616 overrun=220426` per 60s, nė vienas neužstrigo/nepanikavo
- [x] Testas: producer/consumer skirtingais greičiais 60 s —
      `producer_and_consumer_at_different_speeds_for_60_seconds` (`#[ignore]`, paleista
      rankiniu būdu `--release`): producer'io greitis kas 1s persijungia greitas/lėtas
      (sąmoningai sukelia ir overrun, ir underrun), consumer'is fiksuoto greičio — testas
      PRAĖJO (assert'ai patvirtino, kad abu scenarijai realiai suveikė bent kartą). Greita
      (2s) sanity versija `producer_and_consumer_at_different_speeds_short` įeina į įprastą
      `cargo test`.

> **Pastaba dėl overrun semantikos:** `rtrb::Producer` API neturi būdo pašalinti jau įrašytus
> (dar neperskaitytus) sample'us — tik `Consumer` (kita gija) gali juos paimti. „Mesk
> seniausius" realizuota PER ĮEINANTĮ chunk'ą (paliekamas tik naujausias segmentas, kuris
> tilpsta laisvoje vietoje), ne per jau eilėje esančius duomenis — žr. detalų paaiškinimą
> `audio/ring.rs` modulio doc komentare.

---

### P3.3 — Resampling `[x]`

**Priklausomybės:** P3.2
**Failai:** `crates/nullbyte-core/src/audio/resampler.rs`

**Ką daryti:**
- `rubato::SincFixedIn` arba `FastFixedIn`: `av_info.timing.sample_rate` → įrenginio rate
- Testuok: 32040 → 48000 (SNES), 44100 → 48000 (Genesis), 32768 → 48000 (GBA)
- Resampling vyksta **emu gijoje**, ne audio callback'e

**Acceptance:**
- [x] SNES garsas skamba teisingu tonu (ne per aukštai/žemai) — patikrinta DVIGUBAI: (1)
      automatiniai testai `snes_rate_preserves_440hz_pitch`/`genesis_rate_preserves_440hz_pitch`/
      `gba_rate_preserves_440hz_pitch` (Goertzel algoritmu, be FFT priklausomybės, patikrina,
      kad 440Hz energija po resampling'o aiškiai dominuoja prieš klaidingo santykio dažnį —
      pagautų apverstą ratio klaidą); (2) REALUS klausomas testas
      (`plays_resampled_440hz_snes_tone`, 32040→realaus įrenginio rate, 7s per
      `audio/output.rs`) — vartotojas patvirtino girdėjęs švarų, teisingą 440Hz toną
- [x] Nėra aliasing artefaktų — testas `near_nyquist_tone_does_not_alias`: 0.45×32040Hz
      (arti įvesties Nyquist) tonas po resampling'o į 48000Hz — signalo energija testo
      dažnyje >10x viršija energiją klasikiniame alias veidrodyje (`input_rate - test_freq`)
- [x] Resampling < 1 ms per kadrą — benchmark `resampling_under_1ms_per_frame` (~534 kadrų
      batch'as, tipinis 60fps/32040Hz core'o dydis, release build'e) PRAĖJO su < 1ms limitu

> **Pastaba dėl apimties:** šis modulis (`AudioResampler`) yra savarankiškas, pilnai
> testuotas konverteris (interleaved i16 @ rate X → interleaved i16 @ rate Y). Sujungimas su
> realiu žaidimo garso srautu (`core::runner`'io `audio_sample_batch_cb` duomenys →
> resampler → `audio::ring::AudioProducer`) ir dinaminis rate control — P3.4 darbas (tas
> pats šablonas kaip P1.1–P1.6 vs P1.7, arba P2.1–P2.3 vs P2.4: pirma paruošiami sluoksniai
> atskirai, tada sujungiami vienoje užduotyje).

---

### P3.4 — Dynamic rate control ir audio-driven sync 🔴 `[x]`

**Priklausomybės:** P3.3, P1.7
**Failai:** `crates/nullbyte-core/src/audio/resampler.rs`, `crates/nullbyte-core/src/core/runner.rs`

**Ką daryti:**
- Formulė iš `CLAUDE.md` §8.6: koreguok resampling ratio pagal buffer occupancy
- `MAX_DELTA = 0.005` (0.5 %) — nepastebima ausiai
- **Pakeisk P1.7 frame pacing'ą**: emu gija nebemiega fiksuotą laiką, o laukia,
  kol ring buffer'yje atsiras vietos → garsas tampa laikrodžiu
- Fast-forward režimas: išjunk rate control, mesk audio sample'us

**Acceptance:**
- [x] **10 minučių SNES žaidimo be vieno traškesio** — patikrinta realiu paleidimu
      (snes9x + Super Mario World, per pilną `audio/output.rs` → `audio/ring.rs` →
      `audio/resampler.rs` → `core/runner.rs` pipeline'ą, sujungtą per naują
      `commands::emulator::start_audio_pump`). 131 statistikos įrašas per ~11 min veikimo,
      **0 overrun per VISĄ laiką**, vartotojas patvirtino girdėjęs švarų garsą be traškesių
      visą 10 min trukmę
- [x] Buffer occupancy svyruoja apie 50 %, nedreifuoja į 0 % ar 100 % — patvirtinta tuo pačiu
      10 min žurnalu: occupancy stabiliai svyravo ~44–74% diapazone (vidurkis ~55–60%),
      niekada neprisiartino prie 0% ar 100%. **Rasta ir ištaisyta reali klaida
      verifikacijos metu:** pirmas bandymas su `BUFFER_HIGH_WATERMARK = 0.9` (toli virš
      tikslinio 50%) leido emuliavimo gijai bėgti NEATSKĖTA (jokio delsimo tarp kadrų) tol,
      kol occupancy pasiekdavo 90%, po to STAIGA „prasiverždavo" per ją ir overrun'indavo
      (stebėta: occupancy=0.91, overrun augo pastoviai ~6.4/s). Priežastis — vieno
      `retro_run()` kadro audio porcija (~8% viso buferio) per didelė santykinai su tolima
      riba. Sumažinus ribą iki `0.6` (arti tikslinio 50%), throttle suveikia kiekvieną kadrą,
      kai occupancy artėja prie tikslo — tai IR YRA audio-driven pacing, ne šalutinis
      efektas. Po pataisymo: measured_fps stabiliai ~50 (tikras SNES fps=50.007), overrun=0
- [x] Vaizdas ir garsas nesiskiria (lūpų sinchronizacija) — patvirtinta vartotojo per 10 min
      testą. **Pastaba:** ankstesniame bandyme (su laikinu fast-forward perjungimo testu
      viduryje sesijos) vartotojas pastebėjo trumpalaikį nesutapimą iškart po perjungimo
      (tikėtina — fast-forward metu garsas sąmoningai metamas/neatsilieka, o vaizdas bėga
      pilnu greičiu; grįžus į normalų režimą sistema pati susisinchronizavo per kelias
      sekundes). Švariame 10 min paleidime BE fast-forward perjungimų — jokio pastebimo
      nesutapimo visą laiką
- [x] Fast-forward veikia be crash'o — patikrinta realiu paleidimu
      (`EmuCommand::SetFastForward(true/false)`): measured_fps šoktelėjo į ~417 (CPU pilnu
      greičiu), audio occupancy nukrito į 0.0 (sample'ai IŠMESTI, kaip numatyta), jokio
      crash'o/panikos. Išjungus fast-forward — švarus, akimirksniu atsistatymas į normalų
      audio-driven pacing'ą (occupancy grįžo į ~50-70% per kelias sekundes)

> **Milestone M3 pasiektas:** žaidimas veikia su vaizdu ir garsu (2026-08-20).

---

## 6. Faza 4 — Įvestis

**Tikslas:** valdyti žaidimą gamepad'u ir klaviatūra.
**Rizika:** 🔴 didelė (P4.0.x — proceso architektūros migracija). **Įvertis:** 6–9 dienos
(2 d. įvestis + 4–7 d. P4.0.x migracija, žr. ADR-016).

> **P4.0.x — proceso architektūros migracija (ADR-016, 2026-08-20).** Prieš tęsiant P4.2
> (mapping) reikia realiai įgyvendinti tai, kas šiuo metu užfiksuota TIK dokumentacijoje
> (CLAUDE.md §3.4/§4/§10, MVP.md ADR-016): Cargo workspace split į tris crate'us ir
> `nullbyte-emu` vaiko procesą. P4.1 (gamepad aptikimas) kodas jau parašytas SENOJE
> struktūroje (`src-tauri/src/input/gamepad.rs`) — P4.0.1 metu jis tiesiog PERKELIAMAS į
> `crates/nullbyte-core/src/input/gamepad.rs` (be logikos pakeitimų), ne perrašomas iš naujo.

### P4.0.1 — Cargo workspace split `[x]`

**Priklausomybės:** —
**Failai:** `Cargo.toml` (naujas, workspace root), `crates/nullbyte-core/Cargo.toml` (naujas),
`crates/nullbyte-app/Cargo.toml` (perkeltas iš `src-tauri/Cargo.toml`), visas esamas
`src-tauri/src/` turinys perskirstytas į `crates/nullbyte-core/src/` (core/video/audio/input/
error.rs) ir `crates/nullbyte-app/src/` (likusi dalis), `.gitignore`

**Ką daryti:**
- Sukurk root `Cargo.toml` su `[workspace]` `members = ["crates/*"]`, `resolver = "2"`
- Sukurk `crates/nullbyte-core/` — perkelk `core/`, `video/`, `audio/`, `input/`, `error.rs`.
  Priklausomybės iš senojo `Cargo.toml`, kurių šiems moduliams reikia (libloading, wgpu, cpal,
  rubato, rtrb, gilrs, thiserror, tracing, md-5/sha1/crc32fast — NE tauri/rusqlite/reqwest)
- Perkelk `archive.rs` (P1.6 naudojamas iš `core::loader`) TIESIOGIAI į
  `crates/nullbyte-core/src/archive.rs` (NE po `library/` — `library/` kaip DB/skenavimo
  konceptas lieka tik `nullbyte-app`, bet archyvo IŠPAKAVIMAS reikalingas core'o ROM
  įkėlimui, taigi turi būti bendras)
- Sukurk `crates/nullbyte-app/` — perkelk likusį `src-tauri/` turinį (`db/`, `library/`
  [scanner.rs, hasher.rs — BE archive.rs], `scraper/`, `commands/`, `state.rs`, `paths.rs`,
  `lib.rs`, `main.rs`, `migrations/`, `capabilities/`, `icons/`, `tauri.conf.json`,
  `build.rs`). Prideda priklausomybę `nullbyte-core = { path = "../nullbyte-core" }`
- Perkelk `cores/`, `roms/`, `bios/` testų fixture katalogus į
  `crates/nullbyte-core/{cores,roms,bios}/` (jų reikia `CARGO_MANIFEST_DIR`-pagrįstiems
  testams `core/loader.rs`/`environment.rs`/`info.rs`/`runner.rs`)
- Atnaujink visus `use crate::...` kelius abiejuose crate'uose (dabar tarp-crate importai —
  `use nullbyte_core::...` iš `nullbyte-app` pusės)
- Atnaujink `.gitignore` (žr. P0.4 pastabą aukščiau)
- Ištrink tuščią `src-tauri/` katalogą, kai migracija baigta ir viskas veikia

**Neplanuoti, bet būtini radiniai vykdymo metu (plano tekstas jų nenumatė):**
- Senasis `error.rs` (`AppError`) turėjo `Database(rusqlite::Error)`/`Network(reqwest::Error)`
  variantus — jų NEGALIMA palikti `nullbyte-core`'e (kertų „NE tauri/rusqlite/reqwest"
  taisyklę aukščiau). Išspręsta: `nullbyte-core::error::CoreError` (tik `Io`/`Other`, naudoja
  core/video/audio/input/archive.rs) + `nullbyte-app::error::AppError` (originalūs keturi
  variantai + naujas `Core(#[from] CoreError)`, kad `?` veiktų skambinant iš `nullbyte-app` į
  `nullbyte-core`)
- `video::renderer::Renderer::new()` priiminėjo `tauri::window::Window<R>` TIESIOGIAI — realus
  hard-dependency į `tauri` crate'ą `nullbyte-core`'e, ne vien plano tekste numatyta „NE tauri"
  eilutė. Adaptuota DABAR (ne atidėta P4.0.2, nes P4.0.1's savo acceptance reikalauja
  `cargo build --workspace` be `tauri` `nullbyte-core`'e): signatūra tapo
  `new<W: HasWindowHandle + HasDisplayHandle + Send + Sync + 'static>(window: W, size: (u32, u32))`,
  `tauri::async_runtime::block_on` → `pollster::block_on` (nauja, maža, gerai žinoma
  priklausomybė — tas pats sprendimas, kurį naudoja oficialūs wgpu pavyzdžiai). Veikia
  identiškai su Tauri `Window` (dabar) IR winit `Window` (P4.0.2) be jokių tolimesnių pakeitimų
- `audio/output.rs`/`core/environment.rs`/`core/runner.rs` testai naudojo `tracing_subscriber`
  tik testų viduje — pridėta kaip `nullbyte-core`'o `[dev-dependencies]`, ne pagrindinė
  priklausomybė
- `tauri.conf.json`'o `frontendDist` ("../build") tapo "../../build" (dabar dvi katalogų gilumos
  nuo repo šaknies, ne viena)
- **`pnpm tauri dev` patikrinta REALIAI** iš repo šaknies BE jokio papildomo flag'o — Tauri CLI
  automatiškai suranda `crates/nullbyte-app/tauri.conf.json` (jos paieška ieško bet kurio
  katalogo su `tauri.conf.json` + tauri priklausomybe, ne tik pavadinimu `src-tauri`). CLAUDE.md
  §5 „NEPATIKRINTA" pastaba pašalinta

**Acceptance:**
- [x] `cargo build --workspace` be klaidų
- [x] `cargo test --workspace` — visi esami testai praeina, be regresijos (63 nullbyte-core +
      2 nullbyte-app testai, 4 ignoruoti — kaip prieš migraciją)
- [x] `cargo clippy --workspace --all-targets -- -D warnings` švarus
- [x] `pnpm tauri dev` vis dar paleidžia Tauri langą (su atnaujintu keliu/config) — patikrinta
      realiai, langas atsidaro, `nullbyte_lib` startup log'as pasirodo

---

### P4.0.2 — `nullbyte-emu` binaro griaučiai (winit) `[ ]`

**Priklausomybės:** P4.0.1
**Failai:** `crates/nullbyte-emu/Cargo.toml`, `crates/nullbyte-emu/src/main.rs`

**Ką daryti:**
- `winit` event loop; macOS: `EventLoopBuilderExtMacOS::with_activation_policy(ActivationPolicy::Accessory)`
  (CLAUDE.md §10 — kitaip vaikas atsirastų Dock'e kaip antra programa)
- Sukurk `winit::window::Window`, perduok į `video::renderer::Renderer::new()` (adaptuok —
  ji ims `raw-window-handle` iš winit lango, ne Tauri)
- Paleisk `core::runner::EmuThread` (iš `nullbyte-core`) — laikinai hardkodintu core/ROM keliu
  testams, kaip visų ankstesnių fazių verifikacijos hook'ai
- `audio::output::AudioOutput` + audio pump logika (buvusi `start_audio_pump`) — čia
- Resize/klaviatūros/gamepad įvykiai iš winit event loop'o (klaviatūra — NAUJIENA, anksčiau
  negalima)

**Acceptance:**
- [ ] Langas atsidaro, rodo SNES žaidimą (regresijos patikra prieš P2.4 rezultatą)
- [ ] Garsas groja be traškesių (regresijos patikra prieš P3.4 rezultatą)
- [ ] **Klaviatūra REALIAI valdo žaidimą** — patikrink paprastu test mapping'u (pvz. strėlė →
      log'as), kad winit `KeyboardInput` tikrai ateina (tai buvo neįmanoma prieš ADR-016)
- [ ] macOS Dock nerodo antros programos (`ActivationPolicy::Accessory` patikrinta)

---

### P4.0.3 — IPC protokolas (`nullbyte-app` ↔ `nullbyte-emu`) `[ ]`

**Priklausomybės:** P4.0.1
**Failai:** `crates/nullbyte-core/src/ipc.rs` (bendras protokolo tipas), `crates/nullbyte-emu/src/ipc.rs`
(serveris), `crates/nullbyte-app/src/ipc.rs` (klientas)

**Ką daryti:**
- `EmuCommand` (jau egzistuoja `core::runner`) gauna `serde::Serialize`/`Deserialize` —
  naujas `EmuStatus` enum būvio pranešimams atgal (Loaded/Error/Stats/Stopped)
- Transportas: **`tauri-plugin-shell`** sidecar API (`ShellExt::shell().sidecar("nullbyte-emu")?.spawn()`
  → grąžina `Receiver<CommandEvent>` + `CommandChild`), ne žalias `std::process::Command`.
  **Nauja priklausomybė** — įrašyta CLAUDE.md §2 lentelėje. Pasirinkta vietoj žalio `Command`,
  nes automatiškai sprendžia binaro kelią dev vs bundle (žr. P4.0.5), o `CommandChild::write()`/
  `kill()`/`pid()` ir `CommandEvent::{Stdout,Stderr,Error,Terminated}` pilnai pakanka IPC + proceso
  gyvavimo ciklo poreikiams (API patikrinta prieš docs.rs `tauri_plugin_shell::process` puslapį)
- Žinučių formatas: **newline-delimited JSON (NDJSON)**, NE length-prefixed. `CommandEvent::Stdout`
  pagal nutylėjimą skaido baitus per `\n`/`\r`, tad kiekviena `EmuCommand`/`EmuStatus` žinutė —
  vienas kompaktiškas `serde_json` objektas per eilutę (be įterptų `\n`). Length-prefix framing
  būtų reikalingas tik su žaliu `Command`, čia perteklinis
- `nullbyte-emu`: fono gija skaito savo `stdin` per `BufRead::lines()`, parsina kiekvieną eilutę
  kaip `EmuCommand`, siunčia į `EmuThread`; rašo `EmuStatus` į `stdout` (viena eilutė, `flush()`
  iš karto)
- `nullbyte-app`: paleidžia sidecar'ą, siunčia komandas per `CommandChild::write()`, skaito
  `CommandEvent::Stdout` žinutes iš `Receiver`
- **KRITIŠKAI SVARBU:** kadangi IPC eina per `stdout`, `nullbyte-emu` NIEKADA negali rašyti į
  `stdout` nieko kito — nei `println!`/`dbg!`, nei numatytojo `tracing_subscriber` writer'io
  (jis pagal nutylėjimą rašo į stdout!). Vienas pamirštas `println!` arba nenukreiptas
  `tracing` subscriber sugadins protokolą, ir klaida atrodys kaip atsitiktinis JSON parse error,
  ne kaip logging problema. `nullbyte-emu` logina **tik į `stderr`** (`.with_writer(std::io::stderr)`)
  arba į failą — niekada į stdout. Žr. CLAUDE.md §10

**Acceptance:**
- [ ] `Load`/`Pause`/`Resume`/`Stop` komandos pasiekia vaiką ir sukelia teisingą elgesį
- [ ] Būvio pranešimai (klaidos, statistika) pasiekia tėvą
- [ ] Serializacijos klaida NESULAUŽO nei vieno proceso (`Result`, ne `panic!`/`unwrap()`)
- [ ] `nullbyte-emu` paleidus be jokio ROM'o (vien init) — `stdout` NEturi nė vienos baitos,
      kuri nėra validus NDJSON `EmuStatus` (patikrinta rankiniu būdu paleidus binarą tiesiogiai
      terminale ir stebint `stdout` atskirai nuo `stderr`)

---

### P4.0.4 — Proceso gyvavimo ciklas, našlaičių apsauga `[ ]`

**Priklausomybės:** P4.0.2, P4.0.3
**Failai:** `crates/nullbyte-emu/src/ipc.rs` (EOF → shutdown šaka toje pačioje stdin-skaitymo
gijoje, NE atskiras modulis — žr. pastabą žemiau), `crates/nullbyte-app/src/commands/emulator.rs`
(timeout + `CommandChild::kill()`)

**Ką daryti:**
- **Jokio atskiro pipe'o.** IPC `stdin` kanalas (P4.0.3) jau yra gyvumo signalas: kai tėvas
  miršta (bet kokiu būdu, įskaitant `kill -9`), OS uždaro paskutinę `stdin` write-end nuorodą,
  ir vaiko `BufRead::lines()` grąžina `None` (`EOF`) pati savaime — Unix pipe semantika, ne
  papildomas mechanizmas
- Vaikas: kai P4.0.3 stdin-skaitymo gija gauna `EOF` → švarus išsijungimas (`Stop` →
  `EmuThread`, tada `process::exit`). **NE PID pollinimas** (nepatikimas, race'inamas —
  žr. CLAUDE.md §10)
- Tėvas: normaliu atveju (vartotojas uždaro žaidimą) siunčia `Stop` per IPC, laukia proceso
  pabaigos (`CommandEvent::Terminated` iš `Receiver`) su timeout'u, po to `CommandChild::kill()`
  jei reikia

**Acceptance:**
- [ ] Dirbtinai nutraukus tėvo procesą (`kill -9`), vaikas savaime išsijungia per kelias
      sekundes vien dėl `stdin` EOF (patikrinta realiai, ne vien skaitant kodą)
- [ ] Normalus žaidimo uždarymas švariai sustabdo vaiką be „zombie" proceso
- [ ] Vaiko crash'as (dirbtinis panic core'e) nenumuša tėvo proceso

---

### P4.0.5 — `externalBin` packaging `[ ]`

**Priklausomybės:** P4.0.2
**Failai:** `crates/nullbyte-app/tauri.conf.json`

**Ką daryti:**
- `bundle.externalBin` nurodo `nullbyte-emu` binarą su target-triple sufiksu (pvz.
  `binaries/nullbyte-emu-aarch64-apple-darwin`)
- Dokumentuok/automatizuok binaro pervadinimą su target triple prieš bundle'inant
  (`cargo build --bin nullbyte-emu` → nukopijuoti/pervadinti pagal Tauri konvenciją)

**Acceptance:**
- [ ] `pnpm tauri build` sėkmingai supakuoja abu binarus
- [ ] Supakuotas `.app`/`.dmg` paleidžia `nullbyte-emu` teisingai (iš bundle'o kelio, ne dev)

---

### P4.1 — Gamepad aptikimas `[ ]` (kodas paruoštas SENOJE struktūroje, laukia P4.0.1 perkėlimo + fizinio valdiklio testo)

**Priklausomybės:** P0.3 (originaliai), P4.0.1 (kodo perkėlimui į naują crate — žr. pastabą aukščiau)
**Failai:** `crates/nullbyte-core/src/input/gamepad.rs`

**Ką daryti:**
- `gilrs::Gilrs` event pump; polling emu gijoje arba atskiroje gijoje su kanalu
- Prijungimo/atjungimo įvykiai → pranešk UI per Tauri event
- Analoginių ašių deadzone (numatytoji 0.2)

**Įgyvendinta:** `GamepadThread` — dedikuota gija (kaip `EmuThread`/`start_audio_pump`, nes
`gilrs::Gilrs` nėra `Sync`), `next_event_blocking` event pump su 100ms timeout (leidžia
švariai sustoti), `GamepadEvent` kanalu siunčiamas kviečiančiajai pusei. Deadzone (0.2)
taikomas RANKINIU BŪDU (gilrs įmontuoti filtrai išjungti — jie naudoja kiekvieno valdiklio
DB deadzone, ne fiksuotą 0.2 iš MVP.md), tolydžiu remap'u (ne atkirpimu), patikrinta 3
testais (nulinimas, tolydumas ribose, monotoniškumas). Naujas
`commands::input::start_gamepad_pump` persiunčia prisijungimo/atsijungimo įvykius kaip
Tauri `"gamepad-connection"` event'ą UI (mygtukų/ašių įvykiai UI nesiunčiami — jiems
klausytojo dar nėra, P4.2/P4.3).

**Acceptance:**
- [!] Aptinka Xbox, DualShock 4/5, 8BitDo valdiklius — **LAUKIA vartotojo fizinio
      valdiklio** (nė vieno neturėjo po ranka šios sesijos metu, žadėjo turėti už kelių
      valandų — patikrinti vėliau, kai valdiklis bus prieinamas)
- [!] Prijungimas veikiant nesulaužo (hot-plug) — **LAUKIA** tos pačios fizinės patikros
- [x] Veikia macOS — patikrinta: `GamepadThread::spawn()` sėkmingai inicializuoja `gilrs`
      ir švariai baigia darbą net be jokio prijungto valdiklio (`cargo test`,
      `spawn_does_not_panic_without_any_gamepad`). [!] Linux — NEPATIKRINTA (nėra Linux
      mašinos šioje sesijoje, ta pati priežastis kaip P2.3/P2.5/P3.1)

---

### P4.2 — Įvesties mapping'as `[ ]`

**Priklausomybės:** P4.1, P4.0.2 (klaviatūros mapping'ui reikia realių winit `KeyboardInput`
įvykių iš `nullbyte-emu` — Tauri `Window` jų neturėjo, žr. ADR-016)
**Failai:** `crates/nullbyte-core/src/input/mapping.rs`

> **Pastaba (ADR-016, 2026-08-20):** šis task'as sustabdytas prieš pradedant kodą, kai
> paaiškėjo, kad klaviatūros mapping'o įgyvendinti neįmanoma be proceso architektūros
> pakeitimo (žr. Fazė 4a / P4.0.x aukščiau). Gamepad mapping'o pusė NEBLOKUOJAMA — galima
> pradėti nuo jos, kol P4.0.x vyksta. „Mapping'as saugomas DB" priklauso nuo P5.1 (SQLite
> schema dar neegzistuoja) — iki tada laikyk in-memory (žr. sesijos susitarimą 2026-08-20).

**Ką daryti:**
- Fizinis mygtukas → `RETRO_DEVICE_ID_JOYPAD_*` (B,Y,SELECT,START,UP,DOWN,LEFT,RIGHT,A,X,L,R,L2,R2,L3,R3)
- Numatytieji mapping'ai pagal valdiklio tipą
- **Dėmesio:** libretro `A`/`B` yra SNES išdėstyme — Xbox valdiklio `A` fiziškai atitinka
  libretro `B`. Numatytasis mapping'as turi tai gerbti.
- Klaviatūros numatytieji: strėlės + `Z`/`X`/`A`/`S` + `Enter`/`Shift`
- Mapping'as saugomas DB (per-vartotoją, per-platformą)

**Acceptance:**
- [ ] SNES žaidimas valdomas gamepad'u teisingai
- [ ] Klaviatūra veikia
- [ ] Mapping'as išlieka po perkrovimo

---

### P4.3 — Įvesties polling ir input bitmask `[ ]`

**Priklausomybės:** P4.2, P1.4
**Failai:** `crates/nullbyte-core/src/input/mod.rs`, `crates/nullbyte-core/src/core/callbacks.rs`

> **Pastaba (2026-08-20):** dauguma šio task'o „Ką daryti" punktų jau įgyvendinti anksčiau —
> `EmuContext.input_state: [u16; 4]` (4 portai), `input_state_cb` (P1.4) ir
> `GET_INPUT_BITMASKS => true` (P1.5) jau egzistuoja `core/callbacks.rs`/`core/environment.rs`.
> Šio task'o TIKRASIS likęs darbas — sujungti P4.1 (`gilrs`) ir P4.2 (mapping) su šiuo jau
> veikiančiu bitmask sluoksniu per `EmuCommand::SetInput`, IR pačios ARCHITEKTŪROS pokytis:
> žr. ADR-016 — nuo šios užduoties emuliavimo gija (taigi ir `input_state` atnaujinimas)
> gyvena `nullbyte-emu` vaiko procese, ne Tauri procese. Grįžtame prie šio task'o po ADR-016
> dokumentacijos atnaujinimo (žr. pokalbio kontekstą).

**Ką daryti:**
- `retro_set_input_poll` → atnaujina `EmuContext.input_state`
- `retro_set_input_state` → grąžina iš to būvio
- Palaikyk `GET_INPUT_BITMASKS` (greičiau — vienas kvietimas vietoj 16)
- Iki 4 portų (multiplayer)

**Acceptance:**
- [ ] Įvesties delsa nejuntama (subjektyviai; objektyviai < 1 kadras)
- [ ] 2 gamepad'ai vienu metu veikia (testuok su 2-player žaidimu)

---

### P4.4 — Hotkey'ai `[ ]`

**Priklausomybės:** P4.3
**Failai:** `crates/nullbyte-core/src/input/mod.rs`

**Ką daryti:**

| Klavišas | Veiksmas |
|---|---|
| `F1` | Pauzė / tęsti |
| `F2` | Quick save |
| `F4` | Quick load |
| `F5`–`F8` | Save state slot 1–4 |
| `Shift+F5`–`F8` | Load state slot 1–4 |
| `F11` | Fullscreen |
| `Space` (laikant) | Fast-forward |
| `Esc` | Išeiti iš fullscreen / grįžti į biblioteką |
| `Cmd/Ctrl+R` | Reset |

**Acceptance:**
- [ ] Visi hotkey'ai veikia
- [ ] Nekonfliktuoja su žaidimo įvestimi

---

## 7. Faza 5 — Duomenų bazė ir biblioteka

**Tikslas:** ROM skenavimas ir saugojimas SQLite.
**Rizika:** 🟢 maža. **Įvertis:** 2–3 dienos.

### P5.1 — SQLite schema ir migracijos `[ ]`

**Priklausomybės:** P0.3
**Failai:** `crates/nullbyte-app/migrations/001_initial.sql`, `crates/nullbyte-app/src/db/migrations.rs`, `db/models.rs`

**Schema:**

```sql
PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

CREATE TABLE platforms (
    id                INTEGER PRIMARY KEY,
    slug              TEXT NOT NULL UNIQUE,   -- 'snes', 'nes', 'psx'
    name              TEXT NOT NULL,
    screenscraper_id  INTEGER,                -- systemeid
    extensions        TEXT NOT NULL           -- 'sfc,smc,fig'
);

CREATE TABLE games (
    id                INTEGER PRIMARY KEY,
    platform_id       INTEGER NOT NULL REFERENCES platforms(id),
    title             TEXT NOT NULL,
    sort_title        TEXT NOT NULL,          -- be 'The ', lowercase
    rom_path          TEXT NOT NULL UNIQUE,
    rom_size          INTEGER NOT NULL,
    archive_inner     TEXT,                   -- failas archyve, jei toks
    crc32             TEXT,
    md5               TEXT,
    sha1              TEXT,
    description       TEXT,
    developer         TEXT,
    publisher         TEXT,
    genre             TEXT,
    players           INTEGER,
    release_date      TEXT,
    rating            REAL,
    region            TEXT,
    cover_path        TEXT,                   -- SANTYKINIS kelias
    screenshot_path   TEXT,
    wheel_path        TEXT,
    video_path        TEXT,
    scrape_status     TEXT NOT NULL DEFAULT 'pending',  -- pending|ok|notfound|error
    scraped_at        INTEGER,
    last_played       INTEGER,
    play_count        INTEGER NOT NULL DEFAULT 0,
    play_time_seconds INTEGER NOT NULL DEFAULT 0,
    favorite          INTEGER NOT NULL DEFAULT 0,
    added_at          INTEGER NOT NULL,
    file_mtime        INTEGER NOT NULL        -- pakartotiniam skenavimui
);

CREATE INDEX idx_games_platform  ON games(platform_id);
CREATE INDEX idx_games_sort      ON games(sort_title);
CREATE INDEX idx_games_lastplay  ON games(last_played DESC);
CREATE INDEX idx_games_crc       ON games(crc32);

CREATE VIRTUAL TABLE games_fts USING fts5(
    title, description, content='games', content_rowid='id'
);

CREATE TABLE save_states (
    id           INTEGER PRIMARY KEY,
    game_id      INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    slot         INTEGER NOT NULL,        -- 0 = quick save
    path         TEXT NOT NULL,
    thumb_path   TEXT,
    core_name    TEXT NOT NULL,
    core_version TEXT NOT NULL,
    created_at   INTEGER NOT NULL,
    UNIQUE(game_id, slot)
);

CREATE TABLE cores (
    id             INTEGER PRIMARY KEY,
    path           TEXT NOT NULL UNIQUE,
    name           TEXT NOT NULL,
    version        TEXT,
    extensions     TEXT NOT NULL,
    need_fullpath  INTEGER NOT NULL DEFAULT 0,
    last_seen      INTEGER NOT NULL
);

CREATE TABLE platform_core_prefs (
    platform_id  INTEGER PRIMARY KEY REFERENCES platforms(id),
    core_id      INTEGER NOT NULL REFERENCES cores(id)
);

CREATE TABLE rom_directories (
    id         INTEGER PRIMARY KEY,
    path       TEXT NOT NULL UNIQUE,
    recursive  INTEGER NOT NULL DEFAULT 1,
    enabled    INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE settings (
    key    TEXT PRIMARY KEY,
    value  TEXT NOT NULL
);

CREATE TABLE scrape_cache (
    hash_key   TEXT PRIMARY KEY,   -- 'crc:ABCD1234' arba 'name:snes:Super Mario'
    response   TEXT,               -- JSON arba NULL jei notfound
    found      INTEGER NOT NULL,
    fetched_at INTEGER NOT NULL
);
```

**Ką daryti:**
- Migracijos per `PRAGMA user_version`
- `platforms` lentelė užpildoma seed duomenimis (bent 20 platformų su ScreenScraper ID)
- `AppState` laiko `Mutex<Connection>`

**Acceptance:**
- [ ] DB sukuriama pirmą kartą paleidus
- [ ] Migracijos idempotentiškos (paleisk 3 kartus)
- [ ] Seed platformos įrašytos

---

### P5.2 — ROM hash'avimas `[ ]`

**Priklausomybės:** P5.1
**Failai:** `crates/nullbyte-app/src/library/hasher.rs`

**Ką daryti:**
- CRC32 (`crc32fast`), MD5 (`md-5`), SHA1 (`sha1`) — vienu perėjimu per failą
- Archyvams — hash'uok **vidinį** failą
- Failams > 64 MB: streaming, ne visas į atmintį
- **Header skip:** kai kurioms sistemoms hash skaičiuojamas be header'io
  (NES iNES 16 baitų, SNES 512 baitų copier header) — implementuok bent NES atvejį

**Acceptance:**
- [ ] Hash'ai sutampa su `sha1sum`/`md5sum` komandinės eilutės rezultatais
- [ ] 100 failų (~2 GB) hash'avimas < 30 s SSD'e
- [ ] `.zip` vidinis failas hash'uojamas teisingai

---

### P5.3 — ROM skeneris `[ ]`

**Priklausomybės:** P5.2
**Failai:** `crates/nullbyte-app/src/library/scanner.rs`

**Ką daryti:**
- `walkdir` per `rom_directories`
- Plėtinys → platforma (per `platforms.extensions`)
- Nežinomi plėtiniai praleidžiami tyliai
- **Inkrementinis skenavimas:** jei `rom_path` + `file_mtime` nepasikeitė — praleisk
- Ištrintų failų aptikimas → pažymėk arba pašalink
- Pavadinimo valymas: `Super Mario World (USA) [!].sfc` → `Super Mario World`,
  regionas → `region` laukas
- Progresas per `Channel<ScanProgress { current, total, current_file }>`
- **Viena SQLite transakcija** visam skenavimui

**Acceptance:**
- [ ] 500 ROM'ų katalogas nuskenuojamas < 60 s
- [ ] Pakartotinis skenavimas be pakeitimų < 2 s
- [ ] Progresas rodomas realiu laiku
- [ ] Pavadinimai išvalyti teisingai (testai su 20 pavyzdžių)

---

### P5.4 — Bibliotekos užklausos `[ ]`

**Priklausomybės:** P5.3
**Failai:** `crates/nullbyte-app/src/db/games.rs`, `crates/nullbyte-app/src/commands/library.rs`

**Ką daryti:**
- `list_games(filter)` — filtras: platforma, paieška (FTS5), favorite, rūšiavimas, puslapiavimas
- `get_game(id)`, `set_favorite(id, bool)`, `record_play(id, seconds)`
- `list_platforms()` — su žaidimų kiekiu
- Tauri komandos plonos, grąžina `camelCase` JSON

**Acceptance:**
- [ ] Paieška „mario" randa visus Mario žaidimus < 50 ms su 5000 įrašų
- [ ] Puslapiavimas veikia
- [ ] TS tipai atitinka Rust struct'us

---

## 8. Faza 6 — ScreenScraper

**Tikslas:** metaduomenys, viršeliai ir gameplay video.
**Rizika:** 🟡 vidutinė (išorinis API, kvotos). **Įvertis:** 2–3 dienos.

### P6.1 — API klientas `[ ]`

**Priklausomybės:** P5.1
**Failai:** `crates/nullbyte-app/src/scraper/screenscraper.rs`, `crates/nullbyte-app/src/scraper/types.rs`

**Ką daryti:**
- `reqwest` klientas į `https://www.screenscraper.fr/api2/jeuInfos.php`
- Parametrai pagal `CLAUDE.md` §9.1
- Strategija: `crc`+`md5`+`sha1`+`romtaille` → jei nerado, `romnom`+`systemeid`
- JSON atsako struct'ai (`serde`) — atsargiai, ScreenScraper JSON yra nenuoseklus
  (kartais masyvas, kartais objektas → naudok `#[serde(untagged)]` kur reikia)
- Regionų ir kalbų prioritetai iš `CLAUDE.md` §9.2

**Acceptance:**
- [ ] Žinomas SNES ROM'as randa teisingus metaduomenis
- [ ] Nežinomas ROM'as → `NotFound`, ne klaida
- [ ] Blogas JSON nesulaužo (graceful degradation)
- [ ] Credentials iš `.env` / nustatymų, ne hardcode

---

### P6.2 — Rate limiting ir cache `[ ]`

**Priklausomybės:** P6.1
**Failai:** `crates/nullbyte-app/src/scraper/rate_limit.rs`, `scrape_cache` lentelė

**Ką daryti:**
- Prieš užklausą — tikrink `scrape_cache`
- Cache'uok ir sėkmes (be TTL), ir „notfound" (TTL 7 dienos)
- Semaforas pagal `ssuser.maxthreads` iš atsako (numatytoji 1)
- Exponential backoff: 429/430/`API closed` → 2s, 4s, 8s, 16s, tada sustok
- Kvotos likutis iš atsako (`ssuser.requeststoday` / `maxrequestsperday`) → rodyk UI

**Acceptance:**
- [ ] Pakartotinė užklausa nesikreipia į tinklą
- [ ] Kvotos viršijimas nesulaužo — sustoja su aiškiu pranešimu UI
- [ ] Vienalaikių užklausų nedaugiau nei `maxthreads`

---

### P6.3 — Media atsisiuntimas `[ ]`

**Priklausomybės:** P6.2
**Failai:** `crates/nullbyte-app/src/scraper/media.rs`

**Ką daryti:**
- Atsisiųsk `box-2D`, `ss`, `wheel`, `video-normalized` (fallback `video`)
- Saugok į `media_dir()` pagal `CLAUDE.md` §9.4 struktūrą
- DB laiko **santykinius** kelius
- Praleisk, jei failas jau egzistuoja ir dydis > 0
- Video dydžio limitas (numatytasis 10 MB) — didesnius praleisk

**Acceptance:**
- [ ] Viršeliai ir video atsisiunčia
- [ ] Nutrūkęs atsisiuntimas nepalieka sugadinto failo (rašyk į `.tmp`, tada `rename`)
- [ ] Pakartotinis scraping'as nesiunčia to paties dar kartą

---

### P6.4 — Scraping orkestracija `[ ]`

**Priklausomybės:** P6.3
**Failai:** `crates/nullbyte-app/src/commands/scraper.rs`

**Ką daryti:**
- `scrape_game(id)` — vienas žaidimas
- `scrape_library(platform_id?)` — visi `scrape_status = 'pending'`
- Progresas per `Channel<ScrapeProgress { current, total, title, status, quota_left }>`
- Atšaukimas (`CancellationToken`)
- **Niekada automatiškai** — tik vartotojui paspaudus

**Acceptance:**
- [ ] 50 žaidimų scraping'as baigiasi be klaidų
- [ ] Progresas realiu laiku
- [ ] Atšaukimas veikia iškart
- [ ] Kvotos pabaiga sustabdo švariai

> **Milestone M4:** biblioteka pilna su metaduomenimis.

---

## 9. Faza 7 — UI

**Tikslas:** tikras, gražus, naudojamas interfeisas.
**Rizika:** 🟢 maža. **Įvertis:** 4–5 dienos.

### P7.1 — Layout ir navigacija `[ ]`

**Priklausomybės:** P0.2, P5.4
**Failai:** `src/routes/+layout.svelte`, `src/lib/components/layout/*`

**Ką daryti:**
- Sidebar: platformų sąrašas su žaidimų kiekiu, „Visi", „Mėgstami", „Neseniai žaisti"
- TopBar: paieška, rūšiavimo pasirinkimas, nustatymų mygtukas
- Command palette (`Cmd/Ctrl+K`) — shadcn `command` komponentas
- Tamsi tema, tanki, klaviatūra naviguojama

**Acceptance:**
- [ ] Sidebar rodo tikras platformas iš DB
- [ ] `Cmd+K` atidaro paletę
- [ ] Klaviatūros navigacija veikia (Tab, strėlės)

---

### P7.2 — Žaidimų grid'as ir kortelės `[ ]`

**Priklausomybės:** P7.1
**Failai:** `src/lib/components/library/GameCard.svelte`, `GameGrid.svelte`

**Ką daryti:**
- Responsive grid, viršelio proporcijos, `Skeleton` kol kraunasi
- Viršeliai per `convertFileSrc()` (asset protokolas), ne base64
- **Virtualizacija** (`@tanstack/svelte-virtual`) — privaloma
- Placeholder žaidimams be viršelio (platformos spalva + pavadinimas)
- Hover: pakilimas, šešėlis, pavadinimo overlay

**Acceptance:**
- [ ] 5000 žaidimų grid'as slenka 60 FPS
- [ ] Viršeliai rodomi
- [ ] Be viršelio — tvarkingas placeholder

---

### P7.3 — Video preview 🟡 `[ ]`

**Priklausomybės:** P7.2, P6.3
**Failai:** `src/lib/components/library/VideoPreview.svelte`

**Ką daryti:**
- Hover 300 ms → prasideda video (`muted`, `loop`, `playsinline`, `preload="none"`)
- Fade-in perėjimas nuo viršelio prie video
- Mouse leave → sustabdyk, `currentTime = 0`, atlaisvink
- **Vienu metu groja tik VIENAS video** — globalus „aktyvus preview" būvis
- `$effect` cleanup privalomas (kitaip liks groti fone)

**Acceptance:**
- [ ] Greitai slenkant pele video nepradeda groti (debounce veikia)
- [ ] Niekada negroja 2 video vienu metu
- [ ] Atminties naudojimas nekyla slenkant per 100 kortelių

---

### P7.4 — Žaidimo detalių puslapis `[ ]`

**Priklausomybės:** P7.2
**Failai:** `src/routes/game/[id]/+page.svelte`

**Ką daryti:**
- Hero: screenshot fone (blur + gradientas), wheel logotipas viršuje
- Metaduomenys: aprašymas, kūrėjas, leidėjas, data, žanras, žaidėjų kiekis, reitingas
- Mygtukai: „Žaisti", „Mėgstamas", „Scrape iš naujo"
- Save states sąrašas su preview paveiksliukais
- Statistika: paskutinį kartą žaista, kiek kartų, kiek laiko

**Acceptance:**
- [ ] Visi metaduomenys rodomi
- [ ] „Žaisti" paleidžia žaidimą
- [ ] Trūkstami duomenys nesulaužo layout'o

---

### P7.5 — Skenavimo ir scraping'o UI `[ ]`

**Priklausomybės:** P5.3, P6.4, P7.1
**Failai:** `src/lib/components/settings/PathsPanel.svelte`, progreso komponentai

**Ką daryti:**
- ROM katalogų pridėjimas per Tauri dialog plugin
- „Skenuoti" mygtukas → progreso juosta su dabartiniu failu
- „Scrape library" → progresas + kvotos likutis
- Atšaukimo mygtukas
- Rezultatų santrauka (rasta / nerasta / klaidos)

**Acceptance:**
- [ ] Katalogo pridėjimas veikia abiejose platformose
- [ ] Progresas sklandus, be UI užšalimo
- [ ] Atšaukimas veikia

---

### P7.6 — Nustatymų ekranas `[ ]`

**Priklausomybės:** P7.1, P4.2
**Failai:** `src/routes/settings/+page.svelte`, `src/lib/components/settings/*`

**Ką daryti:**
- **Keliai:** ROM katalogai, core'ų katalogas, BIOS katalogas
- **Core'ai:** aptiktų core'ų sąrašas, pasirinkimas per platformą
- **Vaizdas:** filtras (nearest/linear), integer scaling, vsync, fullscreen numatytasis
- **Garsas:** įrenginys, garsumas, buferio dydis
- **Įvestis:** valdiklių sąrašas, mygtukų perrišimas (spaudi mygtuką → priskiria)
- **Scraper:** ScreenScraper login, regionų prioritetas, media tipai, kvotos likutis

**Acceptance:**
- [ ] Visi nustatymai išsaugomi DB ir taikomi
- [ ] Mygtukų perrišimas veikia
- [ ] Neteisingi ScreenScraper credentials duoda aiškią klaidą

---

## 10. Faza 8 — Išsaugojimai

**Tikslas:** progresas neprapuola.
**Rizika:** 🟡 vidutinė. **Įvertis:** 1–2 dienos.

### P8.1 — Save states `[ ]`

**Priklausomybės:** P1.7, P5.1
**Failai:** `crates/nullbyte-core/src/core/savestate.rs`

**Ką daryti:**
- `retro_serialize_size()` **prieš kiekvieną** išsaugojimą
- Failas: `states_dir()/{game_id}_{slot}.state`
- Metaduomenys DB: core pavadinimas + versija, laikas
- Preview paveiksliukas: paimk dabartinį kadrą iš triple buffer → PNG
- Įkeliant: jei core nesutampa — įspėjimas UI, bet leisk bandyti
- **Kviesk tik iš emuliavimo gijos, tarp `retro_run()`**

> **Pastaba (ADR-016, 2026-08-20):** nuo proceso architektūros pakeitimo triple buffer'is
> (taigi ir dabartinis kadras preview'ui) gyvena `nullbyte-emu` VAIKO procese, o DB — `nullbyte-app`
> TĖVO procese. „Paimk kadrą → PNG" veiksmas turi vykti VAIKO pusėje (jis turi tiesioginę
> prieigą prie triple buffer'io): `nullbyte-emu` pats užkoduoja dabartinį kadrą į PNG, įrašo
> failą į diską (`states_dir()`), ir per IPC grąžina TIK failo kelią (`String`/`PathBuf`) —
> NE žalius kadro baitus. Tėvas tik įrašo tą kelią į DB. Tai atitinka §10 „IPC riba turi
> likti PLONA" taisyklę.

**Acceptance:**
- [ ] Save → uždaryti → paleisti → load → tas pats taškas
- [ ] 4 slot'ai + quick save nepersidengia
- [ ] Preview paveiksliukas teisingas
- [ ] Kito core state → įspėjimas, ne crash

---

### P8.2 — SRAM `[ ]`

**Priklausomybės:** P1.7
**Failai:** `crates/nullbyte-core/src/core/savestate.rs`

**Ką daryti:**
- `retro_get_memory_data(RETRO_MEMORY_SAVE_RAM)` + `retro_get_memory_size(...)`
- Failas: `saves_dir()/{rom_basename}.srm`
- Įkelk **po** `retro_load_game()`
- Išsaugok: uždarant žaidimą, kas 30 s, ir kai `size > 0` bei turinys pasikeitė
- Atominis rašymas: `.tmp` → `rename`

**Acceptance:**
- [ ] RPG in-game save išlieka po perkrovimo
- [ ] `.srm` failas nesugadinamas staigiai uždarius
- [ ] Core'ai be SRAM (`size == 0`) nesulaužo

---

## 11. Faza 9 — Integracija ir polish

**Tikslas:** viskas veikia kartu, atrodo baigta.
**Rizika:** 🟡 vidutinė. **Įvertis:** 3–4 dienos.

### P9.1 — Žaidimo paleidimo srautas `[ ]`

**Priklausomybės:** P7.4, P2.5, P3.4, P4.4

**Ką daryti:**
- UI „Žaisti" → parink core'ą (per `platform_core_prefs`, arba klausk) → paleisk
- Krovimosi būsena UI
- Klaidos (trūksta core'o, trūksta BIOS, blogas ROM) → aiškūs pranešimai, ne stack trace
- Uždarius žaidimą → grįžti į biblioteką, atnaujinti `last_played` ir `play_time`

**Acceptance:**
- [ ] Paleidimas iš bibliotekos veikia visoms platformoms
- [ ] Trūkstamas core → suprantamas pranešimas su nurodymu ką daryti
- [ ] Žaidimo laikas fiksuojamas

---

### P9.2 — Core'ų perjungimas ir izoliacija 🔴 `[ ]`

**Priklausomybės:** P9.1

**Ką daryti:**
- Ištestuok perjungimą tarp 5+ skirtingų core'ų toje pačioje sesijoje
- Jei atsiranda crash'ų dėl globalaus būvio (`CLAUDE.md` §10) — sprendimai eilės tvarka:
  1. Griežtesnis `deinit` + `dlclose` + pauzė
  2. Neleisti perjungti be aplikacijos restarto (MVP priimtina, dokumentuok)
  3. Child proceso izoliacija (didelis darbas — tik jei būtina)

**Acceptance:**
- [ ] 10 core perjungimų iš eilės be crash'o **ARBA** dokumentuotas apribojimas
- [ ] Atmintis neauga po kiekvieno perjungimo (patikrink 10 ciklų)

---

### P9.3 — Klaidų apdorojimas ir tuščios būsenos `[ ]`

**Priklausomybės:** visos ankstesnės

**Ką daryti:**
- Kiekviena `AppError` variacija turi žmogui suprantamą tekstą UI
- Toast'ai per `sonner`
- Tuščios būsenos: nėra ROM katalogų, nėra core'ų, tuščia paieška — su veiksmo pasiūlymu
- Pirmo paleidimo srautas (onboarding): pridėk core'us → pridėk ROM'us → skenuok

**Acceptance:**
- [ ] Nė vienas klaidos kelias nerodo Rust panic teksto vartotojui
- [ ] Švari instaliacija → aiškus kelias ką daryti

---

### P9.4 — Našumas `[ ]`

**Priklausomybės:** visos ankstesnės

**Ką daryti:**
- Profiliuok: SNES turi veikti < 15 % CPU šiuolaikiniame procesoriuje
- Bibliotekos užkrovimas su 5000 žaidimų < 500 ms
- Atminties naudojimas idle < 200 MB
- Pašalink `--release` build'e visus `debug!` iš karštų kelių

**Acceptance:**
- [ ] Visi trys skaičiai pasiekti
- [ ] Nėra atminties nutekėjimo po 30 min žaidimo

---

### P9.5 — Ikonos, metaduomenys, packaging `[ ]`

**Priklausomybės:** P9.4

**Ką daryti:**
- Aplikacijos ikona (macOS `.icns`, Linux `.png` visų dydžių) — `pnpm tauri icon`
- `tauri.conf.json`: kategorijos, aprašymas, autorius, homepage
- macOS: `universal-apple-darwin` build'as
- Linux: `.AppImage` + `.deb`
- GitHub Actions release workflow

**Acceptance:**
- [ ] `.dmg` atsidaro ir instaliuojasi macOS
- [ ] `.AppImage` veikia švariame Ubuntu
- [ ] Ikona teisinga abiejose platformose

---

### P9.6 — Galutinis MVP patikrinimas `[ ]`

**Priklausomybės:** visos

**Ką daryti:**
- Pereik **visus 7 sėkmės kriterijus** iš §1.4 abiejose platformose
- Švari instaliacija (ištrink `data_dir()`) → visas srautas nuo nulio
- Atnaujink README screenshot'us
- Sukurk `LICENSE` (MIT)
- Tag `v0.1.0`

**Acceptance:**
- [ ] Visi 7 kriterijai ✅ macOS
- [ ] Visi 7 kriterijai ✅ Linux
- [ ] Release build'ai veikia

> **Milestone M5: MVP baigtas.**

---

## 12. Milestone'ai ir progresas

| # | Milestone | Fazės | Įvertis | Statusas |
|---|---|---|---|---|
| M0 | Projektas paleidžiamas | 0 | 1 d. | ✅ |
| M1 | libretro core sukasi headless | 1 | 3–5 d. | ✅ |
| M2 | Vaizdas ekrane | 2 | 3–4 d. | ✅ |
| M3 | Vaizdas + garsas + valdymas | 3, 4 | 10–12 d. (žr. ADR-016 — +4–7 d. P4.0.x migracijai) | 🟡 |
| M4 | Biblioteka su metaduomenimis | 5, 6 | 4–6 d. | ⬜ |
| M5 | **MVP** | 7, 8, 9 | 8–11 d. | ⬜ |

**Bendras įvertis: 27–39 darbo dienos** (vienam žmogui su Claude Code — padidėjo nuo
23–32 d. po ADR-016 proceso architektūros migracijos įtraukimo, 2026-08-20).

### Progreso lentelė

| Faza | Užduočių | Baigta | % |
|---|---|---|---|
| 0 — Pamatai | 5 | 5 | 100 % |
| 1 — libretro | 7 | 7 | 100 % |
| 2 — Vaizdas | 5 | 5 | 100 % |
| 3 — Garsas | 4 | 4 | 100 % |
| 4 — Įvestis (+P4.0.x migracija) | 9 | 1 | 11 % |
| 5 — DB / biblioteka | 4 | 0 | 0 % |
| 6 — ScreenScraper | 4 | 0 | 0 % |
| 7 — UI | 6 | 0 | 0 % |
| 8 — Išsaugojimai | 2 | 0 | 0 % |
| 9 — Polish | 6 | 0 | 0 % |
| **Viso** | **52** | **22** | **42 %** |

---

## 13. Rizikų registras

| ID | Rizika | Tikimybė | Poveikis | Mitigacija |
|---|---|---|---|---|
| **R1** | libretro FFI nestabilumas, segfault'ai callback'uose | Vidutinė | 🔴 Kritinis | P1.4/P1.5 daryti atsargiai, `GET_LOG_INTERFACE` pirmiausia, testuoti su 3+ core'ais anksti |
| **R2** | wgpu + Tauri langas neveikia Linux/Wayland | Vidutinė | 🔴 Kritinis | Dokumentuotas fallback į `Channel` + WebGL canvas (P2.3). Testuoti abu backend'us Fazėje 2, ne pabaigoje |
| **R3** | Garso traškesiai, kurių nepavyksta pašalinti | Vidutinė | 🟡 Didelis | Dynamic rate control yra įrodyta technika (RetroArch). Jei nepavyksta — didink buferį iki 100 ms |
| **R4** | Core'ų globalus būvis neleidžia perjungti be restarto | Aukšta | ✅ **IŠSPRĘSTA** | Child procesas (`nullbyte-emu`) kiekvienam paleidimui — ADR-016 (P4.3, 2026-08-20). Perkelta iš post-MVP į dabar, nes kartu sprendė ir klaviatūros įvesties problemą |
| **R5** | ScreenScraper kvotos per mažos naudingam scraping'ui | Vidutinė | 🟡 Vidutinis | Agresyvus cache'as, batch'inimas, aiškus kvotos rodymas UI. Atsarginis planas: pridėti TheGamesDB |
| **R6** | ScreenScraper dev credentials negaunami | Žema | 🟡 Vidutinis | Kreiptis anksti (Fazėje 0). Alternatyva: TheGamesDB arba OpenVGDB offline |
| **R7** | macOS notarizacija / gatekeeper trukdo platinti | Aukšta | 🟢 Mažas | MVP: instrukcijos README. Post-MVP: Apple Developer paskyra |
| **R8** | Video preview atminties nutekėjimas | Vidutinė | 🟢 Mažas | Griežtas `$effect` cleanup, vienas aktyvus video, testuoti su 100 kortelių |
| **R9** | Svelte 5 / shadcn-svelte breaking changes | Žema | 🟢 Mažas | Užfiksuoti versijas `package.json` be `^` kritinėms |
| **R10** | Scope creep — norisi core options, shader'ių, netplay | Aukšta | 🟡 Vidutinis | §1.3 „NEĮEINA" sąrašas yra įstatymas. Idėjos → `IDEAS.md`, ne į MVP |

---

## 14. Techninių sprendimų žurnalas (ADR)

> Kiekvienas naujas architektūrinis sprendimas ar priklausomybė — nauja eilutė čia.

### ADR-000 — Rust vietoj Zig
**Data:** 2026-08-19 · **Statusas:** priimta
**Kontekstas:** Reikia sistemos kalbos su geru C FFI (libretro API yra C).
**Sprendimas:** Rust.
**Priežastis:** Zig turi elegantiškesnį C interop (`@cImport` be boilerplate) ir greitesnę
kompiliaciją, bet jo ekosistema per jauna šiam projektui — nėra `wgpu`, `cpal`, `rusqlite`,
`gilrs` atitikmenų, todėl tektų rašyti bindings patiems. Rust duoda paruoštus visus reikalingus
sluoksnius, o borrow checker apsaugo nuo memory bug'ų, kurie FFI ir real-time kode yra dažni
ir sunkiai randami. Zig taip pat dar nestabilus (API keičiasi tarp versijų).
**Alternatyvos svarstytos:** Zig, C++, Go, Swift (tik macOS).
**Pasekmės:** Lėtesnė kompiliacija ir daugiau FFI boilerplate'o nei Zig atveju; borrow checker
reikalauja atsargumo su `unsafe` sluoksniu (žr. `CLAUDE.md` §6.2).

### ADR-001 — libretro vietoj savos core API
**Data:** 2026-08-19 · **Statusas:** priimta
**Kontekstas:** Reikia core sistemos. OpenEmu turi savo `.oecoreplugin` API.
**Sprendimas:** libretro.
**Priežastis:** Šimtai paruoštų core'ų, aktyviai palaikoma, dokumentuota. Sava API reikštų
kiekvieno emuliatoriaus adaptavimą ranka — metų darbas.
**Pasekmės:** Priklausomybė nuo libretro API stabilumo (kuri yra labai stabili — v1 nuo 2010 m.).

### ADR-002 — Tauri v2 vietoj Electron / natyvaus UI
**Data:** 2026-08-19 · **Statusas:** priimta
**Kontekstas:** Reikia macOS + Linux UI su dizaino laisve.
**Sprendimas:** Tauri v2 + WebView.
**Priežastis:** Rust backend'as natūraliai tinka FFI ir real-time darbui. WebView duoda
maksimalią dizaino laisvę. Tauri žymiai lengvesnis nei Electron (nėra Chromium).
**Alternatyvos svarstytos:** Slint (mažesnė ekosistema), egui (atrodo kaip dev tool),
Swift+SwiftUI (tik macOS), Qt (C++, licencija).
**Pasekmės:** Vaizdo atvaizdavimas negali eiti per WebView → reikia atskiro native lango (ADR-005).

### ADR-003 — Svelte 5 vietoj React / Solid / Vue
**Data:** 2026-08-19 · **Statusas:** priimta
**Sprendimas:** Svelte 5 su runes.
**Priežastis:** Mažiausiai kodo, kompiliuojasi į vanilla JS (nėra runtime), greičiausias
grid'ams su tūkstančiais elementų. shadcn-svelte duoda gerus komponentus.
**Pasekmės:** Mažesnė ekosistema nei React; kai kurių bibliotekų reikės ieškoti alternatyvų.

### ADR-004 — ScreenScraper vietoj IGDB / OpenVGDB / TheGamesDB
**Data:** 2026-08-19 · **Statusas:** priimta
**Sprendimas:** ScreenScraper API v2.
**Priežastis:** Vienintelis šaltinis, turintis **gameplay video** — pagrindinį produktinį
skirtumą. Taip pat ROM hash matching (tikslesnis nei pavadinimu). Naudoja Batocera,
Recalbox, EmulationStation.
**Alternatyvos:** IGDB (nėra video snaps), OpenVGDB (offline, bet nėra video, retai
atnaujinama), TheGamesDB (mažiau pilna).
**Pasekmės:** Priklausomybė nuo išorinio serviso ir jo kvotų → privalomas agresyvus cache'as (P6.2).

### ADR-005 — wgpu atskirame native lange, ne WebView canvas
**Data:** 2026-08-19 · **Statusas:** priimta, PAPILDYTA ADR-016 (2026-08-20)
**Kontekstas:** Kadrus reikia rodyti 60 k./s.
**Sprendimas:** Atskiras native langas be webview + wgpu `Surface` per `raw-window-handle`.
**Priežastis:** Kadrų siuntimas per IPC į canvas neskaluojasi: 640×480×4 B × 60 = 73 MB/s.
Native langas duoda zero-copy GPU kelią ir vsync.
**Alternatyva (fallback):** `Channel<&[u8]>` → WebGL2 canvas. Priimtina 8/16-bit sistemoms
(256×224 ≈ 7 MB/s), bet ne N64/PSP — šis fallback'as NEBUS naudojamas MVP metu (žr. žemiau).
**Pasekmės:** Du langai vietoj vieno. Reikia atskirai spręsti fullscreen, fokusą, hotkey'us.

> **Papildymas (ADR-016, P4.3, 2026-08-20):** originalus sprendimas („atskiras **Tauri**
> `Window` be webview") pasirodė nepilnas — patikrinta, kad toks langas Tauri v2 neturi
> JOKIO klaviatūros event'ų API. Paaiškėjus šiai spragai KARTU su neišspręsta R4 rizika
> (core'ų globalus būvis), esminis sprendimas „native langas, ne WebView canvas" **LIEKA
> GALIOJANTIS**, bet native langas dabar priklauso **atskiram `nullbyte-emu` vaiko
> procesui** (winit), ne Tauri procesui. Fallback'o (WebGL2 canvas) svarstyta ir sąmoningai
> ATMESTA kaip pagrindinis kelias, nes ji apribotų platformų palaikymą (N64/GameCube/PSP —
> žr. README) ir vis tiek nebūtų išsprendusi R4. Pilna nauja architektūra — ADR-016.

### ADR-006 — Audio-driven sinchronizacija
**Data:** 2026-08-19 · **Statusas:** priimta
**Sprendimas:** Garso plokštė yra laikrodis; emuliavimo greitis derinamas prie garso buferio.
**Priežastis:** Ausis daug jautresnė garso trikdžiams nei akis kadrų netolygumui.
Tai standartinė RetroArch technika (dynamic rate control).
**Pasekmės:** Frame pacing priklauso nuo garso; žaidimams be garso reikia atsarginio pacing'o.

### ADR-007 — `thread_local!` vietoj `static mut` callback kontekstui
**Data:** 2026-08-19 · **Statusas:** priimta
**Kontekstas:** libretro callback'ai neturi `user_data` parametro.
**Sprendimas:** `thread_local! { static CTX: RefCell<Option<EmuContext>> }`.
**Priežastis:** `static mut` yra `deny`-by-default Rust 2024. Visi kvietimai iš vienos gijos →
`thread_local` yra saugus ir be sinchronizacijos kaštų.
**Pasekmės:** Visi `retro_*` kvietimai **privalo** eiti iš tos pačios gijos. Užfiksuota
`CLAUDE.md` §3.2.

### ADR-008 — SQLite (rusqlite) vietoj JSON / sled / SurrealDB
**Data:** 2026-08-19 · **Statusas:** priimta
**Sprendimas:** SQLite su `rusqlite` + `bundled`.
**Priežastis:** FTS5 paieškai, transakcijos skenavimui, brandi, vienas failas, lengva
backup'inti. `bundled` — vienodas elgesys abiejose platformose.
**Pasekmės:** `Connection` nėra `Sync` → `Mutex<Connection>` `AppState`'e.

### ADR-009 — shadcn-svelte vietoj Skeleton / Melt / DaisyUI
**Data:** 2026-08-19 · **Statusas:** priimta
**Sprendimas:** shadcn-svelte + Tailwind v4.
**Priežastis:** Komponentai kopijuojami į projektą → pilna kontrolė ir galimybė juos laisvai
keisti, be „kovos su biblioteka". Didžiausia bendruomenė ir daugiausia pavyzdžių iš Svelte
komponentų bibliotekų.
**Alternatyvos:** Skeleton UI (greitesnis startas gaming estetikai, bet mažesnė bendruomenė),
Melt UI (headless — maksimali laisvė, bet žymiai daugiau darbo), DaisyUI (greita, bet sunku
pasiekti unikalų dizainą).
**Pasekmės:** Reikia mokėti Tailwind. `src/lib/components/ui/` yra generuotas kodas —
sava logika ten nerašoma (žr. `CLAUDE.md` §7.4).

### ADR-010 — `tracing-appender` log failų rotacijai
**Data:** 2026-08-19 · **Statusas:** priimta
**Kontekstas:** P0.5 reikalauja log failo su rotacija `data_dir()/logs/`; `tracing-subscriber`
pats savaime rašo tik į `Write` (pvz. stdout), neturi rotuojančio failų writer'io.
**Sprendimas:** `tracing-appender` (oficialus `tokio-rs/tracing` palydovas) —
`rolling::daily` + `non_blocking` writer.
**Priežastis:** Ta pati organizacija kaip `tracing`/`tracing-subscriber` (jau §2 sąraše), maža,
gerai palaikoma, `non_blocking` neblokuoja UI/emuliavimo gijų rašydama į diską.
**Pasekmės:** `WorkerGuard` privalo gyventi tol, kol veikia programa (laikomas `run()` viduje);
jį numetus, likę log'ai gali neišsirašyti į failą.

### ADR-011 — `GET_LOG_INTERFACE`: transmute'inta ne-variadic funkcija vietoj tikros C-variadic
**Data:** 2026-08-19 · **Statusas:** priimta (laikina, iki Rust c_variadic stabilizacijos)
**Kontekstas:** libretro `retro_log_printf_t` yra C-variadic (`void (*)(level, fmt, ...)`).
Stabilus Rust dar negali *apibrėžti* C-variadic funkcijų (rust-lang/rust#44930 — planuojama
Rust 1.99). Patikrinau ir `printf-compat` crate — jis irgi reikalauja `c_variadic` feature,
t.y. nightly toolchain, kas prieštarautų CLAUDE.md §2 „Rust toolchain: stable".
**Sprendimas:** `core_log_printf` apibrėžta kaip įprasta (ne-variadic) `unsafe extern "C" fn(level, fmt)`,
o jos rodyklė `transmute`'inama į `retro_log_printf_t` tipą prieš perduodant core'ui per
`retro_log_callback.log`. Ji priima tik `level`+`fmt`, NESKAITO varargs.
**Priežastis:** System V AMD64 ir AAPCS64 (macOS + Linux, x86_64 + aarch64 — vieninteliai
mūsų taikiniai) kalbimo konvencijose fiksuotų parametrų perdavimas identiškas variadic ir
ne-variadic funkcijoms; papildomi varargs tiesiog lieka neperskaityti steke/registruose.
Empiriškai patikrinta atskiru scratch projektu prieš įtraukiant į kodą.
**Pasekmės:** Core'ų log pranešimai su `%s`/`%d` ir pan. formatavimo simboliais bus rodomi
NEIŠPLĖSTU (neapdorotu) formatu — prarandami dinaminiai argumentai, bet pati eilutė vis
tiek naudinga debug'inant. Kai Rust 1.99 stabilizuos `c_variadic` (arba `printf-compat`
pereis į stable), verta grįžti ir implementuoti pilną formatavimą.

### ADR-012 — `spin_sleep` kadrų pacing'ui (P1.7)
**Data:** 2026-08-20 · **Statusas:** priimta
**Kontekstas:** P1.7 MVP frame pacing reikalauja tikslaus laukimo iki `1.0 / fps` tarp
`retro_run()` kvietimų (acceptance: „~60 FPS ±1"). Grynas `std::thread::sleep` turi OS
scheduler'io netikslumą (dažnai 1–15 ms), kurio nepakanka šiam tikslumui.
**Sprendimas:** `spin_sleep` — miega natūraliu `sleep` tiek, kiek platforma patikimai leidžia,
paskutinę dalį „spin"-ina (busy-wait), kad pasiektų sub-milisekundinį tikslumą.
**Priežastis:** Plačiai naudojama (2M+ atsisiuntimų), viena paskirtis, jokių papildomų
priklausomybių medžio. Bus pakeista audio-driven sinchronizacija P3.4 (šis frame pacing —
tik laikinas MVP sprendimas, kol garso buferis nėra laikrodis).
**Pasekmės:** `spin_sleep::sleep_until(deadline)` naudojamas vietoj rankiniu būdu skaičiuojamo
delta — išvengia dreifo, kaupiantis apvalinimo klaidoms per daug kadrų.

---

### ADR-013 — `CORE_LOAD_LOCK` testams: serializuoti realaus core'o dlopen/init (P2.4)
**Data:** 2026-08-20 · **Statusas:** priimta
**Kontekstas:** Po P2.4 pakeitimų `cargo test` (numatytasis, lygiagretus) pradėjo intermituotai
žlugti su `SIGSEGV`. Priežastis — CLAUDE.md §3.2 taisyklė #2 (procese vienu metu gali būti
įkeltas tik VIENAS core) buvo pažeidžiama pačių testų: keli testai skirtingose gijose
vienu metu `dlopen`'ina ir `retro_init()`'ina TĄ PATĮ realų core'ą (snes9x/mednafen_psx/
genesis_plus_gx), kurių globalus (ne thread-local) C būvis nėra reentrant. Vienu srautu
(`--test-threads=1`) visi 42 testai praeidavo be klaidos — patvirtino, kad tai lygiagretumo,
ne logikos, problema.
**Sprendimas:** `core::test_support::CORE_LOAD_LOCK` — testų-tik (`#[cfg(test)]`) globalus
`Mutex<()>`, kurį paima kiekvienas testas prieš realaus core'o `CoreHandle::load()` +
`init()`/`load_game()` arba `EmuThread::spawn()` + `EmuCommand::Load`. `lock_core_load()`
atstato `PoisonError` per `into_inner()`, kad vieno testo panic'as nesustabdytų kitų.
**Priežastis:** Serializuoti TIK core'ą liečiančius testus (ne visą test binarą), kad
`cargo test` liktų greitas — visi kiti (pixel_format, frame_buffer, callbacks be realaus
core'o ir t.t.) toliau bėga lygiagrečiai.
**Pasekmės:** `cargo test` (numatytasis) dabar patikimai praeina lygiagrečiai — patikrinta
3 kartus iš eilės po pataisymo, jokio SIGSEGV. Bet koks naujas testas, kuris įkelia realų
`.dylib`/`.so` core'ą, PRIVALO paimti šį užraktą — priešingu atveju rizikuoja tuo pačiu
crash'u.

---

### ADR-014 — Tikras quad'as (4 kampai) vietoj „pilno ekrano trikampio" scale'inamam blit'ui (P2.5)
**Data:** 2026-08-20 · **Statusas:** priimta
**Kontekstas:** P2.4 naudojo standartinį „full-screen triangle" triuką (1 trikampis, 3
viršūnės, be vertex buffer'io) — pigesnis nei quad, populiarus GPU tutorialuose. P2.5
reikėjo sutraukti nupieštą sritį (aspect ratio / integer scaling), tad vertex shader'yje
NDC pozicija tiesiog dauginama iš `scale < 1.0`. **Vizuali verifikacija parodė realią
klaidą:** juoda pillarbox juosta atsirado TIK vienoje pusėje, ne simetriškai. Priežastis —
trikampio triukas veikia TIK todėl, kad jo „perteklinis" kraštas (už NDC [-1,1] ribų)
GPU kerpamas TIKSLIAI ties fiksuota clip riba; padauginus poziciją iš `scale`, viena
trikampio kraštinė (buvusi TIKSLIAI ties riba) susitraukia proporcingai, o kita (buvusi TOLI
už ribos) po dauginimo VIS DAR lieka už ribos ir kerpama prie SENOS fiksuotos vietos —
rezultatas asimetriškas.
**Sprendimas:** Pakeista į tikrą quad'ą — 4 fiksuoti kampai (`array<vec2<f32>, 4>`),
2 trikampiai per indeksų masyvą, 6 viršūnės iš `vertex_index` (vis dar be vertex buffer'io).
Kampai visada tiksliai `(±scale.x, ±scale.y)` — simetrija garantuota konstrukciškai, ne
šalutinis clip'inimo efektas.
**Priežastis:** Vienintelis saugus būdas gauti simetriškai centruotą, savavališkai
sutraukiamą stačiakampį be papildomos geometrijos ar UV perskaičiavimo gudrybių.
**Pasekmės:** Nedidelis GPU kaštas (papildomas trikampis, 3 papildomos viršūnės) —
nereikšmingas šiam pipeline'ui. `pass.draw(0..3, ...)` → `pass.draw(0..6, ...)`. Pamoka
kitiems P2.x/P7.x shader pakeitimams: **kiekvieną vizualų pakeitimą PRIVALOMA patikrinti
realiu ekrano vaizdu, ne vien „kompiliuojasi ir neplaukė crash'as"** — ši klaida būtų
praėjusi visus automatinius testus (jų nėra shader'iams) ir net `cargo clippy`.

---

### ADR-015 — Audio-driven pacing throttle riba ARTI tikslo (0.6), NE toli virš jo (0.9) (P3.4)
**Data:** 2026-08-20 · **Statusas:** priimta
**Kontekstas:** P3.4 audio-driven pacing sustabdo kadrų generavimą, kai audio ring buferio
occupancy pasiekia `BUFFER_HIGH_WATERMARK`. Pirma implementacija naudojo `0.9` — intuityviai
atrodė saugu („toli nuo perpildymo"). **Reali verifikacija parodė priešingai:** occupancy
pasiekė 0.91 ir liko ten, o `overrun_count` augo PASTOVIAI (~6.4/s), nors buferis „turėjo"
turėti 10% laisvos vietos. Priežastis — throttle patikra vyksta TIK prieš kiekvieną
`retro_run()` kadrą; su tolima riba emuliavimo gija bėga VISIŠKAI neatskėta (jokio delsimo
tarp kadrų per `Duration::ZERO` `recv_timeout`) tol, kol occupancy pasiekia 0.9 — o vieno
kadro audio porcija (P3.3 `AudioResampler` vieno `process()` kvietimo išvestis) siekia ~8%
viso buferio talpos. Todėl paskutinis prieš-throttle kadras nuolat „prasiverždavo" per ribą.
**Sprendimas:** `BUFFER_HIGH_WATERMARK = 0.6` — ARTI tikslinio ~50% (CLAUDE.md §8.6), NE
toli virš jo. Su artima riba throttle suveikia KIEKVIENĄ kadrą, kai occupancy artėja prie
tikslo, priversdamas kadrų spartą sekti consumer'io (audio aparatūros) nusausinimo greitį —
tai IR YRA audio-driven pacing apibrėžimas, ne šalutinis efektas.
**Priežastis:** Bet kokia riba, mažesnė už `1.0 - (vieno kadro audio dalis)`, negarantuoja
nulinio overrun'o su ŠIA (single-check-per-frame) throttle architektūra — 0.6 paliko
patogią, patikrintą atsargą.
**Pasekmės:** Po pataisymo — patikrinta 10 min realiu paleidimu: `overrun_count=0` per VISĄ
laiką, occupancy stabiliai svyravo ~44–74% (niekada 0%/100%), `measured_fps` atitiko tikrą
core'o fps (~50.0, ne apvalintą). Pamoka: audio-driven pacing ribos parenkamos NE
intuityviai („kuo toliau nuo pilno, tuo saugiau"), o pagal VIENO ŽINGSNIO dydį santykyje su
talpa — patikrinta tik realiu ilgu (10 min) paleidimu, ne trumpu sanity testu (trumpame
teste occupancy dar nespėja pasiekti problemiškos zonos).

---

### ADR-016 — Atskiras `nullbyte-emu` vaiko procesas (winit) emuliacijai, ne Tauri procesas
**Data:** 2026-08-20 · **Statusas:** priimta
**Kontekstas:** P4.2 metu pradėjus dėlioti klaviatūros mapping'ą, paaiškėjo, kad ADR-005
sprendimas („atskiras **Tauri** `Window` be webview" wgpu vaizdui, nuo P2.3) turi realią
spragą: patikrinta prieš `tauri` 2.11.5 šaltinį — tokia `Window` neturi JOKIO klaviatūros
event'ų API (`WindowEvent` enum'e tik `Resized`/`Moved`/`CloseRequested`/`Destroyed`/
`Focused`/`ScaleFactorChanged`/`DragDrop`/`ThemeChanged` — jokio klaviatūros varianto).
Vartotojo tyrimas patvirtino: Tauri issue [#11671](https://github.com/tauri-apps/tauri/issues/11671)
atviras nuo 2024-11 be sprendimo. Trys realūs variantai buvo apsvarstyti:

- **A (pasirinkta): atskiras vaiko procesas su winit.** Winit'o event loop duoda pilnus
  klaviatūros įvykius (su tikru laikymo/atleidimo būviu, ne vien diskretiems spartos
  klavišams). Kartu išsprendžia R4 (kiekvienas paleidimas = švarus procesas).
- **B (atmesta kaip pagrindinis kelias, liko dokumentuota alternatyva ADR-005):**
  `Channel<&[u8]>` → WebGL2 canvas viename Tauri lange. Išspręstų klaviatūrą „nemokamai" (JS
  keydown/keyup), bet apribotų platformas (N64/GameCube/PSP nesutalpina pralaidumo — žr.
  README) ir nespręstų R4.
- **C (atmesta):** platformai specifinis native kodas (`NSEvent.addLocalMonitorForEvents`
  macOS, `gtk_window().connect_key_press_event()` Linux). Veiktų, bet du atskiri `unsafe`
  keliai; Tauri v2 neatskleidžia `ns_window()` macOS'e (tik `gtk_window()` Linux'e) — tektų
  papildomai per `raw-window-handle` + `objc2`.

**Iš karto ATMESTOS (klaidingai pasiūlytos, tada pačios paneigtos) alternatyvos:**
`global-hotkey`/`tauri-plugin-global-shortcut`, `rdev`, `device_query`. Visos tai **sisteminio
lygio spartos klavišų** bibliotekos, ne lango klaviatūros įvestis: (1) registruoja klavišus
VISAI OS, net kai Nullbyte nefokusuotas — atimtų strėles/WASD iš kitų programų; (2) skirtos
diskretiems paspaudimams (`Cmd+N`), ne nuolatinei „laikoma/nelaikoma" būsenai, kurios reikia
žaidimui; (3) `rdev`/`device_query` reikalauja macOS Accessibility leidimo (sisteminis
dialogas). Tauri dokumentacija pati įspėja, kad tokie shortcuts „can be inherently dangerous".

**Sprendimas:** Emuliatoriaus langas + vykdymas persikelia į atskirą vaiko procesą
`nullbyte-emu` (winit + wgpu + cpal + gilrs — visi P2–P4.1 jau parašyti moduliai persikelia
BEVEIK NEPAKITĘ, keičiasi tik lango kūrėjas: `winit::window::Window` vietoj
`tauri::window::Window`). `nullbyte-app` (Tauri tėvas) paleidžia jį kaip `externalBin`
sidecar procesą kiekvienam žaidimo paleidimui.

**Keturi konkretūs sprendimai (patvirtinti su vartotoju):**
1. **macOS Dock:** winit numatytai naudoja `ActivationPolicy::Regular` (vaikas atsirastų
   Dock'e kaip antra programa). Naudojama
   `EventLoopBuilderExtMacOS::with_activation_policy(ActivationPolicy::Accessory)`.
2. **Našlaičių procesų apsauga:** jei tėvas krenta, Unix'e vaikas savaime NEMIRŠTA. NE PID
   pollinimas (nepatikimas, race'inamas) — **NE atskiras pipe'as** (pirminis planas), o tas
   pats IPC `stdin` kanalas (P4.0.3, `tauri-plugin-shell` sidecar transportas): vaikas jį
   skaito fone atskiroje gijoje; tėvo netikėtas baigimasis uždaro paskutinę `stdin` write-end
   nuorodą → vaikas gauna `EOF` → švariai išsijungia pats. Vienas kanalas dviem paskirtims
   — sprendimas priimtas P4.0.1 pradžioje, kai vartotojas pastebėjo, kad atskiras pipe'as
   priverstų naudoti žalią `std::process::Command` (kuris pats turėtų spręsti binaro kelią
   dev vs bundle), o `tauri-plugin-shell`'io sidecar API tą sprendžia automatiškai, bet
   atskleidžia tik stdin/stdout.
3. **Cargo workspace, trys crate'ai:** `nullbyte-core` (bendra: `core/`, `video/`, `audio/`,
   `input/` — naudoja IR vaikas vykdymui, IR tėvas IPC tipų bendrinimui),
   `nullbyte-app` (Tauri tėvas: `db/`, `scraper/`, `library/`, `commands/`),
   `nullbyte-emu` (vaiko binaras). `tauri.conf.json` `bundle.externalBin` supakuoja antrą
   binarą. Retroaktyviai keičia P0.3 (projekto struktūra) ir P0.4 (CI — reikės build'inti
   visus tris crate'us) prielaidas.
4. **P8.1 save state preview:** triple buffer'is dabar vaike, DB — tėve. Vaikas pats
   užkoduoja dabartinį kadrą į PNG, įrašo į diską, IPC grąžina TIK failo kelią — ne kadro
   baitus (žr. P8.1 pastabą).

**IPC riba lieka plona:** per ją keliauja TIK valdymo žinutės (dabartinis `EmuCommand` enum'as
— `Load`/`Pause`/`Resume`/`Stop`/`SaveState`/`SetFastForward`) ir būvio pranešimai atgal.
Nei vaizdas, nei garsas, nei (dabar) klaviatūra/gamepad'as NIEKADA nekerta proceso ribos —
`video::frame_buffer` triple buffer'is ir `audio::ring` SPSC ring buferis lieka algoritmiškai
NEPAKITĘ, veikia tarp dviejų gijų VIENAME (`nullbyte-emu`) procese, kaip veikė P2.2–P3.4 metu.

**Papildymas (P4.0.3, 2026-08-20):** kadangi IPC eina per `stdin`/`stdout` (o ne atskirą pipe'ą,
žr. punktą #2), `nullbyte-emu` niekada negali rašyti į `stdout` nieko, kas nėra protokolo
žinutė — nei `println!`, nei numatytojo `tracing_subscriber` writer'io (kuris pagal nutylėjimą
rašo į stdout). Logink tik į `stderr` arba failą. Žr. CLAUDE.md §10.

**Priežastis:** Sprendžia DVI problemas vienu architektūriniu žingsniu — klaviatūros
įvestį IR R4 (Aukšta tikimybė, iki šiol be sprendimo, MVP.md §13). Pigiau daryti Fazėje 4
(mažai kodo virš esamos architektūros) nei Fazėje 9 (viskas jau sujungta su biblioteka,
scraping'u, UI).

**Pasekmės:**
- R4 pažymėta išspręsta (žr. §13 rizikų registrą).
- ADR-005 papildyta pastaba (žr. aukščiau) — esminis sprendimas („native langas, ne WebView
  canvas") lieka galiojantis, keičiasi tik proceso priklausomybė.
- Naujas HW render (`RETRO_ENVIRONMENT_SET_HW_RENDER`, ID=14) apribojimas ATRASTAS (nesusijęs
  su šiuo ADR, bet patikrintas tuo pačiu metu) — README platformų lentelė perskaidyta, žr.
  §15 v0.2 sąrašą.
- P2.3/P4.3/P8.1 papildytos pastabomis apie architektūros pasikeitimą (žr. atitinkamas
  sekcijas).
- Realaus kodo migracija (workspace split, `nullbyte-emu` binaras, IPC sluoksnis) — DAR
  NEPADARYTA šios sesijos metu (šis ADR — dokumentacijos/sprendimo fiksavimas). Kitas
  žingsnis: implementuoti, tada grįžti prie P4.2 (mapping) ir P4.3 (polling) su nauja
  architektūra.

---

## 15. Po MVP — idėjų sąrašas

> **Nedaryk nieko iš šio sąrašo, kol MVP nebaigtas.**
> Naujos idėjos rašomos čia, ne į fazių planą.

**v0.2 — Gilesnis emuliavimas**
- **Hardware-rendered core'ų palaikymas** (`RETRO_ENVIRONMENT_SET_HW_RENDER`, ID=14 —
  patikrinta prieš tikrą `libretro.h`, 2026-08-20). Frontend'as turi suteikti GL/Vulkan
  kontekstą + framebuffer'į core'ui — dabar `environment.rs` grąžina `false` (nežinoma
  komanda), tad Nintendo 64 (Mupen64Plus-Next, ParaLLEl N64), GameCube/Wii (Dolphin) ir
  Sony PSP (PPSSPP) core'ai arba nepasileidžia, arba kris į nenaudojamą fallback'ą. README
  platformų lentelė atitinkamai perskaidyta į „veikia MVP" / „reikalauja HW render". Su P8.x
  proceso izoliacijos architektūra (ADR-016) šis darbas natūraliai priklausytų
  `nullbyte-emu` vaiko procesui (kuris jau turi savo wgpu `Device`/`Surface`, galėtų dalintis
  GL kontekstą core'ui per `get_proc_address`)
- Core options UI (per-core nustatymai su `SET_CORE_OPTIONS_V2`)
- Shader'iai: CRT-Royale, scanlines, xBRZ, LCD grid
- Rewind (žiedinis save state buferis)
- Netplay (libretro netplay protokolas)
- RetroAchievements (rcheevos integracija)
- Disk control (multi-disk PS1/PC Engine CD)
- Rumble / vibracija
- Cheat'ai (Game Genie / Action Replay kodai)
- Automatiniai atnaujinimai

**v0.3 — Ekosistema**
- Core downloader tiesiai iš buildbot.libretro.com
- Playlist'ai ir kolekcijos (rankinės ir dinaminės)
- Statistikos dashboard (žaidimo laikas, dažniausiai žaisti, heatmap)
- Šviesi tema
- Lokalizacija (LT, EN, DE, FR)
- Windows palaikymas
- Antrojo metaduomenų šaltinio fallback (TheGamesDB / IGDB)

**Tolimesnės idėjos**
- Steam Deck / handheld režimas (didesni elementai, gamepad navigacija)
- Debesų sinchronizacija save state'ams
- Screenshot galerija
- Žaidimo įrašymas į video
- Big picture / TV režimas
- CLI (`nullbyte play <rom>`)
