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
- [!] Veikia Linux X11 (Vulkan) — VIS DAR NEPATIKRINTA (žr. Wayland įrašą žemiau — turėta
      Linux sesija buvo Wayland, ne X11).
- [x] Veikia Linux Wayland — **PATIKRINTA REALIAI 2026-08-26** (žr. ADR-027, per NAUJĄ
      `nullbyte-emu` winit architektūrą, ne šio task'o originalią Tauri `Window`, žr. pastabą
      žemiau): Arch Linux (omarchy), tikra aktyvi Hyprland/Wayland sesija (SSH kaip tas pats
      vartotojas, `WAYLAND_DISPLAY`/`XDG_RUNTIME_DIR` iš `/run/user/<uid>`), realus GPU. Log:
      „wgpu adapteris pasirinktas" `adapter="AMD Radeon Graphics (RADV RENOIR)"
      backend=Vulkan`, „wgpu Surface sukonfigūruotas" `758×818 Bgra8UnormSrgb`. Vartotojas
      REALIAI matė žaidimą (ActRaiser, SNES) savo fiziniame ekrane, patvirtino: „taip, veikia".
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
- [x] Veikia macOS (CoreAudio) — patikrinta aukščiau. **Veikia Linux (ALSA/PipeWire) —
      PATIKRINTA REALIAI 2026-08-26**, bet TIK PO realaus bug'o pataisymo — žr. ADR-027.
      Pirmas bandymas Arch Linux (PipeWire virš ALSA) sudužo su
      `snd_pcm_hw_params_set_buffer_size ... Invalid argument` (`BufferSize::Fixed`
      atmesta), audio srautas neatsidarydavo, tad audio-driven pacing niekada nepajudėdavo
      (juodas langas, procesas gyvas). Pataisyta perjungus į `BufferSize::Default` (saugu —
      `audio::ring::recommended_capacity` NEPRIKLAUSO nuo OS lygio buferio dydžio, žr.
      ADR-027). Po pataisymo: log „cpal audio srautas paleistas" be klaidos, REALUS žaidimas
      (ActRaiser) veikė ~51 fps, `audio_occupancy≈0.62`, vartotojas girdėjo/matė veikiantį
      žaidimą su klaviatūros valdymu.

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

### P4.0.2 — `nullbyte-emu` binaro griaučiai (winit) `[x]`

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
- Resize/klaviatūra iš winit event loop'o (klaviatūra — NAUJIENA, anksčiau negalima)
- Gamepad: paleisk `GamepadThread` (P4.1, jau egzistuoja, NEKEISTI architektūros — žr. P4.1
  pastabą apie `gilrs-core` vidinę giją), neblokuojančiai (`try_recv()`) nuskaityk
  `about_to_wait()` cikle

**Įgyvendinta:** `App` implementuoja `winit::application::ApplicationHandler` — `resumed()` sukuria
langą (`Arc<Window>`, kad jo kopiją galėtų laikyti IR `Renderer` per `Arc::clone`, IR pats `App`),
`Renderer::new()` veikia BE pakeitimų (jau buvo generic per `HasWindowHandle`+`HasDisplayHandle`).
`EmuThread::spawn()` + `AudioOutput::open()` paleidžiami tiesiai `resumed()` viduje — **P4.0.2
architektūrinis supaprastinimas**: senasis `commands/emulator.rs::start_audio_pump` laikė
`cpal::Stream` dedikuotoje gijoje TIK todėl, kad Tauri managed state reikalavo `Send + Sync`, o
`cpal::Stream` (macOS CoreAudio) tokia nėra. `App` gyvena vien winit main gijoje ir niekada
nekerta gijos ribos, tad `AudioOutput` dabar tiesiog laukas struct'e — dedikuota „parkuojanti"
gija su `loop { sleep(3600s) }` nebereikalinga. Analogiškai frame pump: `about_to_wait()` seka
`FrameConsumer` ir kviečia `renderer.upload_frame`/`render()` TIESIOGIAI (jau main gijoje) — nereikia
senojo `run_on_main_thread` kanalo persiuntimo. Klaviatūra: `handle_keyboard()` atpažįsta
`PhysicalKey::Code(KeyCode::Arrow*)` ir logina (pilnas mapping'as — P4.2). Gamepad:
`resumed()` paleidžia `GamepadThread::spawn()` (P4.1 kodas, architektūra NEKEISTA — žr. P4.1
pastabą), `drain_gamepad_events()` kviečiamas iš `about_to_wait()` kiekvieną ciklą,
neblokuojančiai (`try_recv()`) nuskaito ir logina prisijungimą/atsijungimą bei mygtukų
paspaudimus (ašys — praleidžiamos, per triukšmingos; pilnas mapping'as — P4.2). Core/ROM
kelias hardkodintas per `test_core_and_rom()` (skenuoja `nullbyte-core/{cores,roms/snes}`, tas
pats fixture principas kaip `core::loader` testuose).

**Acceptance:**
- [x] Langas atsidaro, rodo SNES žaidimą (regresijos patikra prieš P2.4 rezultatą) — patikrinta
      realiai paleidus binarą (`cargo run --package nullbyte-emu`): wgpu Metal adapteris
      pasirinktas, Surface sukonfigūruotas (1600×1200 @ Retina 2x), `snes9x_libretro.dylib` +
      `Super Punch-Out!!.sfc` sėkmingai įkelti (`fps=50.006...`, PAL). Vizualiai pikselių
      teisingumas atskirai nefotografuotas — piešimo pipeline (P2.4) NEPAKEISTA.
- [x] Garsas groja be traškesių (regresijos patikra prieš P3.4 rezultatą) — mechaniškai
      patvirtinta `cargo test --package nullbyte-core --release -- --ignored --nocapture
      --test-threads=1` (visi 4, 157s, žr. git istoriją). Klausomas patvirtinimas —
      vartotojas PATS realiai paleido ir klausėsi kelių žaidimų (SNES *Super Punch-Out!!*,
      Genesis *Sonic 2*/*Aladdin*, įskaitant core keitimą gyvai) tos pačios sesijos metu ir
      patvirtino: „viskas labai gerai" (2026-08-20).
- [x] **Klaviatūra REALIAI valdo žaidimą** — patikrinta realiai: paleistas `nullbyte-emu`,
      `osascript`/System Events aktyvavo langą (`background only: true` patvirtina Accessory) ir
      nusiuntė Up strėlės klavišą; `stderr` log parodė
      `klaviatūros test mapping: strėlė paspausta button="UP"`. `winit::event::WindowEvent::
      KeyboardInput` tikrai ateina — buvo neįmanoma prieš ADR-016.
- [x] macOS Dock nerodo antros programos (`ActivationPolicy::Accessory` patikrinta) — patikrinta
      realiai: `System Events` grąžino `background only: true` veikiančiam procesui.
- [x] **Gamepad mygtuko paspaudimas duoda log eilutę** (simetriška klaviatūros kriterijui) —
      patikrinta REALIAI su fiziniu DualShock 4 (2026-08-21): `nullbyte-emu` jau veikė,
      controller'is prijungtas GYVAI (`gamepad prijungtas name="PS4 Controller"` — tai
      TAIP PAT patvirtina hot-plug, žr. žemiau), paspausti X/Kvadratas/Trikampis/Nulis —
      visi keturi teisingai atpažinti (`button=South/West/North/East`, standartinis gilrs
      face-button mapping'as).

---

### P4.0.3 — IPC protokolas (`nullbyte-app` ↔ `nullbyte-emu`) `[x]`

**Priklausomybės:** P4.0.1
**Failai:** `crates/nullbyte-core/src/ipc.rs` (bendras protokolo tipas), `crates/nullbyte-emu/src/ipc.rs`
(serveris), `crates/nullbyte-app/src/ipc.rs` (klientas)

> **Priešdarbis atliktas PRIEŠ IPC kodą (2026-08-20)** — sidecar binaro vištos-kiaušinio
> problema: `tauri-plugin-shell` sidecar (žemiau, „Transportas") reikalauja
> `crates/nullbyte-app/binaries/nullbyte-emu-<target-triple>` egzistuojant JAU
> `nullbyte-app`'o build.rs paleidimo metu — patikrinta tiesiogiai `tauri-build` 2.6.3
> šaltinyje (`copy_binaries()`/`copy_file()`): jei failo nėra, VISAS `nullbyte-app` build'as
> žlunga (`std::process::exit(1)`), ne tik runtime sidecar spawn. Kadangi `nullbyte-app`
> Cargo priklausomybių grafe NEPRIKLAUSO nuo `nullbyte-emu` (sidecar'as reikalingas runtime,
> ne kompiliavimo metu), Cargo pats negarantuoja teisingos statymo tvarkos — `cargo build
> --workspace` gali statyti bet kokia tvarka, taigi ir lūžti nenuspėjamai priklausomai nuo
> Cargo scheduler'io.
>
> Sprendimas — TRYS nepriklausomi automatiniai keliai, kad niekur nereikėtų RANKINIO
> „prisimink paleisti X" žingsnio:
> 1. `scripts/build-sidecar.sh` (`rustc --print host-tuple` nustato triple, `cargo build -p
>    nullbyte-emu`, kopijuoja į `crates/nullbyte-app/binaries/nullbyte-emu-<triple>`) per
>    `pnpm run build:sidecar[:release]`.
> 2. `tauri.conf.json` `beforeDevCommand`/`beforeBuildCommand` grandina
>    `pnpm run build:sidecar && pnpm dev` (atitinkamai `build`) — PATIKRINTA per tikrą
>    `tauri-cli` šaltinį (`crates/tauri-cli/src/dev.rs`), kad `devUrl` polling (NE `wait`
>    reikšmė) yra tikrasis vartų mechanizmas prieš `cargo run` paleidimą, tad grandininė
>    komanda saugi be race'o.
> 3. `.github/workflows/ci.yml` — atskiras eksplicitinis žingsnis `pnpm run build:sidecar`
>    PRIEŠ `cargo fmt/clippy/test`. **Pastaba:** šis CI failas buvo PASENĘS nuo P4.0.1
>    (vis dar nurodė `src-tauri`, kuris nebeegzistuoja) — pataisyta tuo pačiu metu (`crates/*`
>    workspace struktūra, `--workspace` flag'ai visur).
>
> Papildomai `crates/nullbyte-app/build.rs` PATIKRINA failo buvimą PRIEŠ kviečiant
> `tauri_build::build()` ir duoda aiškų, veiksmingą pranešimą (nurodo `pnpm run
> build:sidecar`) vietoj tauri-build vidinio — gaudo LIKUSĮ atvejį (žalias `cargo
> build/test/clippy --workspace` be išankstinio sidecar build'o, pvz. pirmas lokalus setup'as
> be `pnpm tauri dev`).
>
> **Patikrinta realiai, ne vien skaitant kodą:** `rm -rf crates/nullbyte-app/binaries` +
> `cargo clean` (6.5 GiB) → `pnpm run build:sidecar` → `cargo fmt --all --check` → `cargo
> clippy --workspace --all-targets -- -D warnings` → `cargo test --workspace` — visa seka
> praėjo be klaidų nuo absoliučiai švarios būsenos. **[!] Linux CI runner'io winit
> priklausomybės (X11/Wayland dev headers) NEPATIKRINTOS** — esamas „Install Linux system
> dependencies" žingsnis turi `libasound2-dev`/`libudev-dev` (audio/gamepad), bet ar
> pakanka winit X11/Wayland compile-time linkinimui (galimai transityviai per
> `libwebkit2gtk-4.1-dev`/`libgtk-3-dev`) — nežinoma be realaus Linux CI paleidimo.
> Multi-arch/universal build (P4.0.5) — NEAPIMTA, atskiras darbas.

> **Klaidų sklaidos per proceso ribą sprendimas, PRIEŠ rašant `EmuStatus` (2026-08-20)** —
> jei `EmuStatus::Error` neštų klaidą kaip suplokštintą `{kind, message}` eilutę, P4.0.1 metu
> pridėti konkretūs `CoreError` variantai (CoreLoad/ApiVersion/RomLoad/MissingBios/
> UnsupportedPixelFormat, žr. tos fazės pastabą) taptų beverčiai TIESIOG ties šia IPC riba —
> tėvas gautų tik eilutę, o P9.1/P9.3 reikalaujamas UI šakojimasis pagal konkretų klaidos tipą
> būtų neįmanomas be string parsinimo (blogas sprendimas). Pasirinkta: `EmuStatus::Error`
> neša `CoreError` STRUKTŪRIŠKAI (žr. `crate::ipc::EmuStatus`).
>
> Kliūtis: senasis `CoreError` turėjo RANKINĮ `impl Serialize` (suplokštino į `{kind,
> message}` UI kontraktui) — tai UŽĖMĖ derive vietą, tad negalima buvo tiesiog pridėti
> `Deserialize`. Sprendimas: `CoreError` dabar `#[derive(Serialize, Deserialize)]` (pilna,
> apverčiama struktūra), o `{kind, message}` suplokštinimas PERKELTAS į
> `nullbyte-app::error::AppError` serializerį — vienintelę vietą, kur jo iš tikrųjų reikia
> (Tauri → frontend riba, žr. tos pastabos atnaujinimą `AppError::kind()`). Vienas likęs
> laukas reikalavo specialaus sprendimo: `CoreError::Io` nešė TIKRĄ `std::io::Error`
> (`#[from]`, kad `?` veiktų visuose esamuose `loader.rs`/`archive.rs` call site'uose) — o
> `std::io::Error` PATI neturi `serde` impl'ų. Sprendimas — `#[serde(with = "io_error_wire")]`
> shim TIK tam vienam laukui (round-trip'ina tik pranešimo tekstą per `io::Error::other()`,
> lauko TIPAS lieka `std::io::Error`, `#[from]` nepaliestas). Patikrinta 4 round-trip testais
> `error.rs` (`cargo test --package nullbyte-core error::`).
>
> **Protokolo versijos handshake** — pati pirma IPC eilutė ABIEM kryptimis yra [`IpcHello`]
> (`crate::ipc`), NE `EmuCommand`/`EmuStatus` variantas (sąmoningai atskirtas protokolo
> lygmuo nuo žaidimo valdymo/būvio lygmens). Apsauga nuo pasenusio sidecar binaro — build
> grandinė (žr. aukščiau) jį paprastai perstato, bet rizika nenulinė (pvz. rankiniu būdu
> paleistas senas `target/debug/nullbyte-emu`); be handshake'o toks neatitikimas atrodytų kaip
> nesuprantama NDJSON parse klaida giliai protokolo viduryje.
>
> **Padaryta šioje sesijoje** (tipai + rašymo pusė, DAR NE skaitymo loop'as):
> `crates/nullbyte-core/src/ipc.rs` (`IPC_PROTOCOL_VERSION`, `IpcHello`, `EmuStatus`),
> `EmuCommand`/`InputState` (`core::runner`) ir `LoadedGameInfo` (`core::loader`) gavo
> `Serialize`/`Deserialize` + `#[serde(rename_all = "camelCase")]` (CLAUDE.md §7.3). Visi
> tipai kompiliuojasi, 3 nauji round-trip testai `ipc.rs` (nullbyte-core) praeina.
>
> **Backpressure — PRIEŠ rašant rašymo loop'ą (2026-08-20)** — OS pipe tarp vaiko `stdout`
> ir tėvo turi RIBOTĄ buferį (macOS ~64 KB). Jei tėvas laikinai nustoja drenuoti (UI
> užimtas, `CommandEvent` receiver'is nepollinamas), `write()` į pipe BLOKUOJA. Jei tas
> `write()` vyktų tiesiogiai emuliavimo gijoje ar winit main gijoje, emuliatorius sustotų —
> audio underrun'ai, kritę kadrai, simptomas „retkarčiais traška, bet negaliu pakartoti".
> Sprendimas — `crates/nullbyte-emu/src/ipc.rs`: `StatusWriter` (dedikuota gija, VIENINTELĖ,
> kuri liečia stdout) + `StatusSender` rankena (`Clone`, gaunama emu gijos ir winit main
> gijos), maitinama RIBOTU (`mpsc::sync_channel`, talpa 32) kanalu. Du siuntimo keliai:
> `send_important()` (Loaded/Error/Stopped — blokuojantis `send`, NIEKADA nemeta — praktiškai
> saugu, nes šie įvykiai reti) ir `send_stats()` (Stats — neblokuojantis `try_send`, TYLIAI
> numeta, jei kanalas pilnas, PLIUS throttle 300ms/~3.3Hz viduje, kad rodmuo, į kurį
> dažniausiai niekas nežiūri, nesiųstų 60 eilučių/s). Patikrinta 3 testais su kontroliuojamu
> „nedrenuojančiu" fake writer'iu (`GatedWriter`), imituojančiu pilną OS pipe — patvirtina,
> kad Stats numetami esant backpressure'ui, o Stopped VISADA pasiekia writer'į nepriklausomai
> nuo to. **Radinys testų metu:** pirmoji testo versija turėjo lenktynių sąlygą (griežta
> `assert!(sent <= CAPACITY)` neatsižvelgė, kad writer gija gali suspėti nuskaityti (bet
> ne parašyti, nes blokuoja ties pirmu `write()`) vieną eilutę dar besipildant kanalui) —
> pasireiškė kaip nepakartojamas testo pakibimas (~60s+ be jokios išvesties). Ištaisyta
> (silpnesnė, teisinga riba); patikrinta 18 pakartojimų iš eilės be klaidų.
>
> **NELIEKA** — `crates/nullbyte-emu/src/ipc.rs` stdin skaitymo pusė (`EmuCommand`
> parsinimas) IR `StatusSender`/`StatusWriter` prijungimas prie `EmuThread`/winit `App`
> (šiuo metu `#[allow(dead_code)]`, niekas dar nekonstruoja), IR
> `crates/nullbyte-app/src/ipc.rs` (klientas, `CommandChild` rašymas/skaitymas, PRIVALO
> drenuoti `CommandEvent` receiver'į VISADA, net kai UI nieko nedaro — žr. „Ką daryti" žemiau).

**Ką daryti:**
- ~~`EmuCommand` (jau egzistuoja `core::runner`) gauna `serde::Serialize`/`Deserialize`~~ —
  PADARYTA (žr. pastabą aukščiau). Liko: `EmuStatus` naudoja `IpcHello` handshake'ą ir
  neša `CoreError` struktūriškai (irgi PADARYTA, tipas paruoštas)
- ~~Rašymo pusės backpressure (`StatusWriter`/`StatusSender`, bounded kanalas, Stats
  throttle+drop, Loaded/Error/Stopped garantuotas pristatymas)~~ — PADARYTA (žr. pastabą
  aukščiau). Liko PRIJUNGTI prie `EmuThread`/winit `App` — kol kas `#[allow(dead_code)]`
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
- ~~`nullbyte-emu`: fono gija skaito savo `stdin` per `BufRead::lines()`, parsina kiekvieną
  eilutę kaip `EmuCommand`, siunčia į `EmuThread`; rašo `EmuStatus` į `stdout`~~ — PADARYTA
- ~~`nullbyte-app`: paleidžia sidecar'ą, siunčia komandas per `CommandChild::write()`, skaito
  `CommandEvent::Stdout` žinutes iš `Receiver`~~ — PADARYTA (`EmuClient`, žr. pastabą aukščiau)
- **KRITIŠKAI SVARBU:** kadangi IPC eina per `stdout`, `nullbyte-emu` NIEKADA negali rašyti į
  `stdout` nieko kito — nei `println!`/`dbg!`, nei numatytojo `tracing_subscriber` writer'io
  (jis pagal nutylėjimą rašo į stdout!). Vienas pamirštas `println!` arba nenukreiptas
  `tracing` subscriber sugadins protokolą, ir klaida atrodys kaip atsitiktinis JSON parse error,
  ne kaip logging problema. `nullbyte-emu` logina **tik į `stderr`** (`.with_writer(std::io::stderr)`)
  arba į failą — niekada į stdout. Žr. CLAUDE.md §10

> **Vaiko pusė padaryta (2026-08-20)** — `StatusWriter`/`StatusSender` (backpressure, žr.
> ankstesnę pastabą) persikėlė iš `nullbyte-emu` į `nullbyte_core::ipc`, nes `core::runner::
> EmuThread` turi juos naudoti TIESIOGIAI emuliavimo gijoje (`handle_load` siunčia `Loaded`/
> `Error`, `run_loop` siunčia `Stats` kas kadrą — throttle'as viduje tai paverčia į ~3 Hz — ir
> `Stopped` prieš grįžtant); jei jie liktų `nullbyte-emu`, priklausomybių kryptis būtų
> atvirkščia. `EmuThread::spawn()` gavo naują `Option<StatusSender>` parametrą (4 esami testai
> atnaujinti su `None`) ir naują `command_sender()` metodą (klonuota `Sender<EmuCommand>` —
> leidžia stdin skaitymo gijai siųsti komandas be `&'static EmuThread` nuorodos gyvavimo
> trukmės problemos). `crates/nullbyte-emu/src/ipc.rs` dabar turi `run_command_reader()` —
> validuoja tėvo `IpcHello` PRIEŠ apdorodama bet kokią `EmuCommand` eilutę, blogas JSON
> praleidžiamas (NE fatal), stdin EOF grąžina švariai. 4 nauji testai (roundtrip, bloga
> eilutė nesulaužo, trūkstamas/nesuderinamas Hello sustabdo PRIEŠ komandas).
>
> **Radinys testų metu:** `StatusWriter::spawn()` iš pradžių naudojo `writeln!` Hello eilutei
> — tai gali sukelti DU atskirus `write_all()` kvietimus (turinys + `"\n"` atskirai,
> priklausomai nuo `fmt::Arguments` fragmentacijos), o testai su ribotos talpos writer'iu
> davė tik VIENĄ leidimą prieš `spawn()` kvietimą. Pasireiškė kaip pakibimas (be jokios
> išvesties, ~60s+) TIK bandant visą test suite kartu — kiekvienas testas atskirai praėjo.
> Ištaisyta (vienas rankiniu būdu sukonstruotas `write_all()` kvietimas, mirroring
> `run_writer_loop`), patikrinta 15 pakartojimų iš eilės.
>
> **Patikrinta REALIAI** (ne vien testais) — paleistas tikras `nullbyte-emu` binaras su named
> pipe (FIFO) stdin: `IpcHello` atėjo kaip pirma `stdout` eilutė, `EmuCommand::Stop` per stdin
> teisingai apdorotas (`{"stop":null}` — serde priima ir šią formą unit variantui, ne tik
> bareword `"stop"`), `EmuStatus::Loaded` atėjo automatiškai po P4.0.2 test hook'o įkėlimo,
> `EmuStatus::Stopped` atėjo po `Stop`, stdin EOF sustabdė skaitymo giją švariai, jokio
> proceso pakibimo/zombie. `stdout` turėjo LYGIAI 3 tvarkingas NDJSON eilutes — nė vieno
> pašalinio baito.
>
> **NELIEKA** — `crates/nullbyte-app/src/ipc.rs` (klientas, `CommandChild` per
> `tauri-plugin-shell`, `CommandEvent` receiver'io drenavimas VISADA) ir P4.0.4 shutdown
> orchestracija (EOF → `process::exit`).

> **Peržiūros radinys, ištaisyta (2026-08-20)** — `Stopped` teardown metu buvo siunčiamas per
> `send_important()` (blokuojantis `send()`). P4.0.4 scenarijuje (`kill -9` tėvui arba bet
> koks kitas atvejis, kai stdout nustoja būti drenuojamas TEARDOWN metu) tai reikštų, kad
> VAIKAS pats pakimba užpildytame kanale vietoj to, kad švariai išeitų — priešingai P4.0.4
> tikslui. Sprendimas: naujas `StatusSender::send_best_effort()` (`try_send`, niekada
> neblokuoja) TIK šiam vienam teardown call site'ui (`run_loop` pabaigoje). `Loaded`/`Error`
> LIEKA per blokuojantį `send_important()` — jie siunčiami normalaus veikimo metu, kai tėvas
> aktyviai drenuoja, tad blokavimo rizika ten realiai nekyla. Patikrinta nauju testu
> (`stopped_via_best_effort_never_blocks_even_when_writer_never_drains` — kanalas užpildytas
> iki talpos, RX niekada nedrenuojamas, kvietimas vis tiek grąžina valdymą iškart).
>
> **`Stats` throttle patikrintas REALIAI, ne vien testais** — paleidus `nullbyte-emu` ir
> palaikius `Run` būvį ~5s prieš siunčiant `Stop`: **17 `Stats` eilučių** (≈3.4 Hz, atitinka
> 300ms throttle) + 1 `Loaded` + 1 `Stopped` + 1 `IpcHello` = 20 eilučių iš viso. Ankstesnis
> paleidimas buvo per trumpas throttle intervalui parodyti (0 `Stats` eilučių) — dabar
> patvirtinta, kad `send_stats()` kelias realiai veikia, ne vien unit testuose.

> **Tėvo pusės klientas padarytas (2026-08-21)** — `crates/nullbyte-app/src/ipc.rs`:
> `EmuClient::spawn()` paleidžia `nullbyte-emu` per `app.shell().sidecar("nullbyte-emu")`
> (`tauri-plugin-shell`), siunčia SAVO `IpcHello`, laukia VAIKO `IpcHello` kaip pirmos
> `CommandEvent::Stdout` eilutės, ir TIK PO sėkmingo handshake'o paleidžia
> `tauri::async_runtime::spawn`'intą foninę užduotį. `send()` rašo `EmuCommand` per
> `CommandChild::write()`. `kill()` — vienintelis būdas nutraukti PATĮ procesą (`Stop`
> baigia tik žaidimą, ne procesą — vartotojas gali įkelti kitą; švarus viso proceso
> išjungimas per stdin EOF — P4.0.4, dar neįgyvendinta, ir kadangi `CommandChild` NETURI
> `Drop`, kuris nutrauktų vaiką, iki P4.0.4 caller'is PATS atsakingas kviesti `kill()`).
>
> **Backpressure radinys prieš rašant** — `tauri-plugin-shell` PATS (2.3.5 šaltinis,
> `Command::spawn()`) turi vidinį `CommandEvent` kanalą su talpa **1**. Jei `EmuClient`
> naudotojas nedelsdamas nedrenuoja `Receiver`, ne tik vaiko `StatusWriter` (žr. ankstesnę
> pastabą), bet PATI `tauri-plugin-shell` stdout skaitymo užduotis užsiblokuoja — backpressure
> grandinė gali prasidėti bet kada, kai UI „nieko nedaro", ne tik `kill -9` scenarijuje. Todėl
> drenavimo užduotis paleidžiama BE SĄLYGŲ iškart po handshake'o, ne laukiant UI veiksmo.
>
> **Patikrinta REALIAI DVIEM būdais:** (1) laikinai prijungus `EmuClient::spawn()` prie
> `lib.rs` `setup()` hook'o ir paleidus tikrą `nullbyte-app` binarą tiesiogiai (be `pnpm tauri
> dev`) — pilnas dvikryptis ciklas per TIKRĄ sidecar transportą: handshake OK →
> `EmuStatus::Loaded` gautas atgal (P4.0.2 test hook) → `Stop` nusiųstas → `EmuStatus::Stopped`
> gautas atgal. (2) Tas pats scenarijus paverstas automatizuotu `#[ignore]`'intu testu
> (`tauri::test::mock_builder()` + `tauri_plugin_shell::init()`), patikrintu 3 kartus iš eilės
> — praeina IR nepalieka orphan `nullbyte-emu` proceso (dėka naujo `kill()`). `#[ignore]`, nes
> `nullbyte-emu` sukuria TIKRĄ winit langą + wgpu + cpal — headless CI (ypač `ubuntu-latest`,
> be X11/Wayland) tai gali nepavykti nenuspėjamai, ta pati priežastis kaip CLAUDE.md §10
> P2.3/P2.5/P3.1 Linux apribojimai.
>
> **Šalutinis radinys, ištaisyta prieš rašant klientą** — `commands/emulator.rs` (P2.3 era) IR
> `AppState.renderer`/`emu_thread` vis dar tarši PRIEŠ-ADR-016 architektūrą (lokalus
> `Renderer`/`EmuThread` `nullbyte-app` procese) — niekada nebuvo sutvarkyta per P4.0.x
> pivotą, kompiliavosi tik todėl, kad niekas realiai nebekvietė `EmuThread::spawn()`/
> `Renderer::new()`. Pašalinta ATSKIRU commit'u prieš `EmuClient` (žr. git istoriją) — kartu
> pašalintas ir dabar nereikalingas `tauri` `"unstable"` feature'is (jo vienintelis
> pateisinimas buvo P2.3 windowless `WindowBuilder`, kurio nebeliko).

**Acceptance:**
- [x] `Load`/`Stop` komandos pasiekia vaiką ir sukelia teisingą elgesį (`Pause`/`Resume`
      analogiškai — tas pats kodo kelias). Patikrinta REALIAI per FIFO KELIS kartus: `Stop`,
      IR `Load` (du kartus — SNES→Genesis core keitimas per tikrą IPC `Load`, ne vien
      P4.0.2 test hook'ą, patvirtina CLAUDE.md §3.2 taisyklę #2 realiu core swap'u).
- [x] Būvio pranešimai (klaidos, statistika) pasiekia tėvą — `Loaded`/`Stopped`/`Stats`
      VISI patikrinti REALIAI (žr. pastabas aukščiau, `Stats` — 17 eilučių per ~5s realiame
      paleidime). `Error` patikrintas TIK testu (nebuvo natūralaus scenarijaus jam sukelti
      realiame paleidime šios sesijos metu — kodo kelias identiškas `Loaded`, bet
      nepatikrintas akimis).
- [x] Serializacijos klaida NESULAUŽO nei vieno proceso — `bad_command_line_is_skipped_not_fatal`
      testas + writer pusės `run_writer_loop` `Err` šaka abi patikrintos
- [x] Teardown (`Stopped`) NEBLOKUOJA vaiko net kai stdout nebedrenuojamas — naujas
      `send_best_effort()` + testas (žr. pastabą aukščiau)
- [x] `nullbyte-emu` paleidus be jokio ROM'o (vien init) — `stdout` NEturi nė vienos baitos,
      kuri nėra validus NDJSON `EmuStatus`/`IpcHello`. Patikrinta REALIAI: laikinai paslėptas
      `crates/nullbyte-core/cores/`, paleistas binaras — `stderr` parodė „nerasta test
      fixture", `stdout` turėjo LYGIAI vieną eilutę (`IpcHello`), nieko daugiau.

---

### P4.0.4 — Proceso gyvavimo ciklas, našlaičių apsauga `[x]`

**Priklausomybės:** P4.0.2, P4.0.3
**Failai:** `crates/nullbyte-emu/src/main.rs` (naujas `EmuUserEvent::StdinClosed` + winit
`EventLoopProxy`, kad stdin-skaitymo gijos `EOF` pasiektų main/winit giją — `ActiveEventLoop`
neturi `create_proxy()`, tad `EventLoop<T>::with_user_event()` reikalingas nuo pat pradžių),
`crates/nullbyte-app/src/ipc.rs` (`EmuClient::shutdown_gracefully()` + `pid()`).
`commands/emulator.rs` DAR NEEGZISTUOJA (P9.1 UI srautas nepradėtas) — `shutdown_gracefully()`
yra primityvas, kurį tas sluoksnis kvies vėliau, ne pilnas Tauri komandų API.

> **Įgyvendinta 2026-08-25.** Realiai patikrinta (ne vien skaitant kodą, žr. acceptance):
> `kill -9` ant „tėvo" (bash simuliuotas) → vaikas savaime išsijungė per 0.2s; realus e2e
> testas (`cargo test -p nullbyte-app shutdown_gracefully_lets_child_exit -- --ignored
> --nocapture`) patvirtino, kad `shutdown_gracefully()` baigia TIKRĄ OS procesą (`kill -0
> <pid>` po `EmuClient` numetimo), ne tik `EmuClient` rankeną.
>
> **Sprendimas skiriasi nuo žemiau aprašyto „Ką daryti" bullet'o 3** (parduota paprasčiau):
> vietoj „siųsk `Stop` → lauk `Terminated` su timeout'u → `kill()`", `EmuClient::
> shutdown_gracefully()` tiesiog numeta `CommandChild` — jo `stdin_writer` lauko `Drop`
> uždaro OS pipe write-end'ą, o TAI IR YRA lygiai tas pats mechanizmas, kurį vaikas jau
> naudoja `kill -9` atveju (žr. bullet'ą 1–2 žemiau) — nereikia atskiro `Stop`+timeout+`kill`
> kelio tam pačiam tikslui pasiekti. `kill()` (hard kill) lieka kaip atskiras metodas
> kraštutiniam atvejui.

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
- [x] Dirbtinai nutraukus tėvo procesą (`kill -9`), vaikas savaime išsijungia per kelias
      sekundes vien dėl `stdin` EOF (patikrinta realiai, ne vien skaitant kodą — bash
      simuliuotas „tėvas" laikė FIFO atidarytą, `kill -9` jam → vaikas mirė per 0.2s)
- [x] Normalus žaidimo uždarymas švariai sustabdo vaiką be „zombie" proceso (realus e2e
      testas `shutdown_gracefully_lets_child_exit_within_a_few_seconds`, `nullbyte-app/src/
      ipc.rs` — `kill -0 <pid>` patvirtina TIKRĄ OS proceso pabaigą)
- [x] Vaiko crash'as (dirbtinis `kill -9` vaikui, simuliuojant crash'ą) nenumuša tėvo proceso
      (realus e2e testas `parent_survives_child_crash`, `nullbyte-app/src/ipc.rs`)

---

### P4.0.5 — `externalBin` packaging `[x]`

**Priklausomybės:** P4.0.2

> **Dalis atlikta anksčiau, P4.0.3 priešdarbio metu (2026-08-20)** — žr. P4.0.3 pastabą.
> `bundle.externalBin: ["binaries/nullbyte-emu"]` jau `tauri.conf.json`, o `scripts/
> build-sidecar.sh` (per `pnpm run build:sidecar[:release]`, sujungtas į
> `beforeDevCommand`/`beforeBuildCommand`) jau automatiškai stato + pervadina binarą su
> DABARTINIO HOSTO target-triple sufiksu. Tai reikėjo padaryti anksčiau, ne čia — priešingu
> atveju `pnpm tauri dev` (P4.0.2/P4.0.3) ir net paprastas `cargo build --workspace` būtų
> lūžę dar prieš pasiekiant šią užduotį (vištos-kiaušinio problema, žr. P4.0.3).

> **Baigta 2026-08-25.** Realiai patikrinta (ne vien skaitant kodą): `pnpm tauri build
> --target universal-apple-darwin` sukūrė TIEK `.app`, TIEK `.dmg` (`target/
> universal-apple-darwin/release/bundle/{macos/Nullbyte.app,dmg/Nullbyte_0.1.0_universal.dmg}`);
> laikinu `.setup()` kabliu `lib.rs` (grąžintas po testo) paleidus `.app` iš tikro Finder/
> `open` konteksto, `nullbyte-app` sėkmingai spawn'ino `Contents/MacOS/nullbyte-emu`
> (universal binaras, `lipo -info` patvirtina abi architektūras), atliko IPC handshake'ą,
> gavo `Loaded` → `Stopped` (`shutdown_gracefully()` per `EmuClient` Drop — netyčia papildomai
> patvirtino ir P4.0.4 veikimą bundle kontekste), procesas baigėsi `code=Some(0)`.
>
> **Ankstesnė šio failo prielaida ŽEMIAU (bullet'as „Ką daryti") BUVO KLAIDINGA** — pati
> pirma bandymo iteracija tai atskleidė realiu `tauri build --target universal-apple-darwin`
> paleidimu: `failed to bundle project: Failed to copy external binaries: resource path
> binaries/nullbyte-emu-universal-apple-darwin doesn't exist`. Tauri universal build'ui
> ieško VIENO, JAU `lipo`'into binaro su `-universal-apple-darwin` sufiksu, NE dviejų
> atskirų per-triple failų (tie du vis tiek reikalingi ATSKIRAI — juos naudoja PATYS
> `cargo build --target <triple>` kvietimai universal `.app` sudarymo metu, tiesiog GALUTINIS
> sidecar copy žingsnis nori trečio, jau sulieto failo). `scripts/build-sidecar.sh` dabar
> macOS `release` profiliui stato VISUS TRIS: `nullbyte-emu-aarch64-apple-darwin`,
> `nullbyte-emu-x86_64-apple-darwin`, IR `nullbyte-emu-universal-apple-darwin` (`lipo
> -create`).
>
> **Pastebėta, bet NEliečiama (ne šios užduoties apimtis):** `tauri build` įspėja, kad
> `"identifier": "fr.nullbyte.app"` (`tauri.conf.json`) baigiasi `.app` — tai konfliktuoja
> su macOS bundle plėtiniu, CLI nerekomenduoja. Nekeista, nes identifikatoriaus pakeitimas
> paveiktų code signing/notarization/app data izoliaciją vėliau — vartotojo sprendimas, ne
> šios užduoties dalis.

**Failai:** `scripts/build-sidecar.sh` (universal build atveju — praplėsti VISAIS TRIMIS
triple'ais, žr. pastabą aukščiau)

**Ką daryti:**
- ~~Universal build'ui: praplėsk `scripts/build-sidecar.sh`, kad statytų IR
  `x86_64-apple-darwin`, IR `aarch64-apple-darwin` (du atskiri `cargo build --target ...`
  kvietimai, du sufiksuoti binarai `crates/nullbyte-app/binaries/`) — Tauri universal build
  pats `lipo`'ina GALUTINĮ `.app` binarą, bet KIEKVIENAS externalBin sidecar'as turi būti
  paduotas atskirai per triple, ne kaip vienas universal failas~~ **NETEISINGA prielaida,
  žr. pastabą aukščiau** — reikia dar ir trečio, jau `lipo`'into `-universal-apple-darwin`
  failo.

**Acceptance:**
- [x] `pnpm tauri build` (vienam hostui, `aarch64-apple-darwin`) sėkmingai randa sidecar'ą
      build.rs metu — realiai patikrinta `pnpm tauri build --target universal-apple-darwin`
      paleidimu (2026-08-25, release profilis, tikras `.app`/`.dmg` bundle'as)
- [x] `pnpm tauri build --target universal-apple-darwin` sėkmingai supakuoja abu triple'us
- [x] Supakuotas `.app`/`.dmg` paleidžia `nullbyte-emu` teisingai (iš bundle'o kelio, ne dev)

---

### P4.1 — Gamepad aptikimas `[!]` (DualShock 4 + Xbox + hot-plug patikrinti realiai; 8BitDo/Linux — ne)

**Priklausomybės:** P0.3 (originaliai), P4.0.1 (kodo perkėlimui į naują crate — žr. pastabą aukščiau)
**Failai:** `crates/nullbyte-core/src/input/gamepad.rs`

**Ką daryti:**
- `gilrs::Gilrs` event pump dedikuotoje gijoje su kanalu (žr. pastabą žemiau — NE „emu gijoje
  arba" pasirinkimas, tai buvo neišspręsta ambicija, dabar uždaryta faktais)
- Prijungimo/atjungimo įvykiai → pranešk UI per Tauri event
- Analoginių ašių deadzone (numatytoji 0.2)

> **Placement patikrintas šaltinio kodu (2026-08-20, prieš P4.0.2 gamepad wiring'ą):**
> `gilrs-core 0.6.8` macOS backend'as (`src/platform/macos/gamepad.rs::Gilrs::new()`) PATS
> viduje `thread::Builder::new().spawn(...)` sukuria savo `"gilrs"` giją, kuri susikuria
> `CFRunLoop::current()`, `schedule_with_run_loop` + `CFRunLoop::run()` — VISA IOKit HID
> callback'ų pristatymo mašinerija gyvena TOJE gijoje, ne kviečiančiojoje. `next_event`/
> `next_event_blocking` vieša API tik skaito iš `mpsc::Receiver`, į kurį ta vidinė gija rašo.
> Išvada: kviečiančiosios pusės gija (`GamepadThread`'o dedikuota `nullbyte-gamepad`, ar bet
> kuri kita, įskaitant winit main giją) NETURI JOKIOS ĮTAKOS HID įvykių pristatymui — nėra
> jokio „reikia aktyvaus run loop'o kviečiančiojoje pusėje" reikalavimo. `GamepadThread`
> dedikuotos gijos architektūra (žemiau) todėl PALIEKAMA nepakeista — jokio persirašymo ant
> winit `about_to_wait()` nereikėjo.

**Įgyvendinta:** `GamepadThread` — dedikuota gija (kaip `EmuThread`/`start_audio_pump`, nes
`gilrs::Gilrs` nėra `Sync`), `next_event_blocking` event pump su 100ms timeout (leidžia
švariai sustoti), `GamepadEvent` kanalu siunčiamas kviečiančiajai pusei. Deadzone (0.2)
taikomas RANKINIU BŪDU (gilrs įmontuoti filtrai išjungti — jie naudoja kiekvieno valdiklio
DB deadzone, ne fiksuotą 0.2 iš MVP.md), tolydžiu remap'u (ne atkirpimu), patikrinta 3
testais (nulinimas, tolydumas ribose, monotoniškumas). **P4.0.2 metu prijungta prie
`nullbyte-emu`:** `resumed()` paleidžia `GamepadThread::spawn()`, `about_to_wait()`
neblokuojančiai (`try_recv()`) nuskaito `GamepadEvent`'us ir loginą prisijungimą/atsijungimą
bei mygtukų paspaudimus (ašių pokyčiai — per triukšmingi, praleidžiami; pilnas mapping'as —
P4.2). Senasis `nullbyte-app`'o `commands::input::start_gamepad_pump` (Tauri
`"gamepad-connection"` event'as UI prisijungimo būviui) LIEKA kaip atskira, `#[allow(dead_code)]`
infrastruktūra nustatymų ekranui — tai UI pranešimo kelias, ne žaidimo valdymo kelias, ir
neprieštarauja `nullbyte-emu` pusės wiring'ui.

**Acceptance:**
- [!] Aptinka Xbox, DualShock 4/5, 8BitDo valdiklius — **DualShock 4 patikrintas REALIAI**
      (2026-08-21): `gilrs prijungtas name="PS4 Controller"`, X/Kvadratas/Trikampis/Nulis
      visi teisingai atpažinti (`South`/`West`/`North`/`East`). **Xbox Wireless Controller
      PATIKRINTAS REALIAI (2026-08-26, macOS):** `gilrs prijungtas name="Xbox Wireless
      Controller"` iškart, be jokio delsimo ar papildomo setup'o (žr. ADR-026). 8BitDo —
      VIS DAR NEPATIKRINTA (nė vieno neturėjo po ranka), rizika žema (SDL_GameControllerDB
      abstrakcija), bet neįrodyta.
- [x] Prijungimas veikiant nesulaužo (hot-plug) — patikrinta REALIAI: `nullbyte-emu` jau
      veikė (paleistas PRIEŠ prijungiant valdiklį), `Connected` įvykis pagautas gyvai be
      crash'o.
- [x] Veikia macOS — patikrinta: `GamepadThread::spawn()` sėkmingai inicializuoja `gilrs`
      ir švariai baigia darbą net be jokio prijungto valdiklio (`cargo test`,
      `spawn_does_not_panic_without_any_gamepad`; taip pat realiai paleidus `nullbyte-emu`
      P4.0.2 metu — jokio crash'o be prijungto valdiklio, ir su realiu DualShock 4).
      **Veikia Linux — DALINIAI PATIKRINTA 2026-08-26** (Arch, ADR-027): visas
      `cargo test --workspace` (84+80+4 testai, įsk. `spawn_does_not_panic_without_any_gamepad`)
      praėjo švariai realioje Linux mašinoje. Realaus fizinio valdiklio prijungimo Linux'e
      NEPATIKRINTA (tuo metu prieinamoje mašinoje jokio gamepad'o nebuvo — žr. atmintį).

---

### P4.2 — Įvesties mapping'as `[x]` (klaviatūra IR gamepad mygtukai/D-pad patikrinti REALIAI su Xbox valdikliu 2026-08-26)

**Priklausomybės:** P4.1, P4.0.2 (klaviatūros mapping'ui reikia realių winit `KeyboardInput`
įvykių iš `nullbyte-emu` — Tauri `Window` jų neturėjo, žr. ADR-016)
**Failai:** `crates/nullbyte-core/src/input/mapping.rs`, `crates/nullbyte-emu/src/main.rs`
(`handle_keyboard`/`drain_gamepad_events`/`send_port0_input` wiring)

> **Pastaba (ADR-016, 2026-08-20):** šis task'as sustabdytas prieš pradedant kodą, kai
> paaiškėjo, kad klaviatūros mapping'o įgyvendinti neįmanoma be proceso architektūros
> pakeitimo (žr. Fazė 4a / P4.0.x aukščiau). Gamepad mapping'o pusė NEBLOKUOJAMA — galima
> pradėti nuo jos, kol P4.0.x vyksta. „Mapping'as saugomas DB" priklauso nuo P5.1 (SQLite
> schema dar neegzistuoja) — iki tada laikyk in-memory (žr. sesijos susitarimą 2026-08-20).

> **Įgyvendinta 2026-08-25.** VIENA lentelė visiems gamepad'ams (ne per-brand'inė) — `gilrs`
> jau abstrahuoja Xbox/DualShock/8BitDo į tą patį `Button::South/East/North/West` enum'ą
> pagal FIZINĘ poziciją (žr. P4.1), tad SNES `A`/`B` persidengimas (libretro `B` = apatinis
> fizinis mygtukas) tvarkomas VIENĄ kartą, ne per gamintoją. `nullbyte-core` NEPRIKLAUSO nuo
> `winit` (tas pats principas kaip `video::renderer`), tad klaviatūros mapping'as priima
> lokalų `KeyboardKey` enum'ą — `nullbyte-emu` konvertuoja `winit::keyboard::KeyCode` → jį.
> Port'o 0 bitmask'as laikomas DVIEM ATSKIRAIS laukais (`keyboard_buttons`/`gamepad_buttons`,
> sujungiami TIK siunčiant `SetInput`) — vieno šaltinio mygtuko atleidimas kitaip galėtų
> netyčia išvalyti bitą, kurį VIS DAR laiko kitas šaltinis.
>
> **Realiai patikrinta (ne vien testais):** paleista realiu SNES ROM'u (Super Punch-Out!!,
> `nullbyte-emu` per FIFO stdin, `Run` komanda), langas rastas UŽ kitų langų (macOS
> `ActivationPolicy::Accessory` niekada automatiškai neiškelia į priekį — `osascript`
> `AXRaise` tai išsprendė; System Events `visible: false` pasirodė NEPATIKIMA accessory tipo
> programoms, tikra būsena patvirtinta ekrano nuotrauka), vartotojas realiai žaidė
> klaviatūra kelias minutes, patvirtino: „žaidžiu, viskas veikia". **Gamepad mygtukų
> mapping'as NEPATVIRTINTAS realiu valdikliu** (DualShock 4 nebuvo po ranka šią sesiją, tas
> pats apribojimas kaip P4.1 Xbox/8BitDo) — logika identiška klaviatūrai, 6 unit testais
> padengta (`joypad_bit`, SNES layout swap, D-pad/shoulders, unmapped buttons), bet fizinis
> patikrinimas lieka atviras.

> **Realiai patikrinta Xbox valdikliu (2026-08-26) — žr. ADR-026 pilnam sprendimų
> žurnalui.** Visi mygtukai patikrinti PO VIENĄ, realiais paspaudimais, per specialiai tam
> parašytą `gilrs`-lygio diagnostikos skriptą (parodo kiekvieno mygtuko `gilrs::Button`
> pavadinimą IR jo `default_gamepad_mapping()` rezultatą gyvai): A→SNES B, B→SNES A, X→SNES
> Y, Y→SNES X (SNES layout swap PATVIRTINTAS teisingas), LB→L, RB→R, LT→L2, RT→R2, Back→
> SELECT, Start→START, kairio/dešinio stiko paspaudimai→L3/R3 — VISI teisingi. **D-pad
> ATSKLEIDĖ realų bug'ą:** šis valdiklis D-pad siunčia KAIP AŠĮ (`Axis::DPadX`/`DPadY`), NE
> kaip `Button::DPad*` — `nullbyte-emu` iki šiol VISIŠKAI ignoravo `AxisChanged` įvykius
> realiame žaidimo kelyje (sąmoningas MVP apribojimas P4.2 metu, žr. senesnę pastabą kode),
> tad D-pad šiuo valdikliu būtų neveikęs iš viso. Pataisyta (nauja
> `mapping::dpad_axis_ids`/`AXIS_DPAD_THRESHOLD`, `main.rs` `AxisChanged` apdorojimas) IR
> patvirtinta REALIU ŽAIDIMU (ActRaiser, SNES) — vartotojas patvirtino: „veikia, personažas
> juda visomis kryptimis".

**Ką daryti:**
- Fizinis mygtukas → `RETRO_DEVICE_ID_JOYPAD_*` (B,Y,SELECT,START,UP,DOWN,LEFT,RIGHT,A,X,L,R,L2,R2,L3,R3)
- ~~Numatytieji mapping'ai pagal valdiklio tipą~~ — NETEISINGA PRIELAIDA: `gilrs` jau
  abstrahuoja valdiklio tipą (žr. pastabą aukščiau), per-brand'inės lentelės nereikia.
- **Dėmesio:** libretro `A`/`B` yra SNES išdėstyme — Xbox valdiklio `A` fiziškai atitinka
  libretro `B`. Numatytasis mapping'as turi tai gerbti.
- Klaviatūros numatytieji: strėlės + `Z`/`X`/`A`/`S` + `Enter`/`Shift`
- ~~Mapping'as saugomas DB (per-vartotoją, per-platformą)~~ — atidėta P5.1 (žr. ADR-016
  pastabą aukščiau), hardkodintas numatytasis mapping'as kol kas VIENINTELIS.

**Acceptance:**
- [x] SNES žaidimas valdomas gamepad'u teisingai — REALIAI PATIKRINTA Xbox valdikliu
      2026-08-26 (žr. pastabą aukščiau): visi mygtukai + D-pad (po fix'o) veikia teisingai
      realiame žaidime
- [x] Klaviatūra veikia — patikrinta REALIAI (žr. pastabą aukščiau)
- [ ] Mapping'as išlieka po perkrovimo — N/A kol P5.1 neegzistuoja (žr. ADR-016 pastabą);
      hardkodintas mapping'as savaime „išlieka", nes nėra ką prarasti perkrovus

---

### P4.3 — Įvesties polling ir input bitmask `[ ]` (klaviatūros delsa patvirtinta REALIAI 2026-08-25; 2 gamepad'ai vienu metu — nepatikrinta, tik 1 valdiklio po ranka)

**Priklausomybės:** P4.2, P1.4
**Failai:** `crates/nullbyte-core/src/input/mod.rs`, `crates/nullbyte-core/src/core/callbacks.rs`,
`crates/nullbyte-emu/src/main.rs` (`gamepad_ports`/`assign_gamepad_port`/`send_port_input`)

> **Pastaba (2026-08-20):** dauguma šio task'o „Ką daryti" punktų jau įgyvendinti anksčiau —
> `EmuContext.input_state: [u16; 4]` (4 portai), `input_state_cb` (P1.4) ir
> `GET_INPUT_BITMASKS => true` (P1.5) jau egzistuoja `core/callbacks.rs`/`core/environment.rs`.
> Šio task'o TIKRASIS likęs darbas — sujungti P4.1 (`gilrs`) ir P4.2 (mapping) su šiuo jau
> veikiančiu bitmask sluoksniu per `EmuCommand::SetInput`, IR pačios ARCHITEKTŪROS pokytis:
> žr. ADR-016 — nuo šios užduoties emuliavimo gija (taigi ir `input_state` atnaujinimas)
> gyvena `nullbyte-emu` vaiko procese, ne Tauri procese. Grįžtame prie šio task'o po ADR-016
> dokumentacijos atnaujinimo (žr. pokalbio kontekstą).

> **Baigta 2026-08-25** (kiek įmanoma be antro valdiklio). P4.2 metu port'as visada buvo
> hardkodintas į 0 — šis task'as pridėjo REALŲ port'ų priskyrimą: `App.gamepad_ports`
> (`HashMap<gilrs id, port>`) priskiria kiekvieną prisijungusį gamepad'ą KITAM laisvam
> port'ui (0..4) pirmo prisijungimo eile, atsijungus port'as atlaisvinamas (IR išvalomas —
> kitaip paskutinė žinoma bitmask'o reikšmė liktų „įstrigusi"). Klaviatūra visada valdo TIK
> port'ą 0 (fiziškai negali būti antras žaidėjas vienu metu su savimi), sujungiama su TO
> PORTO gamepad'u TIK siunčiant `SetInput` (ne bendru lauku — žr. `send_port_input` doc dėl
> KODĖL). `drain_gamepad_events` per vieną `about_to_wait()` ciklą gali paveikti KELIS
> skirtingus portus vienu metu — kiekvienas pakitęs port'as siunčiamas atskirai.
>
> **Realiai patikrinta:** klaviatūros valdymas (SNES, kelios minutės žaidimo) — vartotojas
> nejautė jokios delsos. **NEpatikrinta realiu hardware'u:** 2 gamepad'ai vienu metu (tik
> vienas DualShock 4 buvo po ranka P4.1 metu, šią sesiją — nė vieno) — logika ta pati, kuri
> jau veikia vienam valdikliui (tik `gamepad_ports` priskyrimas), bet fizinis patikrinimas
> su realiu 2-player žaidimu lieka atviras, tas pats apribojimų klasė kaip P4.1/P4.2.

> **Papildomai patikrinta 2026-08-26:** vieno gamepad'o (Xbox, port 0) input routing per
> `send_port_input`/`SetInput` REALIAI veikia žaidime — įsk. NAUJAI pridėtą D-pad-per-ašį
> kelią (žr. P4.2 pastabą, ADR-026), kuris irgi eina per TĄ PATĮ port'ų priskyrimo
> mechanizmą. 2 gamepad'ai vienu metu — VIS DAR NEPATIKRINTA (tik vienas Xbox valdiklis
> buvo po ranka).

**Ką daryti:**
- `retro_set_input_poll` → atnaujina `EmuContext.input_state`
- `retro_set_input_state` → grąžina iš to būvio
- Palaikyk `GET_INPUT_BITMASKS` (greičiau — vienas kvietimas vietoj 16)
- Iki 4 portų (multiplayer)

**Acceptance:**
- [x] Įvesties delsa nejuntama (subjektyviai) — patikrinta REALIAI (žr. pastabą aukščiau);
      objektyvus matavimas (< 1 kadras) neatliktas, tik subjektyvus vartotojo patvirtinimas
- [!] 2 gamepad'ai vienu metu veikia (testuok su 2-player žaidimu) — logika įgyvendinta ir
      pagrįsta jau veikiančiu 1-gamepad keliu, bet NEPATIKRINTA su realiais 2 valdikliais

---

### P4.4 — Hotkey'ai `[!]` (F1/F2/F5/Cmd+R patikrinti REALIAI 2026-08-25; likusieji — tik unit testais, ta pati kodo šaka)

**Priklausomybės:** P4.3
**Failai:** `crates/nullbyte-core/src/input/hotkeys.rs` (naujas), `crates/nullbyte-core/src/input/mod.rs`,
`crates/nullbyte-emu/src/main.rs` (`handle_hotkey_action`/`toggle_fullscreen`/`ModifiersChanged` wiring)

> **Įgyvendinta 2026-08-25.** `HotkeyKey`/`HotkeyAction` (`hotkeys.rs`) — tas pats
> `winit`-nepriklausomybės principas kaip `mapping.rs` (žr. jo doc). `resolve_hotkey()` yra
> GRYNA funkcija be jokio mutable būvio — `TogglePause` grąžina abstraktų veiksmą, o
> `nullbyte-emu` (`App.paused: bool`) sprendžia, ar siųsti `Pause`, ar `Resume`, nes
> `EmuCommand` neturi „koks dabar būvis" užklausos. `Space` (fast-forward) SĄMONINGAI NĖRA
> `resolve_hotkey()` dalis — tai vienintelis „laikymas = būvis" hotkey (kaip žaidimo
> mygtukas), o visi kiti yra „paspaudimas = vienas veiksmas" (trigger); `nullbyte-emu`
> apdoroja `Space` press/release ATSKIRAI, prieš patikrindamas kitus hotkey'us.
> `WindowEvent::ModifiersChanged` (naujas `main.rs`) reikalingas, nes `winit`
> NETEIKIA modifikatorių tiesiogiai `KeyEvent`'e.
>
> **Quick save/load (F2/F4)** naudoja rezervuotą slot'ą `0`, atskirą nuo numeruotų F5-F8
> slot'ų (`1..=4`) — kad vartotojas netyčia neperrašytų pavadinto save'o. **`Esc`** šiuo metu
> TIK išeina iš fullscreen — „grįžti į biblioteką" dalis N/A, nes bibliotekos lango dar nėra
> (P7 UI nepradėta). **`SaveState`/`LoadState`** patys per se NEVEIKIA (P8.1 dar
> neimplementuota, `EmuThread` juos tik logina ir ignoruoja, žr. `core::runner`) — šis task'as
> pridėjo TIK teisingą klavišo → komandos wiring'ą, ne pačią save state funkciją.
>
> **Realiai patikrinta** (SNES ROM, `nullbyte-emu` per FIFO stdin): `F1` (pauzė → `paused=true`
> log'e), `F2` (quick save), `F5` (save state slot 1), `Cmd+R` (reset) — visi keturi teisingai
> nusiuntė atitinkamą komandą. **NEpatikrinta gyvai:** `Shift+F5` (load slot), `F11`
> (fullscreen), `Esc`, `Space` (fast-forward) — `osascript` klavišų injektavimas į macOS
> Accessory-tipo langą pasirodė nepatikimas (fokusas nuklysdavo atgal į terminalą net po
> `AXRaise`, tiek globalus, tiek process-scoped `System Events` sintaksės variantas) —
> testavimo įrankio, ne produkto, problema. Šie keturi padengti unit testais
> (`f5_through_f8_save_without_shift_load_with_shift` patikrina TIKSLIAI Shift+F5 atvejį) IR
> naudoja LYGIAI TĄ PAČIĄ `resolve_hotkey`/`handle_hotkey_action` kodo šaką, kuri jau
> patvirtinta gyvai kitiems trims trigger-tipo hotkey'ams.

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
- [!] Visi hotkey'ai veikia — 4/9 patikrinti REALIAI, likusieji tik unit testais (žr. pastabą
      aukščiau); `SaveState`/`LoadState` patys funkcionalumas laukia P8.1
- [x] Nekonfliktuoja su žaidimo įvestimi — hotkey klavišai (`F1`-`F11`, `Esc`, `Cmd/Ctrl+R`)
      ir žaidimo klavišai (strėlės, `Z`/`X`/`A`/`S`, `Enter`/`Shift`) visiškai nesikerta pagal
      konstrukciją (`handle_keyboard` patikrina hotkey PIRMIAU, `return` neleidžia patekti į
      žaidimo mapping'ą)

---

## 7. Faza 5 — Duomenų bazė ir biblioteka

**Tikslas:** ROM skenavimas ir saugojimas SQLite.
**Rizika:** 🟢 maža. **Įvertis:** 2–3 dienos.

### P5.1 — SQLite schema ir migracijos `[x]`

**Priklausomybės:** P0.3
**Failai:** `crates/nullbyte-app/migrations/001_initial.sql`, `crates/nullbyte-app/src/db/migrations.rs`, `db/models.rs`

> **Įgyvendinta 2026-08-25.** Migracijos per `PRAGMA user_version` (`const MIGRATIONS: &[(u32,
> &str)]`, `include_str!` — vieninteliai SQL failai, ne runtime katalogo skenavimas, nes
> bundle'inta app'a negarantuoja `migrations/` katalogo egzistavimo runtime metu).
> `foreign_keys = ON` nustatoma PRIE KIEKVIENO `Connection::open()` kvietimo, NE migracijos
> SQL faile — SQLite tai laiko per-connection nustatymu (numatytai OFF), CLAUDE.md §10 tai
> eksplicitiškai reikalauja. `AppState.db: Mutex<Connection>` (rusqlite `Connection` NĖRA
> `Sync`).
>
> **Seed platformos (23, ne tik minimalūs 20):** README.md „Works during the MVP" sąrašas,
> `screenscraper_id` PATIKRINTA prieš community-sourced ScreenScraper systemeid lentelę
> (`gist.github.com/dollerbill/86162c5cb249d79ef01a9ad2c691d29d`, patikrinta per `WebFetch`
> 2026-08-25) — TAI NĖRA oficialus ScreenScraper API atsakas (reikalauja `devid`/
> `devpassword`, P6.1 dar nepradėtas). 14 platformų su REALIU, patikrintu ID; 9, kurių
> nepavyko patikrinti šiame šaltinyje (Atari 7800/800/5200, Neo Geo, Arcade, Intellivision,
> Odyssey²), turi `screenscraper_id = NULL` — SĄMONINGAI, ne spėjama reikšmė (žr. CLAUDE.md
> atminties taisyklę „Verify external API refs"). P6.1 API klientas juos patvirtins/pataisys
> prieš tikrą API atsakymą.
>
> **Pastaba (P5.3, 2026-08-25):** `platforms.extensions` reikšmės PSX/Saturn/SegaCD/Neo Geo/
> Arcade platformoms buvo klaidingos (kelios platformos dalinosi bendru `zip` plėtiniu,
> DVIPRASMIŠKAI) — ištaisyta NAUJA migracija `002_fix_archive_extensions.sql`, ne šio failo
> redagavimu (CLAUDE.md §12 DoD). Žr. P5.3 pastabą dėl detalių.
>
> **Pastaba (P5.4, 2026-08-25):** `games_fts` (žr. schemą aukščiau) NETURĖJO sync trigerių —
> external-content FTS5 lentelė be jų VISADA liktų tuščia, nepriklausomai nuo `games` turinio.
> Ištaisyta NAUJA migracija `003_games_fts_sync_triggers.sql`. Žr. P5.4 pastabą dėl detalių.
>
> **Realiai patikrinta** (ne vien unit testais) — tikras `target/debug/nullbyte-app`
> paleidimas DU kartus iš eilės: pirmą kartą sukūrė `nullbyte.db`/`-shm`/`-wal` faktiniame
> `~/Library/Application Support/Nullbyte/` kelyje, `sqlite3` CLI (nepriklausomas nuo mūsų
> kodo) patvirtino `PRAGMA user_version = 1`, `PRAGMA journal_mode = wal`, 23 eilutes
> `platforms`, teisingus ID (`snes→4`, `genesis→1`, `psx→57`, `nes→3`, `arcade→NULL`), visas
> lenteles (įskaitant `games_fts` FTS5 vidines lenteles). Antras paleidimas — VIS DAR 23
> eilutės (jokio dubliavimo).

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
- [x] DB sukuriama pirmą kartą paleidus — patikrinta REALIU `nullbyte-app` paleidimu, ne
      vien testu (žr. pastabą aukščiau)
- [x] Migracijos idempotentiškos (paleisk 3 kartus) — unit testas (3x `run_migrations`) IR
      realus 2x `nullbyte-app` paleidimas, abu be dubliavimo
- [x] Seed platformos įrašytos — 23 (README „Works during the MVP" sąrašas), 14 su realiu
      patikrintu ScreenScraper ID, 9 su sąmoningu `NULL`

---

### P5.2 — ROM hash'avimas `[x]`

**Priklausomybės:** P5.1
**Failai:** `crates/nullbyte-app/src/library/hasher.rs`

> **Įgyvendinta 2026-08-25.** `crc32fast`/`md-5`/`sha1` pridėta prie `nullbyte-app`
> Cargo.toml (jos jau buvo `nullbyte-core` priklausomybės, bet hash'avimo LOGIKA neturi tos
> pačios „reikia abiem pusėms" priežasties kaip `archive.rs` — tik `nullbyte-app`'o
> bibliotekos skeneriui, tad gyvena čia, ne bendrame crate'e). Archyvams naudoja
> `nullbyte_core::archive::extract_first_match` (bendra su `core::loader`), paduodant
> PLATFORMOS `extensions` stulpelį (ne core'o `valid_extensions`) kaip „ko ieškoti viduje".
> Streaming (>64MB) įgyvendintas NEARCHYVUOTIEMS failams — `nullbyte_core::archive` API jau
> skaito visą vidinį failą į atmintį (esama riba, bendra su core'o krovimu), tad archyvų
> streaming'as liko NEĮGYVENDINTAS (dokumentuota `hasher.rs` doc). NES (iNES) header skip
> įgyvendintas per MAGIC baitus (`4E 45 53 1A`), veikia nepriklausomai nuo plėtinio; SNES
> copier header (512B, be patikimo magic baito) ATIDĖTAS.
>
> **Rasta klaida SAVO test vektoriuose, ne implementacijoje** — pirmi CRC32/SHA1 „žinomi"
> test vektoriai `"abc"` tekstui buvo transkribuoti klaidingai iš atminties (CRC32 `...C1`
> vietoj `...C2`, SHA1 trūko paskutinio `d` simbolio). Patikrinta NEPRIKLAUSOMU šaltiniu
> (Python `hashlib`/`zlib`, ne mūsų kodas) PRIEŠ taisant testus — implementacija buvo
> teisinga nuo pat pradžių, testai — ne (žr. atminties taisyklę „Verify external API refs" —
> tas pats principas taikomas ir kriptografiniams test vektoriams, ne tik API atsakymams).
>
> **Realiai patikrinta** (ne vien sintetiniais duomenimis): `real_rom_hash_matches_system_tools`
> (naujas `#[ignore]` testas) — tikras SNES ROM'as, MD5/SHA1 sutapo su macOS `md5`/
> `shasum -a 1` komandomis (nepriklausomu šaltiniu, ne mūsų `md-5`/`sha1` crate'ais).
> `hashing_100_files_under_30_seconds` (release profilis) — 91 realus test fixture failas
> (SNES/Genesis/PSX/GBA, 1.53 GB) hash'uota per **14.12s** (< 30s riba su atsarga).

**Ką daryti:**
- CRC32 (`crc32fast`), MD5 (`md-5`), SHA1 (`sha1`) — vienu perėjimu per failą
- Archyvams — hash'uok **vidinį** failą
- Failams > 64 MB: streaming, ne visas į atmintį
- **Header skip:** kai kurioms sistemoms hash skaičiuojamas be header'io
  (NES iNES 16 baitų, SNES 512 baitų copier header) — implementuok bent NES atvejį

**Acceptance:**
- [x] Hash'ai sutampa su `sha1sum`/`md5sum` komandinės eilutės rezultatais — patikrinta
      REALIAI (tikras SNES ROM'as vs macOS `md5`/`shasum -a 1`)
- [x] 100 failų (~2 GB) hash'avimas < 30 s SSD'e — 91 realus failas (1.53 GB) per 14.12s
      (release profilis)
- [x] `.zip` vidinis failas hash'uojamas teisingai — unit testas IR realus PSX `.zip`
      (400-500MB kiekvienas, `hashing_100_files_under_30_seconds` sudėtyje)

---

### P5.3 — ROM skeneris `[x]`

**Priklausomybės:** P5.2
**Failai:** `crates/nullbyte-app/src/library/scanner.rs`

> **Įgyvendinta 2026-08-25.** `scan()` SĄMONINGAI nepriklauso nuo Tauri tipų — `on_progress`
> paprastas `FnMut`, ne `tauri::ipc::Channel` (tas pats principas kaip `core::runner`/
> `input::gamepad`: domeno logika testuojama be UI/IPC karkaso, komandų sluoksnis (P7) jį
> persiunčia per tikrą `Channel<ScanProgress>`). Inkrementinis skenavimas tikrina
> `file_mtime` PRIEŠ bet kokį platformos/hash'o skaičiavimą (ne po) — nepakitusiam failui
> visa tai praleidžiama, ne tik DB rašymas. `rom_path` saugomas ABSOLIUČIAI, ne santykinai
> (skirtingai nuo media cache §9.4) — ROM katalogai gali būti bet kur diske, keli vienu metu,
> „santykinis nuo ko" būtų dviprasmiškas be papildomo JOIN'o. Ištrinti failai — PAŠALINAMI
> (ne pažymimi; schema neturi „missing" statuso stulpelio šiam tikslui).
>
> **Rasta ir pataisyta reali P5.1 seed duomenų spraga** — atskleidė BŪTENT šis skeneris,
> paleistas prieš realius fixture failus: keli platformos (PSX/Saturn/SegaCD IR Neo Geo/
> Arcade) dalinosi bendru `zip` plėtiniu, tad grynai extension-based atpažinimas buvo
> DVIPRASMIS (PSX `.zip` netyčia atpažintas kaip Neo Geo, GBA `.zip` — kaip Saturn).
> Sprendimas dviem dalimis: (1) `resolve_platform_and_hashes()` archyvams BANDO kiekvieną
> kandidatą (platformas, kurių `extensions` sąraše yra `zip`/`7z`) ir naudoja tą, kurio
> VIDINIS failas realiai atsiranda archyve — pigu, nes `archive::extract_first_match`
> netikrina turinio, kol nerado sutampančio vardo; (2) NAUJA migracija
> `002_fix_archive_extensions.sql` (NE senos 001 redagavimas — CLAUDE.md §12 DoD) ištaiso
> pačius seed duomenis: PSX/Saturn/SegaCD gauna `zip,7z` (jie TIKRAI taip platinami, mūsų
> „vienas vidinis failas" modelis jiems tinka), Neo Geo/Arcade PRARANDA `zip` (jų realūs
> MAME romset'ai turi daugybę failų viename archyve — tas pats radinys, kurį šios sesijos
> anksčiau atskleidė MAME rankinis testas, žr. P4.0.5 istoriją — vienas-vidinis-failas
> modelis jiems tiesiog netinka, tad neteisinga tvirtinti, kad mokame juos skenuoti).
>
> **Realiai patikrinta** (ne vien sintetiniais duomenimis): `scan_real_fixtures_is_fast` —
> 68 realūs SNES/Genesis/PSX/GBA fixture failai (release profilis): pirmas skenavimas
> **0.42s**, pakartotinis (be pakeitimų) — **0.00s**, abu giliai po 60s/2s ribų. Taip pat
> aptikta ir pataisyta reali klaida (`/tmp` → `/private/tmp` macOS simlink), dėl kurios
> ištrintų failų aptikimas realiame failų sistemos kontekste NIEKADA nesuveiktų be
> `canonicalize()` katalogo keliui prieš `LIKE` palyginimą — atrasta TIK per realų testą su
> tikru failų ištrynimu, ne sintetiniu keliu.

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
- [x] 500 ROM'ų katalogas nuskenuojamas < 60 s — 68 realūs failai per 0.42s (release);
      extrapoliuojant iki 500 (panašaus dydžio SNES/Genesis ROM'ų) liktų giliai po riba
- [x] Pakartotinis skenavimas be pakeitimų < 2 s — 0.00s realiems 68 failams
- [x] Progresas rodomas realiu laiku — `on_progress` kviečiamas po KIEKVIENO failo (unit
      testas `scan_inserts_new_games_and_skips_unknown_extensions` patikrina kvietimų skaičių)
- [x] Pavadinimai išvalyti teisingai (testai su 20 pavyzdžių) — lygiai 20, visi praeina

---

### P5.4 — Bibliotekos užklausos `[x]`

**Priklausomybės:** P5.3
**Failai:** `crates/nullbyte-app/src/db/games.rs`, `crates/nullbyte-app/src/commands/library.rs`

> **Įgyvendinta 2026-08-25.** PIRMAS task'as, kuris realiai užpildo `commands/` sluoksnį ir
> registruoja komandas `lib.rs` `generate_handler![]` — iki šiol frontend'as turėjo tik
> `greet`/`get_app_info`. `db/games.rs` (grynas SQL, be Tauri) + `commands/library.rs`
> (plonas — tik `state.db.lock()` + delegavimas, CLAUDE.md §6.3) atskirti, kaip ir kitur šiame
> projekte. `list_games` sudaro SQL dinamiškai (`Vec<Box<dyn ToSql>>` parametrams) — FTS5
> `JOIN` PRIDEDAMAS TIK kai `search` netuščias po išvalymo, kad tuščia paieška neuždėtų
> nereikalingo JOIN'o. Paieškos terminas verčiamas į FTS5 prefikso sintaksę (`mario*`).
>
> **Rasta ir pataisyta reali spraga (dar viena, P5.1 metu praleista)**: `games_fts` yra
> external-content FTS5 lentelė (`content='games'`) — SQLite JOS NESINCHRONIZUOJA
> automatiškai, reikia eksplicitinių trigerių. Be jų `games_fts` visada liktų tuščia,
> NEPRIKLAUSOMAI nuo `games` turinio — paieška tyliai negrąžintų NIEKO, jokios klaidos,
> pavojingiausia klaidų rūšis. Ištaisyta NAUJA migracija (ne 001 redagavimas)
> `003_games_fts_sync_triggers.sql` — standartinis SQLite external-content sync šablonas
> (AFTER INSERT/UPDATE/DELETE trigeriai) + backfill esamiems įrašams. Patikrinta REALIU
> testu (`deleted_game_disappears_from_search_too`), kad DELETE per trigerį pašalina įrašą
> IR iš `games_fts`, ne tik iš `games`.
>
> **Realiai patikrinta**: FTS5 paieška 5000 sintetinių įrašų — **0.33ms** (limitas 50ms,
> >100x atsarga). Migracijos 2+3 pritaikytos ant TIKRO, jau egzistuojančio DB failo
> (`~/Library/Application Support/Nullbyte/nullbyte.db`, sukurto P5.1 metu su
> `user_version=1`) — `user_version` teisingai pakilo į 3, PSX/Saturn/SegaCD/Neo Geo/Arcade
> `extensions` teisingai pataisyti, visi 3 trigeriai realiai sukurti (`sqlite_master`
> patvirtina) — tai realus ATNAUJINIMO kelias, ne švarus testas nuo nulio.

**Ką daryti:**
- `list_games(filter)` — filtras: platforma, paieška (FTS5), favorite, rūšiavimas, puslapiavimas
- `get_game(id)`, `set_favorite(id, bool)`, `record_play(id, seconds)`
- `list_platforms()` — su žaidimų kiekiu
- Tauri komandos plonos, grąžina `camelCase` JSON

**Acceptance:**
- [x] Paieška „mario" randa visus Mario žaidimus < 50 ms su 5000 įrašų — 0.33ms realiai
- [x] Puslapiavimas veikia — unit testas (`pagination_returns_correct_slice`)
- [x] TS tipai atitinka Rust struct'us — `src/lib/types/index.ts` atnaujintas (`Game`,
      `Platform`, `PlatformSummary`, `GameFilter`, `SortField`, `SortDirection`), `pnpm check`
      švarus

---

## 8. Faza 6 — ScreenScraper

**Tikslas:** metaduomenys, viršeliai ir gameplay video.
**Rizika:** 🟡 vidutinė (išorinis API, kvotos). **Įvertis:** 2–3 dienos.

### P6.1 — API klientas `[x]`

**Priklausomybės:** P5.1
**Failai:** `crates/nullbyte-app/src/scraper/screenscraper.rs`, `crates/nullbyte-app/src/scraper/types.rs`

> **Įgyvendinta 2026-08-25.** Vartotojas turėjo TIKRUS ScreenScraper devid/devpassword IR
> ssid/sspassword kredencialus — patvirtinti realiu `ssuserInfos.php` kvietimu PRIEŠ rašant
> kodą (`success: true`, `maxrequestsperday: 20000`). `types.rs` struktūra NUSTATYTA REALIU
> `jeuInfos.php` atsakymu (Super Metroid, SNES, `crc=AD2CBF9C`), ne spėta iš CLAUDE.md §9.1
> aprašymo (žr. atminties taisyklę „Verify external API refs" — principas taikomas ir API
> atsakymo FORMAI, ne tik atskiriems laukams).
>
> **Du realūs radiniai, kuriuos atskleidė TIK gyvas API kvietimas:**
> 1. **Strategija supaprastinta** — CLAUDE.md aprašo „hash'ai → jei nerado, pavadinimas" kaip
>    DU žingsnius, bet realus API PATS bando abu VIENOJE užklausoje (patikrinta: `crc` +
>    `romnom` kartu grąžino teisingą atsakymą pirmu bandymu). `lookup_game()` siunčia visus
>    turimus laukus VIENU HTTP kvietimu — antras raundas nereikalingas.
> 2. **„Nerasta" = HTTP 404 su PAPRASTU TEKSTU** (`"Erreur : Rom/Iso/Dossier non trouvée !"`),
>    NE JSON — net su `output=json`. Be šio patikrinimo PRIEŠ JSON parsinimą, KIEKVIENAS
>    „nerasta" atvejis būtų klaidingai užkritęs kaip „blogas JSON", ne teisingai atpažintas
>    kaip `NotFound`.
>
> **Reali klaida sugauta RAŠANT TESTUS, ne implementacijoje** (trečia tokia šią sesiją, žr.
> P5.2/P5.3/P5.4 panašius radinius): `pick_available_region()` naiviai imdavo PIRMĄ `noms`
> masyve esantį regioną, kuris BET KOKS priklauso prioriteto sąrašui — realiame atsakyme
> `noms` atėjo tvarka `[ss, us, jp, eu]`, tad grąžindavo „ss", nors CLAUDE.md §9.2 prioritetas
> aiškiai sako „eu" turėtų laimėti. Pataisyta iteruojant PER PRIORITETO sąrašą, ne per `noms`
> (tas pats principas, kurį `pick_region_text`/`pick_lang_text` jau darė teisingai). Taip pat
> testas, bandęs simuliuoti „SSID nenustatytas" per `env::remove_var`, KRITO REALIAME repo,
> nes TIKRAS `.env` failas egzistuoja diske — `dotenvy::dotenv()` jį randa einant per
> tėvinius katalogus ir užpildo „nenustatytą" kintamąjį TIKRA reikšme. Pataisyta: testas
> nustato VISUS keturis kintamuosius eksplicitiškai, nesikliaudamas „nenustatyta" elgesiu.
>
> Nauja priklausomybė: `dotenvy = "0.15"` (`.env` skaitymui — CLAUDE.md §11.8 reikalauja
> MVP.md įrašo naujoms priklausomybėms, tai jis). `.env.example` papildytas
> `SCREENSCRAPER_SSID`/`SSPASSWORD` (neprivalomi, CLAUDE.md §9.3).
>
> **Realiai patikrinta** (ne vien fixture'u): `real_snes_rom_finds_metadata` — tikras SNES
> ROM'as (Super Metroid) rado teisingus metaduomenis (developer/publisher „Nintendo", genre
> „Platform", region „eu" po pataisymo) per GYVĄ API kvietimą. `real_unknown_rom_is_not_found`
> — išgalvotas CRC/pavadinimas grąžino `NotFound`, ne klaidą.

**Ką daryti:**
- `reqwest` klientas į `https://www.screenscraper.fr/api2/jeuInfos.php`
- Parametrai pagal `CLAUDE.md` §9.1
- Strategija: `crc`+`md5`+`sha1`+`romtaille` → jei nerado, `romnom`+`systemeid`
- JSON atsako struct'ai (`serde`) — atsargiai, ScreenScraper JSON yra nenuoseklus
  (kartais masyvas, kartais objektas → naudok `#[serde(untagged)]` kur reikia)
- Regionų ir kalbų prioritetai iš `CLAUDE.md` §9.2

**Acceptance:**
- [x] Žinomas SNES ROM'as randa teisingus metaduomenis — patikrinta REALIU API kvietimu
- [x] Nežinomas ROM'as → `NotFound`, ne klaida — patikrinta REALIU API kvietimu (HTTP 404)
- [x] Blogas JSON nesulaužo (graceful degradation) — grąžina `Err`, ne panic'ina
- [x] Credentials iš `.env` / nustatymų, ne hardcode — `ScreenScraperCredentials::from_env()`

---

### P6.2 — Rate limiting ir cache `[x]`

**Priklausomybės:** P6.1
**Failai:** `crates/nullbyte-app/src/scraper/rate_limit.rs`, `scrape_cache` lentelė

> **Įgyvendinta 2026-08-25.** `types.rs` papildytas `Serveurs`/`SsUser` struct'ais — REALIU
> `jeuInfos.php` kvietimu patikrinta (2026-08-25), kad `serveurs`/`ssuser` blokai yra
> KIEKVIENAME sėkmingame atsakyme, ne tik dedikuotame `ssuserInfos.php` endpoint'e (žr. P6.1
> pastabą tuo pačiu principu). `screenscraper::lookup_game()` grąžina naują `LookupSuccess`
> (`ScrapeOutcome` + `Option<QuotaInfo>`) per naują `LookupError` (`RateLimited` vs. `Failed`)
> — atskirtas nuo bendro `AppError`, kad `rate_limit.rs` galėtų skirtingai reaguoti (backoff
> vs. tiesiog klaida), nepažeidžiant CLAUDE.md §6.1 „vieno klaidų tipo" taisyklės (`LookupError`
> lieka crate-vidinis, konvertuojamas į `AppError` TIK po išnaudotų bandymų).
>
> **Sąžiningas apribojimas:** 429/430/„API closed" šaka NĖRA patikrinta gyvu API atsakymu —
> realaus limito pasiekimas reikalautų sąmoningai išeikvoti vartotojo TIKRĄ dienos kvotą, kas
> nebūtų atsakingas dev kredencialų naudojimas. Ši šaka remiasi TIK MVP.md specifikacijos
> tekstu (žr. kodo komentarą `screenscraper.rs::lookup_game`) — pažymėta aiškiai, kad
> ateityje, jei elgesys pasirodys kitoks, būtų aišku KUR tikrinti pirmiausia.
>
> `RateLimiter` (semaforas, prasideda nuo 1 leidimo, auga TIK aukštyn per `update_maxthreads` —
> tokio `Semaphore` neturi saugaus būdo atimti jau išduotus leidimus, MVP supaprastinimas).
> `cached_lookup()` priima injektuojamą `fetch` closure'ą (ne tiesiogiai
> `screenscraper::lookup_game`), kad greiti testai patikrintų cache/backoff logiką be tinklo.
>
> **Realiai patikrinta** (ne vien injektuotu `fetch`, žr. sesijos pastabą apie sluoksnių
> sujungimo klaidas — P6.1 sukūrė HTTP+JSON sluoksnį, P6.2 jį PIRMĄ KARTĄ realiai panaudoja):
> `real_lookup_populates_cache_then_second_call_is_cache_hit` — `cached_lookup` sujungtas su
> TIKRU `screenscraper::lookup_game`, tikru `.env`, tikru SQLite failu (ne `:memory:`). Pirmas
> kvietimas gavo realų Super Metroid atsakymą IR realią kvotą (`maxthreads: 1,
> maxrequestsperday: 20000`), antras kvietimas su tuo pačiu raktu grąžino iš cache
> (`from_cache: true`), be naujo tinklo kvietimo.

**Ką daryti:**
- Prieš užklausą — tikrink `scrape_cache`
- Cache'uok ir sėkmes (be TTL), ir „notfound" (TTL 7 dienos)
- Semaforas pagal `ssuser.maxthreads` iš atsako (numatytoji 1)
- Exponential backoff: 429/430/`API closed` → 2s, 4s, 8s, 16s, tada sustok
- Kvotos likutis iš atsako (`ssuser.requeststoday` / `maxrequestsperday`) → rodyk UI

**Acceptance:**
- [x] Pakartotinė užklausa nesikreipia į tinklą — patikrinta REALIU API kvietimu pirmam,
      cache'u antram (žr. aukščiau) IR sintetiniu testu (`fetch` iškviestas lygiai 1 kartą)
- [x] Kvotos viršijimas nesulaužo — sustoja su aiškiu pranešimu UI (`AppError::Other` su
      „kvota viršyta..." tekstu po backoff'o išnaudojimo) — 429/430 šaka NEVERIFIKUOTA gyvai
      (žr. pastabą aukščiau), tik sintetiniais testais pagal MVP.md specifikaciją
- [x] Vienalaikių užklausų nedaugiau nei `maxthreads` — `RateLimiter` semaforas, testas
      `update_maxthreads_only_grows_and_adds_correct_permit_delta`

---

### P6.3 — Media atsisiuntimas `[x]`

**Priklausomybės:** P6.2
**Failai:** `crates/nullbyte-app/src/scraper/media.rs`

> **Įgyvendinta 2026-08-25.** `download_game_media()` sąmoningai priima `&[Media]` (žaliavinį
> `types.rs` tipą), NE `screenscraper::Jeu` ar `GameMetadata` — media.rs nežino nieko apie
> ScreenScraper užklausos formavimą, tik apie tai, kaip iš duotų media įrašų pasirinkti
> geriausią kiekvienam tipui (regionų prioritetas kaip §9.2) ir saugiai atsisiųsti. Kas paduos
> `medias` iš tikro `jeu` atsakymo — P6.4 orkestracijos sprendimas, dar nepriimtas.
>
> Atominis rašymas: `{final}.tmp` → `rename` (POSIX `rename` tame pačiame FS taške yra
> atominis). Pavienio media įrašo klaida (bloga nuoroda, nutrūkęs ryšys, viršytas dydžio
> limitas) NIEKADA negrąžina `Err` iš `download_game_media` — tik `None` tam vienam laukui
> (`tracing::warn!` paliekamas pėdsakas), nes vieno viršelio nepavykimas neturėtų sužlugdyti
> viso žaidimo scraping'o. `Err` grąžinamas TIK jei pats `media_dir` katalogas nesukuriamas.
>
> **Saugumo taisyklė, pritaikyta testams (žr. `types.rs`/P6.1 pastabą):** realaus
> ScreenScraper atsakymo `medias[].url` turi devid/devpassword ATVIRU TEKSTU — gyvas testas
> TIESIOGIAI negalėjo naudoti `screenscraper::lookup_game()` (jis grąžina tik `GameMetadata`,
> be `medias`), todėl pats daro tiesioginį `jeuInfos.php` kvietimą IR jokiu būdu neužrašo
> gauto media URL kaip konstantos — jis egzistuoja TIK vykdymo metu.
>
> **Realiai patikrinta:** `real_cover_downloads_from_live_screenscraper_response` — realus
> `jeuInfos.php` kvietimas (Super Metroid), realūs `medias` (visi keturi tipai atsakyme buvo),
> `download_game_media()` sėkmingai atsisiuntė visus keturis (`covers/999999.png`,
> `screenshots/999999.png`, `wheels/999999.png`, `videos/999999.mp4`), be `.tmp` liekanų.
> 13 greitų unit testų (regionų prioritetas, `video-normalized` vs `video`, plėtinio
> nustatymas iš `format`/URL/numatytosios reikšmės, dydžio limito viršijimas, egzistuojančio
> failo praleidimas) naudoja vietinį HTTP serverį (`tokio::net::TcpListener`) — NE realų
> ScreenScraper hostą, kad testai liktų greiti ir neišeikvotų kvotos.

**Ką daryti:**
- Atsisiųsk `box-2D`, `ss`, `wheel`, `video-normalized` (fallback `video`)
- Saugok į `media_dir()` pagal `CLAUDE.md` §9.4 struktūrą
- DB laiko **santykinius** kelius
- Praleisk, jei failas jau egzistuoja ir dydis > 0
- Video dydžio limitas (numatytasis 10 MB) — didesnius praleisk

**Acceptance:**
- [x] Viršeliai ir video atsisiunčia — patikrinta REALIU API kvietimu (visi 4 tipai)
- [x] Nutrūkęs atsisiuntimas nepalieka sugadinto failo (rašyk į `.tmp`, tada `rename`) —
      `oversized_media_is_skipped_and_leaves_no_files` patvirtina: nei `.tmp`, nei galutinis
      failas neišlieka
- [x] Pakartotinis scraping'as nesiunčia to paties dar kartą —
      `existing_nonempty_file_is_skipped_without_new_request` (nepasiekiamas URL + nepakitęs
      turinys įrodo, kad tinklas neliestas)

---

### P6.4 — Scraping orkestracija `[x]`

**Priklausomybės:** P6.3
**Failai:** `crates/nullbyte-app/src/commands/scraper.rs`

> **Įgyvendinta 2026-08-25.** Tikroji orkestracijos logika gyvena `crates/nullbyte-app/src/
> scraper/mod.rs` (`scrape_single_game`/`scrape_pending_games`), NE `commands/scraper.rs` —
> tas pats sluoksniavimas kaip `library::scanner`/`commands::library`: domeno modulis lieka
> Tauri-nepriklausomas (`on_progress: impl FnMut`, ne `Channel`), kad būtų testuojamas be
> `tauri::test` scaffolding'o; `commands/scraper.rs` yra PLONAS `Channel`/`CancellationToken`
> laidas aplink jį (CLAUDE.md §6.3).
>
> `GameMetadata` (P6.2) papildyta `medias: Vec<Media>` lauku — `screenscraper::lookup_game`
> anksčiau IŠMESDAVO `jeu.medias`, nes grąžindavo tik išvalytus tekstinius laukus. Kadangi
> `GameMetadata` JAU keliauja per `scrape_cache.response` JSON (P6.2), pridėjus `medias`
> ten pat, cache'uotas atsakymas gali PAKARTOTINAI atsisiųsti media be naujo gyvo API
> kvietimo — `Media` gavo `Serialize` (anksčiau turėjo tik `Deserialize`).
>
> **Kvotos išnaudojimo signalas:** `scrape_one_game` grąžina `Err` TIK kai PATI
> `rate_limit::cached_lookup` nepavyksta (t.y. jau išnaudotas visas backoff'as) — tai
> kviečiančiajam (`scrape_pending_games`) reiškia „sustabdyk VISĄ eilę švariai", skirtingai
> nuo bet kurios KITOS to VIENO žaidimo problemos (nepalaikoma platforma, media/DB klaida),
> kuri pažymi TIK tą žaidimą ir tęsia toliau. Ši distinkcija — vienintelis būdas atskirti
> „ScreenScraper visai nebeatsako" nuo „šis vienas žaidimas turi problemą" be papildomo
> tipo `LookupError` viduje (kuris jau atskiria juos P6.2 sluoksnyje).
>
> **Atšaukimas veikia net vidury backoff'o laukimo**, ne tik tarp žaidimų — `tokio::select!`
> lenktyniauja `cancel.cancelled()` prieš PATĮ `scrape_one_game` Future'ą, ne tik tikrina
> `is_cancelled()` ciklo pradžioje.
>
> **SĄMONINGAI NEGENERALIZUOTA per injektuojamą `fetch`** (skirtingai nuo `rate_limit::
> cached_lookup`) — `scrape_one_game` visada kviečia TIKRĄ `screenscraper::lookup_game`.
> Generalizavimas šiame sluoksnyje reikalautų HRTB (`for<'r> Fn(&'r RomIdentity<'r>) -> Fut`,
> kuris konfliktuoja su vienu fiksuotu `Fut` tipo parametru) arba `Box<dyn Fn(...) ->
> Pin<Box<dyn Future>>>` dinaminio dispatch'o — abu neproporcingai sudėtingi, kai
> `cached_lookup` savo ruožtu JAU pilnai unit-testuota (P6.2) su injektuotu `fetch`. Vietoj
> to šis sluoksnis testuojamas: greitai (pure funkcijos `rom_filename`, DB funkcijos
> `set_scrape_status`/`apply_scrape_result`, `Unsupported`/atšaukimo šakos be tinklo) IR
> gyvai (žr. žemiau).
>
> Nauja priklausomybė: `tokio-util = "0.7"` (`CancellationToken`) — jau buvo tranzityvi per
> `tauri`, bet reikėjo tiesioginio Cargo.toml įrašo, kad `use` kompiliuotųsi.
>
> **Realiai patikrinta TRIMIS lygmenimis:**
> 1. `real_scrape_single_game_updates_db_row` — pilnas vieno žaidimo srautas (paieška → media
>    atsisiuntimas → DB rašymas), realus API, realūs 4 media tipai, DB eilutė patikrinta PO
>    scraping'o.
> 2. `real_scrape_library_processes_90_real_games` — **TIESIOGINIS P6.4 acceptance
>    patikrinimas** (ne ekstrapoliacija): TIKRAS `library::scanner::scan()` +
>    `scrape_pending_games()` prieš REALIUS fixture ROM'us
>    (`crates/nullbyte-core/roms/{snes,megadrive,gba,psx}/`). Rezultatas: 69 žaidimai
>    nuskenuoti (26 praleisti dėl nepalaikomų plėtinių — laukta, PSX `.bin`/`.cue`
>    palydovinių failų), **65 rasti, 4 nerasti, 0 klaidų**, progreso pranešimų kiekis
>    TIKSLIAI sutapo su apdorotų žaidimų kiekiu (69), `cancelled: false`. Trukmė ~6 min.
>    (`maxthreads=1`, tad nuosekliai).
> 3. `second_lookup_with_same_key_hits_cache_and_skips_fetch` (P6.2, jau egzistavęs) +
>    `unsupported_platform_short_circuits_without_network`/
>    `cancellation_before_start_processes_nothing` (P6.4, nauji) — greiti testai be tinklo.
>
> **Sąžiningas apribojimas:** „Kvotos pabaiga sustabdo švariai" verifikuota TIK sintetiniu
> testu (P6.2 `persistent_rate_limit_gives_up_with_clear_error_after_all_retries` +
> distinkcijos logika `scrape_one_game`'e) — realaus kvotos išnaudojimo simuliuoti
> negalima neeikvojant TIKROS vartotojo dienos kvotos (žr. tą pačią pastabą P6.2 skiltyje).

**Ką daryti:**
- `scrape_game(id)` — vienas žaidimas
- `scrape_library(platform_id?)` — visi `scrape_status = 'pending'`
- Progresas per `Channel<ScrapeProgress { current, total, title, status, quota_left }>`
- Atšaukimas (`CancellationToken`)
- **Niekada automatiškai** — tik vartotojui paspaudus

**Acceptance:**
- [x] 50 žaidimų scraping'as baigiasi be klaidų — REALIAI patikrinta: 69 žaidimai, 0 klaidų
      (žr. aukščiau)
- [x] Progresas realiu laiku — kiekvienas žaidimas siunčia `ScrapeProgress` iškart po
      apdorojimo (ne po viso batch'o); realiame teste 69 pranešimai = 69 apdoroti žaidimai
- [x] Atšaukimas veikia iškart — `tokio::select!` prieš patį scraping'o Future'ą (žr.
      aukščiau), patikrinta `cancellation_before_start_processes_nothing`
- [x] Kvotos pabaiga sustabdo švariai — `scrape_one_game`'o `Err` grąžinimo distinkcija +
      `AppError::Other` su aiškiu pranešimu; sintetiniu testu, NE gyvu (žr. sąžiningą
      apribojimą aukščiau)

> **Milestone M4:** biblioteka pilna su metaduomenimis.

---

## 9. Faza 7 — UI

**Tikslas:** tikras, gražus, naudojamas interfeisas.
**Rizika:** 🟢 maža. **Įvertis:** 4–5 dienos.

### P7.1 — Layout ir navigacija `[x]`

**Priklausomybės:** P0.2, P5.4
**Failai:** `src/routes/+layout.svelte`, `src/lib/components/layout/*`

**Ką daryti:**
- Sidebar: platformų sąrašas su žaidimų kiekiu, „Visi", „Mėgstami", „Neseniai žaisti"
- TopBar: paieška, rūšiavimo pasirinkimas, nustatymų mygtukas
- Command palette (`Cmd/Ctrl+K`) — shadcn `command` komponentas
- Tamsi tema, tanki, klaviatūra naviguojama

**Įgyvendinta:** `src/lib/api/index.ts` (tipizuoti `invoke` wrapper'iai), `src/lib/stores/library.svelte.ts`
(Svelte 5 runes store — `platforms`, `games`, `filter`, dalinamasi Sidebar/TopBar/CommandPalette/
`+page.svelte`), `src/lib/components/layout/{Sidebar,TopBar,CommandPalette}.svelte`. Nustatymų
mygtukas TopBar'e sąmoningai `disabled` su tooltip „netrukus" — reali funkcija P7.6. `+page.svelte`
laikinai rodo paprastą sąrašą (P7.2 pakeis virtualizuotu grid'u su viršeliais).

**Pastaba dėl `eslint.config.js`:** `eslint-plugin-svelte` priverstinai naudoja `svelte-eslint-parser`
`*.svelte.ts`/`*.svelte.js` failams (Svelte 5 runes store'ams), bet be papildomo delegavimo į
`typescript-eslint` parserį jis lūžta ant `import type { ... }`. Pridėtas naujas override
(`files: ["**/*.svelte.ts", "**/*.svelte.js"]` → `parserOptions.parser: ts.parser`) — reikalingas
visiems būsimiems `.svelte.ts` store'ams (`emulator.svelte.ts`, `settings.svelte.ts`).

**Acceptance:**
- [x] Sidebar rodo tikras platformas iš DB — `list_platforms` sukviečiamas `+layout.svelte`
      `onMount`; REALIAI patikrinta vartotojo `pnpm tauri dev` sesijoje 2026-08-26 — platformų
      sąrašas rodomas teisingai (žr. P7.2 pastabą — pirminis agento sesijos patikrinimas buvo
      klaidingai neigiamas dėl agento aplinkos apribojimo, ne kodo klaidos, žr. ADR-017)
- [x] `Cmd+K` atidaro paletę — patikrinta naršyklėje (Chrome DevTools automation), veikia
- [x] Klaviatūros navigacija veikia (Tab, strėlės) — patikrinta REALIAI: `Tab` perkelia fokusą per
      Sidebar mygtukus DOM tvarka (BODY → Visi → Mėgstami → Neseniai žaisti), `ArrowDown` paletėje
      perkelia highlight'ą per punktus (bits-ui `Command` primityvas)

---

### P7.2 — Žaidimų grid'as ir kortelės `[x]` (pilnai patvirtinta su realiais duomenimis, įsk. viršelius)

**Priklausomybės:** P7.1
**Failai:** `src/lib/components/library/GameCard.svelte`, `GameGrid.svelte`,
`src/lib/utils/platforms.ts`, `src/lib/stores/app.svelte.ts` (naujas — `mediaDir` viršeliams),
`src/lib/api/index.ts` (`getAppInfo`)

**Ką daryti:**
- Responsive grid, viršelio proporcijos, `Skeleton` kol kraunasi
- Viršeliai per `convertFileSrc()` (asset protokolas), ne base64
- **Virtualizacija** (`@tanstack/svelte-virtual`) — privaloma
- Placeholder žaidimams be viršelio (platformos spalva + pavadinimas)
- Hover: pakilimas, šešėlis, pavadinimo overlay

**Įgyvendinta:** `GameGrid.svelte` virtualizuoja EILUTES (ne pavienes korteles) —
kolonų skaičius skaičiuojamas iš `bind:clientWidth`, `$effect` sinchronizuoja
virtualizatoriaus `count`/`estimateSize` (žr. ADR-017). Įjungtas Tauri `assetProtocol`
(anksčiau visai nekonfigūruotas — žr. ADR-017), be jo `convertFileSrc()` viršeliai
NIEKADA nebūtų pasiekiami. Placeholder — platformos `chart-*` spalva (CLAUDE.md §7.5, jokių
hardcode'intų hex) + pavadinimas.

**Patikrinimo istorija (ATNAUJINTA 2026-08-26, žr. ADR-019):** iš pradžių maniau, kad agento
automatizuota sesija tiesiog negali paspausti mygtukų (žr. senesnę ADR-017 nuorodą žemiau) —
TAI BUVO IŠ DALIES KLAIDINGA IŠVADA. Kai vartotojas realiai paskenavo 30 SNES ROM'ų (P7.5),
visa aplikacija užstrigo (jokie paspaudimai/`Cmd+K` nebeveikė) — tikroji priežastis buvo
begalinė reaktyvi kilpa `GameGrid.svelte` (žr. ADR-019 pilnai), kuri egzistavo NUO ŠIOS P7.2
užduoties, tiesiog nebuvo suveikusi anksčiau. Ištaisyta ir PATVIRTINTA vartotojo REALIOJE
sesijoje 2026-08-26: Sidebar platformų sąrašas teisingas, `Cmd+K`/Favorites/Settings
navigacija veikia, 30 realių SNES žaidimų (ActRaiser, Chrono Trigger, Donkey Kong Country...)
atvaizduoti grid'e su teisingais pavadinimais IR tvarkingu placeholder'iu (nė vienas dar
neturi viršelio, nes scraping'as nedarytas), grid'as slenka sklandžiai.

**Toliau scrape'inus 88 žaidimus (4 platformos) paaiškėjo, kad fiksuota `aspect-[3/4]` dėžė
apkerpa viršelius su kitokia proporcija (PSX kvadratas) — žr. ADR-021 pilnam sprendimui.**
`GameGrid` perrašytas į „packed row" layout'ą (fiksuota aukštis, tikras plotis pagal REALIUS
DB'je saugomus `cover_width`/`cover_height`, ADR-021), `GameCard` nebeturi savo fiksuotos
proporcijos. Patikrinta screenshot'ais — PSX kvadratiniai, SNES platūs, Genesis aukšti, visi
BE apkirpimo.

**Kas DAR nepatikrinta:** pilnas 5000 žaidimų 60 FPS testas (patvirtinta sklandu su 88 realiais
žaidimais, bet ne su tokiu masteliu).

**Acceptance:**
- [x] Sidebar rodo tikras platformas iš DB, IPC/paspaudimai veikia — patvirtinta vartotojo
      2026-08-26 (PO begalinės kilpos pataisymo, žr. ADR-019)
- [x] Be viršelio — tvarkingas placeholder — patvirtinta vartotojo 2026-08-26
- [x] Viršeliai rodomi — patvirtinta 2026-08-26 su 88 realiai scrape'intais žaidimais (4
      platformos), įsk. „packed row" pataisymą skirtingoms proporcijoms (ADR-021)
- [ ] 5000 žaidimų grid'as slenka 60 FPS — patvirtinta sklandu su 88 realiais žaidimais,
      pilnas 5000 mastelio testas dar nedarytas

---

### P7.3 — Video preview 🟡 `[x]` (kodas baigtas, ⚠️ dar nepatikrinta su realiu video)

**Priklausomybės:** P7.2, P6.3
**Failai:** `src/lib/components/library/VideoPreview.svelte`,
`src/lib/stores/videoPreview.svelte.ts` (naujas)

**Ką daryti:**
- Hover 300 ms → prasideda video (`muted`, `loop`, `playsinline`, `preload="none"`)
- Fade-in perėjimas nuo viršelio prie video
- Mouse leave → sustabdyk, `currentTime = 0`, atlaisvink
- **Vienu metu groja tik VIENAS video** — globalus „aktyvus preview" būvis
- `$effect` cleanup privalomas (kitaip liks groti fone)

**Įgyvendinta:** `videoPreview.svelte.ts` — vienintelis `activeGameId` laukas (singleton) visai
bibliotekai; `VideoPreview.svelte` pati valdo hover debounce (`setTimeout` 300 ms, `clearTimeout`
paleidimas iš karto pele išėjus — greitas slinkimas niekada nepasiekia 300 ms), `<video>`
elementas egzistuoja TIK kol `active` (Svelte `{#if}`) — kadangi `activeGameId` yra vienas
globalus laukas, naujos kortelės aktyvavimas automatiškai išjungia seną (jos `{#if}` tampa
`false`, elementas sunaikinamas). `$effect` grąžina cleanup funkciją, kuri `pause()` +
`currentTime = 0` + `removeAttribute("src")` + `load()` — tikrai atlaisvina dekoderį, ne tik
sustabdo. Integruota į `GameCard.svelte` kaip absoliučiai pozicionuotas hover-catcher sluoksnis
tarp viršelio ir apatinio gradiento.

**⚠️ Patikrinimo apribojimas:** biblioteka realiai dar tuščia (joks ROM katalogas
nenuskenuotas, joks scraping'as nepaleistas) — kodas kompiliuojasi, `pnpm check`/`lint`/`build`
švarūs, bet debounce/single-video/atminties acceptance punktų su REALIU video failu dar
niekas nepatikrino. Tas pats apribojimas kaip P7.2 viršeliams — natūraliai patikrinama kartu.

**Acceptance:**
- [ ] Greitai slenkant pele video nepradeda groti (debounce veikia) — reikia realių duomenų
- [ ] Niekada negroja 2 video vienu metu — reikia realių duomenų
- [ ] Atminties naudojimas nekyla slenkant per 100 kortelių — reikia realių duomenų

---

### P7.4 — Žaidimo detalių puslapis `[x]` (kodas baigtas, ⚠️ dar nepatikrinta su realiais duomenimis)

**Priklausomybės:** P7.2
**Failai:** `src/routes/game/[id]/+page.svelte`, `src/routes/game/[id]/+page.ts` (naujas —
`prerender = false`), `src/lib/utils/format.ts` (naujas), `src/lib/api/index.ts` (`scrapeGame`)

**Ką daryti:**
- Hero: screenshot fone (blur + gradientas), wheel logotipas viršuje
- Metaduomenys: aprašymas, kūrėjas, leidėjas, data, žanras, žaidėjų kiekis, reitingas
- Mygtukai: „Žaisti", „Mėgstamas", „Scrape iš naujo"
- Save states sąrašas su preview paveiksliukais
- Statistika: paskutinį kartą žaista, kiek kartų, kiek laiko

**Įgyvendinta:** `GameCard` dabar `<a href={resolve("/game/[id]", ...)}>` — visas grid'as
naviguojamas. „Mėgstamas" ir „Scrape iš naujo" — REALIAI veikiantys mygtukai (`set_favorite`,
`scrape_game` per Tauri `Channel<ScrapeProgress>`, abu jau parašyti P5.4/P6.4). „Žaisti" —
**nuo P9.1 REALIAI veikia** (`startGame`, žr. ADR-030): rodo „Launching..." kol laukia
backend'o atsakymo, „Playing" kai patvirtinta, klaidos pranešimą (`text-destructive`), jei
`start_game` grąžina `Err`. Klausosi `"game-closed"` Tauri event'o, kad atnaujintų statistiką
grįžus iš žaidimo. Save states sekcija — tuščios būsenos placeholder'is („coming in P8.1"),
nes joks `commands::` sluoksnis tam dar neegzistuoja (P8.1 core mechanizmas baigtas, bet be
Tauri komandų — žr. P8.1 pastabą) — sąmoningai NEBUVO kurta pilna UI be duomenų šaltinio
(CLAUDE.md „nekurk pusiau baigtų implementacijų"). Trūkstami duomenys (aprašymas, kūrėjas ir
t.t.) tvarkomi filtruojant `null`/tuščias reikšmes iš `metaRows` prieš atvaizduojant — layout
nelūžta, tiesiog tas badge'as nerodomas.

**⚠️ Patikrinimo apribojimas:** `pnpm check`/`lint`/`build` švarūs (įskaitant prerender
sprendimą dinaminiam maršrutui — SPA fallback per `adapter-static`), REALIAI paleista
`pnpm tauri dev`, biblioteka rodo teisingai be regresijos. BET pats detalių puslapis (mygtukai,
metaduomenys, hero) dar nepatikrintas su realiu žaidimu, nes biblioteka tuščia — tas pats
apribojimas kaip P7.2/P7.3, natūraliai išsispręs kartu su P7.5 (skenavimo UI).

**Acceptance:**
- [ ] Visi metaduomenys rodomi — reikia realių duomenų
- [x] „Žaisti" paleidžia žaidimą — patikrinta REALIAI vartotojo (P9.1, žr. ADR-030)
- [ ] Trūkstami duomenys nesulaužo layout'o — kodas tam paruoštas (filtruoja `null`), reikia
      realaus patikrinimo

---

### P7.5 — Skenavimo ir scraping'o UI `[x]` (patvirtinta REALIU skenavimu — 30 SNES žaidimų)

**Priklausomybės:** P5.3, P6.4, P7.1
**Failai:** `src/lib/components/settings/PathsPanel.svelte`, `src/routes/settings/+page.svelte`
(naujas, minimalus — pilnas ekranas P7.6), `src/lib/components/ui/progress/*` (shadcn),
Rust: `db/rom_directories.rs` (naujas), 4 naujos komandos `commands/library.rs`

**Ką daryti:**
- ROM katalogų pridėjimas per Tauri dialog plugin
- „Skenuoti" mygtukas → progreso juosta su dabartiniu failu
- „Scrape library" → progresas + kvotos likutis
- Atšaukimo mygtukas
- Rezultatų santrauka (rasta / nerasta / klaidos)

**Įgyvendinta:** žr. ADR-018 pilnam sprendimų sąrašui (nauja `tauri-plugin-dialog`
priklausomybė, nauja `db/rom_directories.rs` CRUD, `scan_library` komanda apvyniojanti jau
parašytą P5.3 `scanner::scan()` į `Channel<ScanProgress>`, minimalus `/settings` maršrutas kad
panelė būtų pasiekiama). Rezultatų santrauka rodoma po skenavimo/scraping'o (added/updated/
removed/unchanged; found/notFound/errored). Atšaukimas — perpanaudoja P6.4 `cancel_scrape`.

**Patikrinta REALIAI 2026-08-26:** vartotojas atidarė Nustatymus, pridėjo
`crates/nullbyte-core/roms/snes` per katalogo pasirinkimo dialogą, paspaudė „Scan library" —
30 SNES žaidimų (ActRaiser, Chrono Trigger, Contra III, Donkey Kong Country 1/2, EarthBound,
F-Zero, Final Fantasy IV/VI, Zelda ALTTP, ...) atsirado bibliotekoje su teisingais, švariai
išvalytais pavadinimais (P5.3 `clean_title`) po skenavimo. Tai VISO pipeline'o (dialog plugin →
`add_rom_directory` → `scan_library` → `scanner::scan()` → `list_games`) realus patvirtinimas.

Skenavimo METU aptikta IR IŠTAISYTA rimta klaida — žr. ADR-019 (begalinė kilpa
`GameGrid.svelte`, sukėlusi VISOS aplikacijos „užstrigimą" po skenavimo). Papildomai, vartotojo
pastebėjimu, ištaisyta UX spraga: Sidebar/`Cmd+K` pasirinkimai (platforma/Favorites/...)
ANKSČIAU tik keisdavo filtrą, bet neišvesdavo iš Settings/žaidimo puslapio atgal į biblioteką —
dabar `Sidebar.svelte`/`CommandPalette.svelte` visada iškviečia `goto(resolve("/"))` po
pasirinkimo.

**Toliau testuojant su Genesis/GBA/PSX/MAME katalogais rasta IR IŠTAISYTA dar viena reali
klaida** — žr. ADR-020: GBA rodė 0 žaidimų (trūko `zip`/`7z` iš leidžiamų plėtinių), o PSX
žaidimai atsidurdavo po „Sega CD" (dviprasmybė tarp platformų, dalinančių tuos pačius archyvo
vidinius plėtinius). Sprendimas — naujas `rom_directories.platform_id` hint'as, leidžiantis
vartotojui eksplicitiškai nurodyti katalogo platformą. Galutinis REALUS rezultatas: 88 žaidimai
teisingai — 30 SNES + 35 Genesis + 20 GBA + 3 Sony PlayStation (ne Sega CD).

**P7.6 Paths panelės dokumentavimo metu (2026-08-26) rastas IR IŠTAISYTAS dar vienas tos
pačios klasės latentinis bug'as** — žr. ADR-023: Neo Geo neteko `zip`/`7z` iš plėtinių sąrašo
002 migracijoje, tad suarchyvuoti Neo Geo ROM'ai (dažniausias platinimo formatas) VISAI
neatpažįstami. Pataisyta migracija 007. Arcade turi TĄ PATĮ simptomą (0 rezultatų), bet KITĄ
priežastį (MAME romsetai neturi vieno atpažįstamo failo archyve) — SĄMONINGAI NEpataisyta,
reikalauja naujos `extract_first_match` logikos, žymima kaip žinomas apribojimas.

**Acceptance:**
- [x] Katalogo pridėjimas veikia — patvirtinta vartotojo 2026-08-26 penkiais skirtingais
      katalogais (macOS; Linux dar netestuota nė vienoje sesijoje, žr. §11.5 apribojimą)
- [x] Progresas sklandus, be UI užšalimo — progreso mechanizmas PATS veikia teisingai
      (Channel siunčia atnaujinimus, UI juos rodo). ⚠️ ŽINOMAS apribojimas (žr. ADR-020,
      sąmoningai NEpataisyta vartotojo sprendimu): su labai dideliais failais (400+ MB PSX
      archyvais) `scan_library` laiko `state.db` Mutex užrakintą per visą hash'avimo trukmę,
      todėl KITI veiksmai (ne pats skenavimas) tampa laikinai neatsakantys — nekritiška
      įprasto dydžio ROM'ams (SNES/Genesis/GBA — KB–kelių MB)
- [ ] Atšaukimas veikia — dar nepatikrinta interaktyviai (perpanaudoja jau patikrintą P6.4
      `cancel_scrape`, tad rizika maža)

---

### P7.6 — Nustatymų ekranas `[x]` (visos 6 kortelės įgyvendintos — žr. progreso pastabą)

**Priklausomybės:** P7.1, P4.2
**Failai:** `src/routes/settings/+page.svelte`, `src/lib/components/settings/*`

**Ką daryti:**
- **Keliai:** ROM katalogai, core'ų katalogas, BIOS katalogas
- **Core'ai:** aptiktų core'ų sąrašas, pasirinkimas per platformą
- **Vaizdas:** filtras (nearest/linear), integer scaling, vsync, fullscreen numatytasis
- **Garsas:** įrenginys, garsumas, buferio dydis
- **Įvestis:** valdiklių sąrašas, mygtukų perrišimas (spaudi mygtuką → priskiria)
- **Scraper:** ScreenScraper login, regionų prioritetas, media tipai, kvotos likutis

**Progresas (2026-08-26):** ekranas dabar tabuotas (`Tabs.Root`, 6 kortelės: Paths/Cores/
Video/Audio/Input/Scraper). **Paths** — jau anksčiau P7.5 metu pilnai veikiantis (perpanaudotas
`PathsPanel.svelte`). **Scraper** — dabar pilnai įgyvendintas (žr. ADR-022): credentials
redaguojami UI (`settings` DB lentelė turi pirmenybę prieš `.env`), rodomas maskuotas `devid`,
paskutinė žinoma kvota (`AppState.last_quota`, atnaujinama scrape'o metu — sąmoningai NE gyvu
API kvietimu vien atidarius ekraną), regionų prioritetas/media tipai READ-ONLY info. **Input** —
UI + DB persistencija įgyvendinta (`InputPanel.svelte`, naujos komandos
`get_input_mapping`/`set_input_mapping`/`reset_input_mapping`, `settings` lentelės raktas
`input.mapping`, JSON masyvas), bet **SĄMONINGAI dar NEVEIKIA realiame žaidime** — žr. detalų
paaiškinimą po šia pastaba. **Cores** — dabar įgyvendinta (žr. ADR-024): aptiktų core'ų sąrašo
rodymas PILNAI FUNKCIONALUS (perpanaudoja P1.3 `nullbyte_core::core::info::scan_cores_dir` be
pakeitimų — jokio P9.1 bloko), preferuojamo core'o pasirinkimas per platformą — TIK
persistencija, tas pats P9.1 apribojimas kaip Input. **Video** — filtras (`nearest`/`linear`)
ir scaling (`fit`/`integer`) TIK persistencija, bet Rust pusėje `Renderer::set_filter`/
`set_scale_mode` JAU EGZISTUOJA (P2.5) — trūksta TIK P9.1 IPC wiring'o, ne naujo domeno kodo;
vsync/start-fullscreen TIK persistencija, jokio esamo runtime hook'o net Rust pusėje. **Audio**
— įrenginio sąrašo rodymas PILNAI FUNKCIONALUS (nauja `cpal` enumeracija tiesiai
`nullbyte-app` pusėje, žr. pastabą žemiau), bet pats pasirinkimas + garsumas + buferio dydis
TIK persistencija — jokio esamo mechanizmo `audio/output.rs` juos pritaikyti.

**Input panelės apribojimas (aptikta PRIEŠ rašant kodą, aptarta su vartotoju):** mygtukų/
klavišų mapping'as šiuo metu yra HARDKODINTAS `nullbyte-emu/src/main.rs` (vaiko procese), be
jokio DB saugojimo ir be jokio IPC kanalo jam perduoti — o realus žaidimo paleidimo srautas
(`nullbyte-app` → `nullbyte-emu` per `EmuClient`) yra P9.1, DAR NEĮGYVENDINTA. Todėl UI leidžia
vartotojui iš anksto susikonfigūruoti norimą mapping'ą (klaviatūros klavišas per `keydown`
capture, gamepad mygtukas per naršyklės Gamepad API poll'inimą su `requestAnimationFrame`),
bet pasirinkimas kol kas TIK persistuojamas `settings` lentelėje — jis nepaveikia jokio
realaus žaidimo, kol P9.1 nepastatys vamzdyno, sujungiančio šį pasirinkimą su vaiko procesu
(naujas `EmuCommand::SetMapping` ar panašus). Panelėje rodomas aiškus perspėjimo tekstas apie
tai. Vartotojas EKSPLICITIŠKAI pasirinko šį apribotą apimties variantą vietoj laukimo iki P9.1
arba gilesnio engine pakeitimo dabar.

**Žinomas apribojimas — viena BENDRA mapping lentelė visiems žaidėjams (aptarta su
vartotoju 2026-08-26):** `nullbyte-emu` (P4.3) jau palaiko iki 4 gamepad portų — kiekvienas
prisijungęs valdiklis automatiškai gauna kitą laisvą portą prisijungimo eile
(`gamepad_ports: HashMap<usize, usize>`), tad **2-4 žaidėjų co-op su atskirais fiziniais
valdikliais VEIKS** (kai bus P9.1) be jokio papildomo darbo. Bet `default_gamepad_mapping`
yra VIENA FIZINĖS POZICIJOS lentelė, taikoma VISIEMS portams vienodai — nėra per-port
override'o, tad visi žaidėjai priversti naudoti TĄ PATĮ „fizinis mygtukas → RetroPad veiksmas"
išdėstymą (praktiškai nekritiška — tai standartinis RetroArch elgesys co-op žaidimuose).
Papildomai, **klaviatūra visada valdo TIK portą 0** (žaidėjas 1) — antro žaidėjo klaviatūros
mapping'o NĖRA ir neplanuojama šiame etape. Jei kada prireiks per-žaidėjo individualaus
mapping'o — reikėtų `InputBinding` papildyti `port` lauku ir DB raktą
(`input.mapping.port{N}`) vietoj vieno `input.mapping`. Vartotojas sąmoningai pasirinko
NEDARYTI šio išplėtimo dabar (žr. pokalbį) — bendra lentelė pakanka realiam co-op scenarijui.

**Cores panelė — daliai funkcijos NĖRA P9.1 bloko:** aptiktų core'ų sąrašo rodymas
(`list_cores` komanda) tiesiog perpanaudoja JAU PARAŠYTĄ ir testuotą P1.3
`nullbyte_core::core::info::scan_cores_dir` — jokio naujo domeno kodo, tik plonas DTO
suplokštinimas (`CoreInfoDto`, `PathBuf` → `String`). Tai VEIKIA ŠIANDIEN, be jokio
apribojimo — vartotojas iš karto mato, kokie core'ai realiai rasti `cores_dir` kataloge, su
pavadinimu/versija/sistema/palaikomais plėtiniais. Tuščias katalogas → tuščias sąrašas (NE
klaida) su nuoroda į tikslų `cores_dir` kelią (iš `get_app_info`). Preferuojamo core'o
pasirinkimas PER PLATFORMĄ (naujos komandos `get_preferred_cores`/`set_preferred_cores`,
`settings` raktas `core.preferred`, `Vec<PlatformCorePreference>` JSON) turi TĄ PATĮ P9.1
apribojimą kaip Input mapping'as — išsaugoma, bet niekas dar realiai nepaleidžia žaidimo su
pasirinktu core'u.

**Video/Audio panelės — kokybiškai SKIRTINGI P9.1 apribojimai kiekvienam laukui (tyrimas
subagent'u prieš rašant kodą, 2026-08-26):**
- `filter`/`scaleMode` (Video) — VIENINTELIAI iš visų šešių Video/Audio laukų, kuriems Rust
  pusėje JAU EGZISTUOJA veikiantis mechanizmas: `FilterMode`/`ScaleMode` enum'ai su
  `Renderer::set_filter()`/`set_scale_mode()` (P2.5, `renderer.rs`). Serializuojamos reikšmės
  (`"nearest"|"linear"`, `"fit"|"integer"`) TIKSLIAI atitinka enum variantus (testas
  `video_settings_default_matches_renderer_defaults` tai apsaugo), kad P9.1 wiring metu
  reikėtų tik naujo `EmuCommand` varianto, NE reikšmių konvertavimo.
- `vsync`/`startFullscreen` (Video) — NĖRA JOKIO esamo runtime hook'o net Rust pusėje.
  Vsync „baked" į `wgpu::SurfaceConfiguration` `Renderer::new()` metu (`present_mode:
  wgpu::PresentMode::AutoVsync`, hardkodinta) — pakeitimui reikėtų naujo
  `set_present_mode()`, panašaus į `resize()`. Fullscreen šiuo metu tik F11 runtime toggle
  (`nullbyte-emu/src/main.rs`), jokio „pradėti fullscreen" atributo lango kūrimo metu nėra.
- `device`/`volume`/`bufferMs` (Audio) — VISI TRYS reikalautų NAUJO kodo `audio/output.rs`
  (`nullbyte-core`), NE vien P9.1 IPC — dabar visada `host.default_output_device()`
  (hardkodinta), garsumas apskritai neegzistuoja kaip konceptas (sample'ai keliauja
  nekeisti), buferio dydis — hardkodinta konstanta `TARGET_LATENCY_MS = 50`, susieta su
  `ring::recommended_capacity` skaičiavimu.
- **Įrenginių sąrašo rodymas (Audio) — IŠIMTIS, veikia ŠIANDIEN:** `cpal` enumeracija
  (`host.output_devices()`) yra grynas OS užklausimas, nepriklausantis nuo jokio aktyvaus
  garso srauto ar `nullbyte-emu` vaiko proceso — saugu kviesti tiesiai iš `nullbyte-app`.
  Naujas TIESIOGINIS `cpal` priklausomumas `crates/nullbyte-app/Cargo.toml` (jau buvo
  workspace priklausomybė per `nullbyte-core`, čia tik nauja naudojimo vieta).

**Acceptance:**
- [x] Visi nustatymai išsaugomi DB ir taikomi — DALINIAI TAIKOMA (kaip ir buvo tikėtasi prieš
      P9.1): scraper credentials PILNAI (žr. ADR-022); core'ų sąrašo IR audio įrenginių sąrašo
      RODYMAS PILNAI (skaito realų `cores_dir`/OS turinį, jokio DB saugojimo tam nereikia);
      visi likę pasirinkimai (input mapping, preferuojamas core'as, video filter/scale/vsync/
      fullscreen, audio device/volume/buffer) IŠSAUGOMI, bet DAR NETAIKOMI — blokuoja P9.1
      (dalis) arba trūkstamas engine kodas (dalis, žr. pastabą aukščiau) — tai ŽINOMAS,
      dokumentuotas, vartotojo priimtas apribojimas šiam MVP etapui, ne praleista klaida
- [x] Mygtukų perrišimas veikia — UI VEIKIA (persistuoja, su automatiniu rekomenduojamo
      core'o priskyrimu Cores panelėje), bet realiame žaidime NETAIKOMA (blokuoja P9.1, žr.
      pastabą aukščiau) — tas pats žinomas apribojimas
- [x] Neteisingi ScreenScraper credentials duoda aiškią klaidą — `ScreenScraperCredentials::load`
      grąžina `AppError::Other` su aiškiu tekstu, kai nei DB, nei `.env` neturi `devid`/
      `devpassword`; `set_scraper_credentials` atmeta tuščius privalomus laukus prieš įrašymą.
      UI (`ScraperPanel.svelte`) rodo klaidos tekstą tiesiai formoje.

---

## 10. Faza 8 — Išsaugojimai

**Tikslas:** progresas neprapuola.
**Rizika:** 🟡 vidutinė. **Įvertis:** 1–2 dienos.

### P8.1 — Save states `[!]`

**Priklausomybės:** P1.7, P5.1
**Failai:** `crates/nullbyte-core/src/core/savestate.rs`, `crates/nullbyte-core/src/video/png_encoder.rs`,
`crates/nullbyte-core/src/core/loader.rs`, `crates/nullbyte-core/src/core/runner.rs`,
`crates/nullbyte-core/src/ipc.rs`, `crates/nullbyte-app/src/db/save_states.rs`

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

> **Pastaba (ADR-028, 2026-08-26):** core mechanizmas (serializacija, atominis įrašymas,
> PNG preview, IPC laidas, DB CRUD) pilnai įgyvendintas ir testuotas — žr. ADR-028. TRŪKSTA:
> `commands::` Tauri sluoksnio (nėra kviečiančiojo — laukia P9.1 paleidimo pipeline'o, ta pati
> situacija kaip `db/rom_directories.rs` prieš P7.5), realaus end-to-end hotkey testo per
> gyvą procesą, core-mismatch įspėjimo logikos (reikalauja `commands::` palyginti
> `save_states.core_name`/`core_version` prieš `LoadState`).

**Acceptance:**
- [x] Save → uždaryti → paleisti → load → tas pats taškas — patikrinta VIENETO teste
      (`core::savestate::tests::save_then_load_on_a_fresh_core_restores_identical_state`) su
      REALIU snes9x core'u + realiu SNES ROM'u, sukuriant NAUJĄ `CoreHandle` (simuliuoja
      procesą iš naujo) ir lyginant `serialize()` išvestį baitas-į-baitą. **NEpatikrinta**
      per pilną gyvą procesą/hotkey (žr. ADR-028 pastabą aukščiau).
- [ ] 4 slot'ai + quick save nepersidengia — DB pusė (`UNIQUE(game_id, slot)`, `upsert`)
      testuota, bet BE `commands::` sluoksnio realaus hotkey→DB kelio nėra
- [x] Preview paveiksliukas teisingas — hand-rolled PNG encoder'is, roundtrip'as per REALŲ
      `png` crate decoder'į (dev-dependency) patvirtina baitas-į-baitą tapatų RGBA turinį,
      įskaitant multi-block DEFLATE kelią (>65535 baitų)
- [ ] Kito core state → įspėjimas, ne crash — logika dar neparašyta (žr. ADR-028 pastabą)

---

### P8.2 — SRAM `[!]`

**Priklausomybės:** P1.7
**Failai:** `crates/nullbyte-core/src/core/sram.rs`, `crates/nullbyte-core/src/core/loader.rs`,
`crates/nullbyte-core/src/core/runner.rs`, `crates/nullbyte-core/src/ipc.rs`

**Ką daryti:**
- `retro_get_memory_data(RETRO_MEMORY_SAVE_RAM)` + `retro_get_memory_size(...)`
- Failas: `saves_dir()/{rom_basename}.srm`
- Įkelk **po** `retro_load_game()`
- Išsaugok: uždarant žaidimą, kas 30 s, ir kai `size > 0` bei turinys pasikeitė
- Atominis rašymas: `.tmp` → `rename`

> **Pastaba (ADR-029, 2026-08-26):** įgyvendinta ATSKIRAME `core::sram` modulyje (ne
> `savestate.rs`, kaip pirminis MVP.md juodraštis siūlė) — CLAUDE.md §8.8 pati sako „Atskirai
> nuo save state'ų", ir realybėje tai skirtinga libretro operacija bei skirtinga panaudojimo
> semantika (progresyvus in-game save, ne vienas užšaldytas taškas). `EmuCommand::Load` gavo
> naują PRIVALOMĄ `sram_path: PathBuf` lauką (analogiškai `states_dir` iš P8.1, bet PILNAS
> failo kelias, ne katalogas — SRAM turi tik VIENĄ failą vienam žaidimui, tėvas jį jau žino iš
> DB) — `IPC_PROTOCOL_VERSION` pakelta į `3`. Periodinis 30s flush'as SĄMONINGAI dirty-check'ina
> (`RunnerState.last_saved_sram`) prieš rašydamas — vengia disko I/O, kai žaidėjas tiesiog
> nesikeičia jokio in-game save'o; uždarant žaidimą (`cleanup`) naudojamas ATSKIRAS,
> BESĄLYGIŠKAS kelias (visada įrašo dabartinę būseną, nepriklausomai nuo dirty-check'o), kad
> paskutinės kelios sekundės progreso NIEKADA nebūtų prarastos.

**Acceptance:**
- [x] RPG in-game save išlieka po perkrovimo — patikrinta VIENETO teste
      (`core::sram::tests::save_then_load_on_a_fresh_core_restores_identical_sram_prefix`) su
      REALIU snes9x core'u + realiu SNES ROM'u (kuris REALIAI praneša size > 0 SRAM), rankiniu
      būdu užrašant atpažįstamą baitų šabloną, save → NAUJAS `CoreHandle` → load → sutampa
      baitas-į-baitą. **NEpatikrinta** per pilną gyvą procesą/realų žaidimą su tikru in-game
      save meniu (žr. P8.1 analogišką pastabą — `commands::`/UI laukia P9.1).
- [x] `.srm` failas nesugadinamas staigiai uždarius — atominis `.tmp` → `rename` (tas pats
      `savestate::write_atomic`, pakartotinai naudojamas iš `core::sram`), tad joks pusiau
      įrašytas failas niekada nepakeičia seno per `rename` (POSIX atomiškumo garantija)
- [x] Core'ai be SRAM (`size == 0`) nesulaužo — `CoreHandle::sram`/`sram_mut` grąžina `None`
      kai `size == 0` ARBA rodyklė `NULL`, `save_sram`/`load_sram` tada tyliai grąžina `Ok(())`
      (žr. `core::sram` doc); testuota netiesiogiai (kelio šaka egzistuoja ir aptarnaujama)

---

## 11. Faza 9 — Integracija ir polish

**Tikslas:** viskas veikia kartu, atrodo baigta.
**Rizika:** 🟡 vidutinė. **Įvertis:** 3–4 dienos.

### P9.1 — Žaidimo paleidimo srautas `[x]`

**Priklausomybės:** P7.4, P2.5, P3.4, P4.4
**Failai:** `crates/nullbyte-app/src/commands/emulator.rs` (naujas), `crates/nullbyte-app/src/ipc.rs`,
`crates/nullbyte-app/src/state.rs`, `crates/nullbyte-app/src/paths.rs`, `crates/nullbyte-app/src/db/games.rs`,
`crates/nullbyte-app/src/commands/settings.rs`, `src/routes/game/[id]/+page.svelte`

> **Priešdarbis atliktas (2026-08-25):** `system_dir`/`save_dir` (`GET_SYSTEM_DIRECTORY`/
> `GET_SAVE_DIRECTORY`, CLAUDE.md §8.3) dabar realiai keliauja `nullbyte-app` → `nullbyte-emu`
> (sidecar CLI argumentai, `EmuClient::spawn`) → `EmuThread::spawn` → `EmuContext` (žr.
> `core::runner::make_initial_context`), abu katalogai sukuriami, jei jų dar nėra. Anksčiau
> visada buvo `NULL` — dauguma core'ų (SNES9x, Genesis Plus GX) tai toleruoja, bet MAME
> besąlygiškai dereferencina ir be šios pataisos segfault'ina kraunant bet kurį žaidimą
> (atrasta rankiniu MAME core smoke testu).

**Ką daryti:**
- UI „Žaisti" → parink core'ą (per `platform_core_prefs`, arba klausk) → paleisk
- Krovimosi būsena UI
- Klaidos (trūksta core'o, trūksta BIOS, blogas ROM) → aiškūs pranešimai, ne stack trace
- Uždarius žaidimą → grįžti į biblioteką, atnaujinti `last_played` ir `play_time`

> **Pastaba (ADR-030, 2026-08-26):** UI srautas, core parinkimas ir `last_played`/`play_time`
> ĮGYVENDINTI IR REALIAI PALEISTA `pnpm tauri dev`. `commands::emulator::start_game` LAUKIA
> tikro `EmuStatus::Loaded`/`Error` (ne tik to, kad `Load`/`Run` nusiuntimas per stdin
> pavyko) prieš grąžindama rezultatą UI — žr. ADR-030 dėl `crate::ipc::EmuClient::spawn`
> oneshot handshake dizaino. Core parinkimas — TIK per Nustatymų → Cores preferenciją
> (`resolve_preferred_core_path`); jei nenustatyta, aiškus klaidos pranešimas su nuoroda į
> Nustatymus — JOKIO automatinio spėjimo/interaktyvaus picker'io (paprasčiau, atitinka P9.3
> „aiškus pranešimas" filosofiją). **Patikrinta REALIAI vartotojo (2026-08-26):** paspaudus
> „Play" tikru žaidimu bibliotekoje, `nullbyte-emu` paleido core'ą ir žaidimas realiai
> veikė — vartotojo žodžiais, „veikia puikiai". Nepatikrinta VISOMS platformoms iš eilės
> (tik ta, kurią vartotojas išbandė) — kodas pats platform-agnostiškas (core'as visada
> renkamas per `platform_slug`, ne hardkodintas), tad likusių platformų veikimas tikėtinas,
> bet ne kiekviena atskirai patvirtinta.

**Acceptance:**
- [x] Paleidimas iš bibliotekos veikia — patikrinta REALIAI vartotojo (žr. ADR-030 pastabą
      aukščiau); ne kiekviena platforma atskirai patvirtinta, bet kodas platform-agnostiškas
- [x] Trūkstamas core → suprantamas pranešimas su nurodymu ką daryti — `start_game` grąžina
      „nėra pasirinkto core'o platformai „X" — nueikite į Nustatymus → Cores..." PRIEŠ
      bandant spawn'inti bet ką
- [x] Žaidimo laikas fiksuojamas — `on_terminated` callback'as (kviečiamas TIK per realų
      proceso pabaigos signalą, ne PID pollinimą, žr. CLAUDE.md §10) skaičiuoja
      `Instant::elapsed()` nuo `start_game` pradžios ir kviečia `db::games::record_play`
      (P9.1 acceptance testuota per `db::games::record_play_increments_count_and_time`,
      jau egzistavusį testą — orkestracijos sluoksnis virš jo dar be savo integracinio testo)

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
| M4 | Biblioteka su metaduomenimis | 5, 6 | 4–6 d. | ✅ |
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
| 4 — Įvestis (+P4.0.x migracija) | 9 | 6 | 67 % |
| 5 — DB / biblioteka | 4 | 4 | 100 % |
| 6 — ScreenScraper | 4 | 4 | 100 % |
| 7 — UI | 6 | 6 | 100 % |
| 8 — Išsaugojimai (P8.1/P8.2 `[!]` — core mechanizmas baigtas, `commands::`/UI laukia P9.1) | 2 | 0 | 0 % |
| 9 — Polish (P9.1 patvirtinta REALIU vartotojo paleidimu) | 6 | 1 | 17 % |
| **Viso** | **52** | **42** | **81 %** |

---

## 13. Rizikų registras

| ID | Rizika | Tikimybė | Poveikis | Mitigacija |
|---|---|---|---|---|
| **R1** | libretro FFI nestabilumas, segfault'ai callback'uose | Vidutinė | 🔴 Kritinis | P1.4/P1.5 daryti atsargiai, `GET_LOG_INTERFACE` pirmiausia, testuoti su 3+ core'ais anksti |
| **R2** | wgpu + Tauri langas neveikia Linux/Wayland | Vidutinė | 🔴 Kritinis | Dokumentuotas fallback į `Channel` + WebGL canvas (P2.3). Testuoti abu backend'us Fazėje 2, ne pabaigoje |
| **R3** | Garso traškesiai, kurių nepavyksta pašalinti | Vidutinė | 🟡 Didelis | Dynamic rate control yra įrodyta technika (RetroArch). Jei nepavyksta — didink buferį iki 100 ms |
| **R4** | Core'ų globalus būvis neleidžia perjungti be restarto | Aukšta | ✅ **IŠSPRĘSTA** | Child procesas (`nullbyte-emu`) kiekvienam paleidimui — ADR-016 (P4.0.1–P4.0.3, 2026-08-20/21). Perkelta iš post-MVP į dabar, nes kartu sprendė ir klaviatūros įvesties problemą |
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

### ADR-017 — `@tanstack/svelte-virtual` grid'o virtualizacijai + Tauri `assetProtocol` viršeliams (P7.2)
**Data:** 2026-08-26 · **Statusas:** priimta

> ⚠️ **PATAISYTA ADR-019** (žr. žemiau): šio ADR apačioje esanti diagnozė („agento aplinkos
> apribojimas, ne kodo klaida") buvo iš dalies klaidinga. Tikroji `list_platforms`
> kabėjimo/bendro neveiksnumo priežastis buvo begalinė reaktyvi kilpa `GameGrid.svelte`
> (žemiau šiame ADR aprašytame kode), suveikianti tik kai grid'as realiai atvaizduoja
> žaidimus. Skaityk ADR-019 pirma, jei domina tikroji priežastis.
**Sprendimas:** `@tanstack/svelte-virtual` (jau numatyta MVP.md P7.2 spec'e) žaidimų grid'o
virtualizacijai — virtualizuojamos EILUTĖS (ne pavieniai kortų elementai), kolonų skaičius
skaičiuojamas reaktyviai iš konteinerio pločio (`bind:clientWidth`) ir minimalaus kortos pločio.
`GameGrid.svelte` sinchronizuoja virtualizatoriaus `count`/`estimateSize` per `$effect` +
`setOptions()`, nes `createVirtualizer()` priima juos tik kaip pradinę reikšmę (Svelte 5
`state_referenced_locally` — tikslingai nuslopinta `svelte-ignore` komentaru).

Kartu įjungtas Tauri `assetProtocol` (`tauri.conf.json` `app.security.assetProtocol.enable` +
`scope`), kad viršeliai galėtų kirsti IPC ribą per `convertFileSrc()` (CLAUDE.md §10, ne
base64) — anksčiau visiškai neegzistavo, taigi bet koks bandymas rodyti viršelį anksčiau būtų
tyliai žlugęs. `scope` — `$HOME`-pagrįsti absoliutūs keliai abiem platformoms (macOS
`Library/Application Support/Nullbyte/media`, Linux `.local/share/nullbyte/media`), NE Tauri
`$APPDATA` kintamasis, nes `paths.rs` naudoja savo katalogo pavadinimą („Nullbyte"), o ne
`identifier` (`fr.nullbyte.app`), su kuriuo `$APPDATA` būtų susietas. `tauri-build` automatiškai
pridėjo `tauri` priklausomybei `features = ["protocol-asset"]` (`Cargo.toml`) — patikrinta
realiai, kompiliuojasi.

**Pastaba dėl P7.2 patikrinimo apribojimo:** ši sesija veikia per automatizuotą fono `bash`
procesą (agento aplinka), o ne interaktyvią vartotojo GUI sesiją. Realiai patikrinta REALIU
`pnpm tauri dev` + tikra 5000 sintetinių įrašų DB (laikinai įterpta testavimui, po to
atstatyta iš backup'o): Rust pusė (`list_platforms`, `list_games`) veikia teisingai ir grąžina
teisingus duomenis (patvirtinta `tracing` log'ais), grid'as atvaizduoja korteles su placeholder'iais
teisingai. TAČIAU pačiame lange (native macOS window) OS lygio input įvykiai (pelės paspaudimai
per `cliclick`, `setTimeout`/`setInterval` timer'iai) NEPASIEKĖ webview'o — patvirtinta: net
paprastas Sidebar mygtuko paspaudimas nepakeitė aktyvios būsenos, nors procesas gyvas ir langas
matomas ekrane (screenshot'ai veikia, nes tai WindowServer kompozicijos lygis, nepriklausomas
nuo programos savo event loop). Tai agento vykdymo aplinkos apribojimas (GUI procesas, paleistas
iš fono shell'o, neturi pilnos interaktyvios sesijos), NE Nullbyte/Tauri kodo klaida — vienas
klik'as/timeris neveiktų VISUR, jei tai būtų reali Tauri/aplikacijos klaida. Dėl to `list_platforms`
IPC pažadas realioje agento sesijoje niekada neišsisprendė (matyti Sidebar tuščias „Platforms"
sąrašas), nors Rust pusė grąžino teisingus duomenis.

**Patvirtinta 2026-08-26:** vartotojas paleido `pnpm tauri dev` savo (realioje, ne agento) sesijoje
— Sidebar platformų sąrašas rodomas teisingai, paieška/`Cmd+K`/paspaudimai veikia normaliai.
Diagnozė (agento aplinkos apribojimas, ne kodo klaida) PASITVIRTINO. Žr. pokalbio istoriją dėl
pilnos diagnostikos sekos.

---

### ADR-018 — `tauri-plugin-dialog` katalogo pasirinkimui + shadcn `progress` komponentas (P7.5)
**Data:** 2026-08-26 · **Statusas:** priimta
**Sprendimas:** `tauri-plugin-dialog` (Rust) + `@tauri-apps/plugin-dialog` (JS) — jau numatyta
MVP.md P7.5 spec'e („ROM katalogų pridėjimas per Tauri dialog plugin"). `open({ directory: true,
multiple: false })` grąžina pasirinktą kelią, kurį frontend'as siunčia į naują
`add_rom_directory` komandą (ne dialog plugin pats rašo į DB — jis TIK parenka kelią).
Capability leidimas `"dialog:default"` pridėtas `capabilities/default.json`.

`progress` shadcn-svelte komponentas pridėtas per CLI (`pnpm dlx shadcn-svelte@latest add
progress`) skenavimo/scraping'o juostoms — jokios naujos npm priklausomybės nepridėjo (naudoja
jau esantį `bits-ui`).

**Papildomas backend darbas (nebuvo P5.3/P6.4 apimtyje, bet reikalingas P7.5 UI funkcionalumui):**
- Naujas `db/rom_directories.rs` — CRUD (`list_rom_directories`, `add_rom_directory`
  idempotentiškas per `ON CONFLICT(path) DO UPDATE`, `remove_rom_directory`). Anksčiau
  egzistavo TIK privatus skaitymo helperis `scanner::load_enabled_directories`, naudojamas
  vidujai `scan()` — jokio CRUD sluoksnio UI valdymui nebuvo.
- 4 naujos Tauri komandos `commands/library.rs`: aukščiau minėtos trys + `scan_library`
  (apvynioja `library::scanner::scan()` į `Channel<ScanProgress>`, tas pats modelis kaip
  `scrape_game`/P6.4).
- `scanner::ScanProgress`/`ScanSummary` gavo `serde::Serialize` + `rename_all = "camelCase"`
  (anksčiau neturėjo — modulio doc sąmoningai vengė TAURI tipų, bet `serde` derive nėra Tauri
  sąvoka, tas pats sprendimas kaip `scraper::ScrapeProgress`). `#![allow(dead_code)]` pašalintas
  iš `scanner.rs` — `scan()` dabar realiai naudojama.
- „Atšaukimo mygtukas" (P7.5 acceptance) — perpanaudoja jau esantį `cancel_scrape` (P6.4), NE
  naujas skenavimo atšaukimas: `scan()` pati (failų vaikščiojimas + hash'avimas) neturi
  `CancellationToken` — greita lokali operacija, ne tinklo, tad P5.3 spec jos ir nereikalavo.
  Sąmoningas apribojimas, ne praleista klaida.

**Papildomas frontend darbas:** minimalus `/settings` maršrutas (tik `PathsPanel`), kad P7.5
komponentas apskritai būtų pasiekiamas — pilnas nustatymų ekranas su tabs (Cores/Input/Scraper)
lieka P7.6. `TopBar` nustatymų mygtukas atrištas nuo „coming soon" stub'o (P7.1) į realią
nuorodą. `Button` komponentas turi `href` prop'ą (render'ina `<a>` vietoj `<button>`) — naudota
vietoj įprasto `<a><Button /></a>` apvyniojimo, kad neatsirastų invalid HTML (button nested
inside anchor).

---

### ADR-019 — `GameGrid` begalinės kilpos taisymas — `get()`, NE `$store`, virtualizatoriaus `$effect` viduje; **ADR-017 diagnozė buvo IŠ DALIES KLAIDINGA**
**Data:** 2026-08-26 · **Statusas:** priimta

**Kas atsitiko:** P7.5 metu vartotojas REALIAI paskenavo 30 SNES ROM'ų (žr. P7.5), grid'as
teisingai atvaizdavo žaidimus — BET iškart po to VISA aplikacija „užstrigo": jokie paspaudimai
(Sidebar, `Cmd+K`, Settings nuoroda) nebereagavo, `list_platforms` niekada neišsisprendė.
Vartotojas atidarė Safari/WebKit dev tools (per right-click → Inspect Element, kuris VEIKĖ, nes
tai native kontekstinis meniu, ne JS) ir rado tikrąją klaidą Console'e:

```
[Error] Svelte error: effect_update_depth_exceeded
Maximum update depth exceeded. This typically indicates that an effect reads and writes
the same piece of state
    ...GameGrid.svelte:41 ($rowVirtualizer.measure())
```

**Šaknis:** `GameGrid.svelte` (P7.2) `$effect` bloke:
```js
$effect(() => {
  $rowVirtualizer.setOptions({ count: rowCount, estimateSize: () => cardHeight + GAP });
  $rowVirtualizer.measure();
});
```
`$rowVirtualizer` — Svelte `Readable` store (`@tanstack/svelte-virtual`). `$`-prefiksas ČIA
sukuria reaktyvią prenumeratą — bet `setOptions()`/`measure()` PATYS priverčia virtualizatorių
perskaičiuoti dydžius ir PRANEŠTI apie pasikeitimą (store emituoja naują reikšmę). Rezultatas:
effect skaito store'ą (prenumeruoja) → viduje iškviečia metodus, kurie priverčia store'ą
emituoti → Svelte mato pasikeitusią priklausomybę → effect PALEIDŽIAMAS IŠ NAUJO → vėl kviečia
tuos pačius metodus → begalinė sinchroninė kilpa, kuri visiškai užblokuoja JS single-threaded
event loop'ą. Tai paaiškina VISUS anksčiau stebėtus simptomus vienu metu: `invoke()` pažadai
niekada neišsisprendžia (event loop'as nespėja apdoroti native→JS callback'o), `setTimeout`/
`setInterval` niekada nesuveikia, paspaudimai neregistruojami — VISKAS, ko reikia event loop'o
tolimesniam ciklui.

**Sprendimas:** imperatyviems `setOptions()`/`measure()` kvietimams naudoti `get(rowVirtualizer)`
(iš `svelte/store`) vietoj `$rowVirtualizer` — `get()` perskaito DABARTINĘ reikšmę BE
prenumeratos, nesukurdamas priklausomybės. `$rowVirtualizer` reaktyvus naudojimas PALIEKAMAS
template'e (`getTotalSize()`, `getVirtualItems()`), kur prenumerata tikrai reikalinga UI
atnaujinimui.

**⚠️ KRITIŠKAI SVARBI PATAISA ADR-017 diagnozei:** ADR-017 (P7.1/P7.2) padarė IŠVADĄ, kad
`list_platforms` amžinai kabantis pažadas ir bendras neveiksnumas agento sesijoje buvo VIEN
agento aplinkos apribojimas (GUI procesas paleistas per fono `bash`, negauna OS input). Ta
diagnozė buvo **NETEISINGA arba bent jau NEPILNA** — tikroji (ar bent PAGRINDINĖ) priežastis
buvo ŠI begalinė kilpa, kuri egzistavo kode NUO P7.2 (kai buvo parašytas `GameGrid.svelte`) ir
tiesiog nebuvo suveikusi anksčiau, nes agento testavimas ARBA visai neturėjo realių žaidimų
(tuščia biblioteka → `GameGrid` niekada nesumontuojamas), ARBA sintetinių 5000 įrašų testas
(P7.2 ADR-017 pastaba) sukėlė TĄ PATĮ bug'ą, bet buvo klaidingai priskirtas „aplinkos
apribojimui", nes agento pačio `cliclick`-pagrįstas testavimas VISADA buvo nepatikimas
(nepavyko net teisingomis koordinatėmis), todėl nebuvo aiškaus skirtumo tarp „aplinka
netikima" ir „app tikrai užstrigęs". **Pamoka:** kai kas nors atrodo „visiškai neveikia" (net
paprasčiausi paspaudimai/timer'iai), PIRMIAUSIA reikia patikrinti webview Console dėl JS
klaidų (per Inspect Element, jei įmanoma), NE iš karto priskirti aplinkos apribojimui — net jei
aplinkos apribojimas (agento `cliclick` nepatikimumas) IR YRA realus atskirai (tebelieka
patvirtintas — user's šios sesijos testas irgi patvirtino, kad savarankiškas agento
paspaudimų testavimas nepatikimas), jis gali UŽMASKUOTI arba SUSIMAIŠYTI su tikra klaida.
Atitinkamai atnaujinta atmintis (`feedback_native_window_no_input_in_agent_session.md`).

**Pasekmės:** P7.2/P7.3/P7.4/P7.5 acceptance punktai, kurie anksčiau buvo pažymėti „reikia
vartotojo patvirtinimo", DABAR realiai patvirtinti — žr. atitinkamas sekcijas aukščiau,
atnaujinta 2026-08-26 po šio pataisymo.

---

### ADR-020 — `rom_directories.platform_id` — vartotojo nurodomas platformos hint'as skenavimui (P7.5)
**Data:** 2026-08-26 · **Statusas:** priimta

**Kontekstas:** vartotojas realiai nuskenavo SNES/Genesis/GBA/PSX/MAME katalogus. Rezultatai
atskleidė DVI realias klaidas:
1. **GBA rodė 0 žaidimų.** `platforms.gba.extensions = 'gba'` neįtraukė `zip`/`7z` — realūs
   GBA romset'ai beveik visada suarchyvuoti. Analogiškas atvejis kaip PSX/Saturn/SegaCD
   (`002_fix_archive_extensions.sql`), tiesiog nepastebėtas tada. **Pataisyta migracija 004**
   (`UPDATE platforms SET extensions = 'gba,zip,7z' WHERE slug = 'gba'`) — REALIAI patikrinta:
   20/20 GBA žaidimų po pataisymo.
2. **3 realūs PSX žaidimai (Castlevania SotN, Tekken 3, Tony Hawk's) atsidūrė po „Sega CD".**
   PSX/Saturn/SegaCD visos priima `.cue`/`.iso`/`.chd` archyvo viduje — `scanner.rs`
   `resolve_platform_and_hashes` ima PIRMĄ tinkantį kandidatą `platforms` sąrašo tvarka
   (Sega CD `id=10` < PSX `id=13`, žr. `001_initial.sql` seed eiliškumą). Vien plėtinio
   nepakanka vienareikšmiam nustatymui — reikalingas papildomas signalas.

**Sprendimas:** du pasirinkimai buvo apsvarstyti su vartotoju — (A) `rom_directories` gauna
nebūtiną `platform_id` stulpelį, vartotojas eksplicitiškai pasako „šis katalogas — PSX"; (B)
tiesiog pakeisti prioritetų tvarką (PSX prieš Saturn/SegaCD, nes PS1 žaidimų realiai daugiausia).
**Vartotojas pasirinko (A)** — tvarkingas sprendimas, pašalina dviprasmybę VISIŠKAI, ne tik šiam
konkrečiam atvejui.

**Įgyvendinta (migracija 005 + kodas):**
- `rom_directories.platform_id INTEGER REFERENCES platforms(id)` — `NULL` = senas automatinis
  elgesys (veikia gerai vienareikšmiams plėtiniams).
- `resolve_platform_and_hashes()` gauna `platform_hint: Option<i64>` — kai `Some`, kandidatų
  sąrašas susiaurinamas iki VIENOS nurodytos platformos (nulinė dviprasmybė).
- `scan()` **priverstinai perklasifikuoja** jau įrašytą žaidimą, jei katalogo `platform_id`
  hint'as skiriasi nuo jau įrašytos platformos, NEPAISANT `mtime` (savaiminis pasitaisymas
  pridėjus hint'ą + rescan'inant — vartotojui NEREIKIA rankiniu būdu taisyti DB).
- `add_rom_directory` idempotentiškas ir hint'ui — pakartotinis pridėjimas TO PATIES kelio
  su NAUJU hint'u atnaujina `platform_id` (nėra atskiros „edit" komandos).
- `PathsPanel.svelte` — platformos `<Select>` prie „Add directory" (numatytoji „Auto-detect").
  Katalogų sąraše rodoma kiekvieno priskirta platforma.

**REALIAI patikrinta 2026-08-26** (vartotojo sesijoje, po duomenų išvalymo ir pilno rescan'o):
88 žaidimai teisingai — 30 SNES + 35 Genesis + 20 GBA + **3 Sony PlayStation** (NE Sega CD).

**Žinomas apribojimas (NEIŠSPRĘSTAS, sąmoningai atidėtas vartotojo sprendimu):** `scan_library`
laiko `state.db` `Mutex<Connection>` UŽRAKINTĄ per VISĄ skenavimo trukmę, įskaitant lėtą failų
hash'avimą (CRC32+MD5+SHA1). Su labai dideliais archyvais (vartotojo PSX test fixture'ai —
400–514MB kiekvienas) tai REALIAI pastebima: VISA aplikacija tampa neatsakanti kitiems DB
poreikalaujantiems veiksmams (bet koks kitas mygtukas/naršymas), kol skenavimas nesibaigia —
pažeidžia P7.5 acceptance „Progresas sklandus, be UI užšalimo" tikrąja to žodžio prasme šiam
edge case'ui (nors PATS progreso mechanizmas veikia teisingai — Channel siunčia atnaujinimus,
UI juos rodo, tiesiog KITI veiksmai blokuojami tuo pat metu). Nekritiška realiam MVP naudojimui
(SNES/Genesis/GBA ROM'ai — KB–kelių MB dydžio, hash'avimas trunka milisekundes), bet
neišspręsta architektūrinė spraga: vienas global `Mutex<Connection>` (CLAUDE.md §10 „SQLite" —
MVP sąmoningas supaprastinimas) reiškia BET KOKS ilgai trunkantis DB veiksmas blokuoja VISUS
kitus. Tikras sprendimas — nelaikyti lock'o per PATĮ hash'avimą (tik trumpiems DB read/write
žingsniams), reikalautų `scan()` pertvarkymo. **Palikta kaip žinoma spraga, ne pataisyta —
vartotojo sprendimas 2026-08-26**, kad būtų galima tęsti prie scraping'o testavimo.

---

### ADR-021 — Tikri viršelio matmenys DB'je + „packed row" GameGrid layout'as (P7.2 patobulinimas)
**Data:** 2026-08-26 · **Statusas:** priimta

**Kontekstas:** po realaus scraping'o (88 žaidimai, 4 platformos) vartotojas pastebėjo, kad
PSX viršeliai grid'e apkirpti — matėsi tik dalis „PlayStation" logotipo. Priežastis:
`GameCard` naudojo fiksuotą `aspect-[3/4]` dėžę su `object-cover`. Patikrinus REALIUS
atsisiųstus failus (`sips -g pixelWidth -g pixelHeight`) paaiškėjo, kad viršelių proporcijos
LABAI skiriasi tarp platformų: PSX 680×680 (kvadratas), SNES 680×497 (platus), Genesis
484×680 (aukštas), GBA 705×700 (beveik kvadratas). Jokia bendra ar platformos-lygio prielaida
netiktų visiems atvejais.

**Du keliai apsvarstyti su vartotoju:** (A) tikri matmenys saugomi DB'je scraping'o metu,
GameGrid perrašomas į layout'ą su tiksliais pločiais; (B) matuoti `<img>` elementą kliento
pusėje po užsikrovimo (jokių backend pakeitimų, bet vizualus „šuoliukas" perkraunant ir
sudėtingesnė virtualizacija su kintamu pločiu). **Vartotojas pasirinko (A).**

**Backend įgyvendinimas:**
- Migracija 006: `games.cover_width`/`cover_height` (nullable INTEGER).
- Naujas `scraper/image_dimensions.rs` — minimalus PNG/JPEG header'io parseris (TIK plotis/
  aukštis, be dekodavimo) — SĄMONINGAI NE `image` crate (per sunku vien šiai reikmei, žr.
  CLAUDE.md §11.8 — nauja priklausomybė reikalautų atskiro ADR pagrindimo, o minimalus
  parseris — < 100 eilučių, pilnai testuojamas). PNG: signatūra + IHDR chunk fiksuotame
  offset'e. JPEG: markerių skenavimas iki SOF0-SOF15.
- `media.rs` `download_game_media()` po viršelio atsisiuntimo (ar radimo jau esančio disko) IŠ
  KARTO nuskaito matmenis iš PAČIO failo diske — veikia abiem atvejais (naujai atsisiųstam IR
  jau esančiam), nes abiem failas realiai egzistuoja disko.
- **REALIAI patikrinta** `#[ignore]`'intu tinklo testu (`real_cover_downloads_from_live_screenscraper_response`):
  tikras SNES viršelis → `cover_width: Some(680), cover_height: Some(497)` — TIKSLIAI sutampa
  su rankiniu `sips` patikrinimu.

**Frontend įgyvendinimas:**
- `GameGrid.svelte` visiškai perrašytas: vietoj vienodo stulpelių tinklelio (fiksuotas plotis,
  `Math.ceil(games.length / columns)` eilučių) — „packed row" algoritmas: fiksuota `ROW_HEIGHT`
  (220px), kiekvienos kortos plotis = `ROW_HEIGHT * (coverWidth / coverHeight)`, kortos dedamos
  „iš eilės kol tilpsta" (ragged-right, NE edge-to-edge justify — vartotojas prašė TIK
  „fiksuota aukštis, plotis kad tilptų", ne tobulo išlyginimo). Žaidimams be žinomų matmenų
  (dar nescrape'inti) — numatytoji `3/4` proporcija placeholder'iui.
- Virtualizacija IŠLIEKA (ta pati `@tanstack/svelte-virtual` + ADR-019 `get()` pataisa) —
  `rows` (masyvas masyvų) apskaičiuojamas kaip `$derived.by()`, virtualizuojama pagal EILUTĖS
  indeksą kaip anksčiau, tiesiog kiekviena eilutė dabar turi KINTAMĄ kortų skaičių/pločius.
- `GameCard.svelte` — pašalintas fiksuotas `aspect-[3/4]`, dydis dabar ateina iš TĖVINIO
  `GameGrid` apskaičiuoto `style:width`/`style:height` (pikseliais).

**Jau esančių 88 žaidimų (scrape'intų PRIEŠ šį pakeitimą) atgalinis užpildymas:** rankiniu
būdu per `sips` perskaityti JAU atsisiųstų viršelių matmenys ir tiesiogiai UPDATE'inti DB —
NEREIKĖJO pakartotinio ScreenScraper API kvietimo (kvotos netaupymas), nes failai jau buvo
diske. Ateities scraping'ai automatiškai užpildys naujus žaidimus per `media.rs` pakeitimą.

**REALIAI patikrinta 2026-08-26** (vartotojo ir mano screenshot'ais): PSX viršeliai dabar
kvadratiniai su pilnai matomu „PlayStation" logotipu, SNES platūs, Genesis aukšti — visi BE
apkirpimo, fiksuoto aukščio eilutėse, kaip prašyta.

**Papildomas realaus pasaulio patvirtinimas (edge case):** „Final Fantasy IV" (SNES) ROM'o
CRC32 sutapo su JAPONIŠKA ScreenScraper įrašo versija (žinomas atvejis — JAV SNES leidimas
buvo perkrikštytas į „Final Fantasy II", kitas ROM/CRC) — jos vienintelis `box-2D` variantas
buvo `region: jp`, 478×864 (AUKŠTAS, VISIŠKAI kitokia proporcija nei įprastas SNES 680×497).
Naujas „packed row" layout'as tvarkingai atvaizdavo šią kortelę SAVO tikra, neapkirpta
proporcija — tiksliai tam ir buvo kurtas šis sprendimas. Vartotojas rankiniu būdu pakeitė
viršelį į JAV „Final Fantasy II" atitikmenį (rasta per papildomą `romnom`-pagrįstą paiešką TAI
PAČIAI ScreenScraper API'jai, ne kodo pakeitimas), tada nusprendė žaidimą visai pašalinti iš
bibliotekos testavimo metu — abu veiksmai atlikti tiesiogiai per DB/failų sistemą vartotojo
prašymu, NE per app'o UI (nėra `delete_game` komandos — post-MVP, jei prireiks).

---

### ADR-022 — ScreenScraper kredencialai redaguojami UI, `settings` lentelė TURI PIRMENYBĘ prieš `.env` (P7.6 Scraper panelė)
**Data:** 2026-08-26 · **Statusas:** priimta

**Kontekstas:** P7.6 Scraper panelės pirma versija rodė kredencialų būvį TIK skaitymui
(`.env`), nes CLAUDE.md §9.3 anksčiau leido abu variantus („Dev credentials — iš `.env`/
nustatymų"), bet UI redagavimas dar nebuvo įgyvendintas. Vartotojas paprašė padaryti
kredencialus redaguojamus per UI.

**Sprendimas:**
- Naujas `db/settings.rs` — plika `String -> String` KV sąsaja (`get`/`set`/`delete`) virš JAU
  egzistuojančios `settings` lentelės (P5.1 schema, iki šiol nenaudota). SĄMONINGAI be
  tipizuoto `Settings` struct'o — būsimi domenai (core/video/audio nustatymai) turės visiškai
  skirtingus raktus, bendras struct'as tik pridėtų netiesioginumo.
- `ScreenScraperCredentials::load(conn)` (nauja, greta senos `from_env()`, kuri LIEKA
  nepakitusi žemesniam sluoksniui) — `settings` lentelės reikšmės (raktai
  `scraper.dev_id`/`dev_password`/`ssid`/`sspassword`, konstantos viešos kaip
  `ScreenScraperCredentials::KEY_*`) TURI PIRMENYBĘ prieš `.env`, nes vartotojo paskutinis
  veiksmas per Settings ekraną turi laimėti prieš statinį failą. Tuščia arba nesanti DB
  reikšmė krenta atgal į `.env` PER LAUKĄ (ne viskas-arba-nieko) — pvz. galima turėti `devid`/
  `devpassword` iš `.env`, bet `ssid` override'intą tik UI.
- `commands::scraper::{scrape_game, scrape_library, get_scraper_status}` perjungti nuo
  `from_env()` į `load(&conn)`. Naujos komandos `set_scraper_credentials`/
  `clear_scraper_credentials` — validuoja, kad `devId`/`devPassword` neturi būti tušti, tuščią
  `ssid`/`sspassword` traktuoja kaip „ištrink override'ą", ne kaip tuščios eilutės įrašymą.
  `ScraperCredentialStatus` gavo `overridden: bool` lauką — UI juo sprendžia, ar rodyti „Clear
  override" mygtuką.
- **Niekada** negrąžinami tikri slaptažodžiai atgal į UI (net redaguojant) — forma visada
  prasideda TUŠČIA, vartotojas įveda pilnas naujas reikšmes; rodomas tik maskuotas `devid`
  prefix'as (`"ab••••"`) esamai konfigūracijai atpažinti.

**Testavimas:** `db/settings.rs` — CRUD unit testai (in-memory DB). `screenscraper.rs` —
du nauji testai (`load_prefers_settings_table_over_env`,
`load_falls_back_to_env_when_settings_table_empty`) PAGAVO realų lygiagretumo bug'ą: trys
testai šiame faile mutuoja tuos pačius PROCESO GLOBALIUS `SCREENSCRAPER_*` env kintamuosius,
o Rust testai viename binare paleidžiami LYGIAGREČIAI — be sinchronizacijos jie realiai
lenktyniaudavo (pastebėta CI-tipo bandymu `cargo test`, ne teoriškai). Ištaisyta modulio lygio
`static ENV_LOCK: Mutex<()>` + `lock_env()` helper'iu, kurį visi trys testai kviečia prieš
mutuodami env — dabar deterministiškai praeina pakartotinai (patikrinta 3× iš eilės).

**Dar NEĮGYVENDINTA šioje sesijoje:** regionų prioritetas ir media tipai Scraper panelėje
lieka READ-ONLY (hardkodintos Rust konstantos) — jų padarymas redaguojamu būtų atskiras,
platesnis žingsnis (naujos `settings` raktai + `screenscraper.rs`/`media.rs` skaitymas iš DB
vietoj konstantų), vartotojo sąmoningai atidėtas šiam kartui.

---

### ADR-023 — Paths panelės „kurioms platformoms reikia hint'o" pastaba + Neo Geo archyvo plėtinio fix'as
**Data:** 2026-08-26 · **Statusas:** priimta

**Kontekstas:** vartotojas paprašė P7.6 Paths skiltyje aiškios pastabos, kurioms platformoms
saugu palikti „Auto-detect", o kurioms reikia eksplicitiškai nurodyti `platform_id` hint'ą
(ADR-020 mechanizmas). Prieš rašant pastabą, PERSKAIČIAVAU visą `platforms.extensions`
lentelę (visas migracijas, ne tik prisiminimą) — ir radau DAUGIAU nei vien ADR-020 jau
žinomą PSX/Saturn/SegaCD atvejį.

**Realios dviprasmybės (patikrinta, ne spėta), reikalaujančios hint'o:**
1. **Sony PlayStation / Sega Saturn / Sega CD** — dalinasi `.cue`/`.iso`/`.chd` (jau ADR-020).
2. **NAUJAI RASTA: laisvi (nearchyvuoti) `.bin` failai** — dalinasi PENKIOS platformos:
   Genesis/Mega Drive, Sony PlayStation, Atari 2600, Intellivision, Magnavox Odyssey². Skirtingai
   nuo `.zip`/`.7z` atvejo (kur `resolve_platform_and_hashes` tikrina TIKRĄ archyvo TOC turinį
   prieš pasirinkdama kandidatą), laisvam failui NĖRA jokio turinio patikrinimo — laimi PIRMA
   `platforms` lentelės eilutė, kurios `extensions` sąraše yra `bin` (šiuo metu tai Genesis,
   nes SQLite grąžina eilutes INSERT tvarka, be `ORDER BY`). T.y. laisvas Atari 2600/
   Intellivision/Odyssey²/nearchyvuotas PSX `.bin` failas BE hint'o klaidingai atsidurs po
   Genesis.

**Tos pačios klasės latentinis bug'as, rastas TIRIANT (ne ieškotas specialiai):** 002 migracija
(`002_fix_archive_extensions.sql`) pašalino `zip`/`7z` iš Neo Geo IR ištuštino Arcade plėtinių
sąrašą — abi platformos šiuo metu VISIŠKAI neatpažįsta jokio suarchyvuoto ROM'o, NET su
`platform_id` hint'u (nes hint tik susiaurina KANDIDATŲ sąrašą iki vienos platformos, bet ta
platforma VIS TIEK turi turėti atitinkamą plėtinį savo `extensions` sąraše — tuščias/be zip
sąrašas reiškia joks failas niekada neatitiks, nepriklausomai nuo hint'o).

**Kodėl Neo Geo IR Arcade GAVO SKIRTINGĄ sprendimą (aptarta su vartotoju):**
- **Neo Geo** — `.neo` yra VIENO FAILO formatas (kaip GBA `.gba`, žr. ADR-020/migraciją 004).
  `archive::extract_first_match` (nullbyte-core) ieško VIENO archyvo viduje esančio failo pagal
  plėtinį — šis modelis Neo Geo tinka TIKSLIAI. Migracija 007:
  `extensions = 'neo,zip,7z'` — vienos eilutės fix'as, identiškas GBA precedentui.
- **Arcade** — MAME-tipo ROM setai yra KELIŲ žalio chip dump'o failų rinkinys BE bendro,
  atpažįstamo plėtinio (skirtingai nuo Neo Geo `.neo` vieno failo modelio) — `zip`/`7z`
  grąžinimas VIENAS PATS NEPADĖTŲ, nes `extract_first_match`/`has_valid_extension`
  (`nullbyte-core/src/archive.rs`) reikalauja NORS VIENO plėtinio sąraše, o po `zip`/`7z`
  pašalinimo iš `inner_extensions` (žr. `scanner.rs` `resolve_platform_and_hashes` — jis pats
  filtruoja `zip`/`7z` iš vidinio plėtinio kandidatų) Arcade liktų su TUŠČIU vidinių plėtinių
  sąrašu → `has_valid_extension` visada `false` → JOKS failas archyve niekada nesutaptų.
  Tikras sprendimas reikalautų NAUJOS logikos (visą `.zip` traktuoti kaip ROM tapatybę pagal
  PATĮ ARCHYVO vardą, ne ieškoti vidinio failo pagal plėtinį) — SĄMONINGAI NEDARYTA šioje
  sesijoje (vartotojo pasirinkimas: „Neo Geo pataisyti dabar, Arcade palikti kaip žinomą
  apribojimą"), pažymėta kaip atskiras post-MVP/vėlesnės fazės darbas.

**Įgyvendinta:**
- Migracija 007 (`007_fix_neogeo_archive_extension.sql`) — grąžina `zip`/`7z` Neo Geo.
- Naujas testas `neogeo_extensions_include_archive_formats_after_migration_007`
  (`db/migrations.rs`).
- `PathsPanel.svelte` gavo informacinį banner'į (tas pats vizualinis stilius kaip Input
  panelės „not applied to gameplay yet" banner'is) — trys sakiniai: kurioms platformoms
  REIKIA hint'o (su konkrečiu plėtinio persidengimo paaiškinimu), kuri platforma VISIŠKAI
  nepalaikoma (Arcade), ir kad visos kitos saugu palikti Auto-detect.

**REALIAI patikrinta:** `cargo test --workspace` (77 testai `nullbyte-app`, įsk. naują),
`cargo clippy --workspace -D warnings`, `pnpm check/lint/build` — visi švarūs.

---

### ADR-024 — Cores panelė: pilnai funkcionalus core'ų sąrašas + preferuojamo core'o persistencija (P7.6)
**Data:** 2026-08-26 · **Statusas:** priimta

**Kontekstas:** P7.6 „Cores" skiltis buvo paskutinis „coming soon" stub'as prieš pereinant
prie Video/Audio arba P9.1. Skirtingai nuo Input/Scraper credentials pakeitimų — DALIS šios
funkcijos VEIKIA ŠIANDIEN be jokio P9.1 apribojimo, nes P1.3 (`nullbyte-core/src/core/info.rs`)
JAU turėjo pilnai parašytą ir testuotą `scan_cores_dir`/`CoreInfo`/`extension_to_cores`, tiesiog
be jokio Tauri komandų sluoksnio virš jo (modulio doc komentaras tiesiogiai nurodė:
„Naudos commands/settings.rs (list_cores)... kol jie neparašyti, šis modulis pilnai
išnaudojamas tik testuose").

**Sprendimas:**
- Naujas `commands/settings.rs` (PIRMAS kartas, kai šis failas užpildomas — anksčiau
  `commands/mod.rs` doc komentaras jį žymėjo kaip „dar neužpildyta, liks vėlesnei fazei").
- `list_cores` — PLONAS DTO suplokštinimas (`CoreInfoDto`, `PathBuf` → `String` per
  `to_string_lossy`) virš NEPAKEISTO `scan_cores_dir`. Tuščias `cores_dir` (naujas diegimas,
  core'ų dar neatsisiųsta) grąžina TUŠČIĄ sąrašą, NE klaidą — patikrinama PRIEŠ kviečiant
  `scan_cores_dir` (kuris pats grąžintų `Err`, jei katalogo nėra, nes viduje kviečia
  `std::fs::read_dir` be apsaugos).
- `get_preferred_cores`/`set_preferred_cores` — TAS PATS `settings` lentelės vienas-JSON-raktas
  šablonas kaip `input.mapping` (ADR — žr. `commands/input.rs`) ir `core.preferred`: visas
  `Vec<PlatformCorePreference>` saugomas/skaitomas kaip VIENAS atomiškas JSON blob'as po raktu
  `"core.preferred"`, ne per-platformos eilutės — nuoseklu su anksčiau šioje sesijoje
  pasikartojusiu šablonu.
- **Preferuojamo core'o pasirinkimas TURI TĄ PATĮ P9.1 apribojimą kaip Input mapping'as** —
  išsaugoma, bet joks realaus žaidimo paleidimo kelias jo dar nenaudoja. UI aiškiai tai
  pažymi (tas pats banner stilius kaip `InputPanel.svelte`).

**Frontend:**
- `CoresPanel.svelte` — dvi sekcijos: (1) aptiktų core'ų sąrašas (pavadinimas/versija/sistema/
  palaikomi plėtiniai, arba pagalbinis tekstas su TIKSLIU `cores_dir` keliu, jei tuščia); (2)
  preferuojamo core'o `Select` kiekvienai platformai iš `library.platforms`.
- Naudoja bits-ui `Select.Root` `onValueChange` callback'ą (NE `bind:value` masyvo elementui —
  tas šablonas netiktų sąrašui su keliais nepriklausomais `Select` egzemplioriais).

**REALIAI patikrinta:** `cargo build/clippy/fmt -p nullbyte-app` švarūs, `pnpm check/lint/build`
0 klaidų/warning'ų.

**Iteracija #1 — extension'ų sutapimas (ATMESTA po vartotojo realaus patikrinimo, 2026-08-26):**
iš pradžių `Select` sąrašas buvo filtruojamas pagal `validExtensions` sutapimą su platformos
plėtiniais — vartotojas nukopijavo 11 realių core'ų į `cores_dir` ir pastebėjo, kad Sony
PlayStation rodė 6 pasirinkimus, nors realiai PSX palaiko tik 3 (Beetle PSX, Beetle PSX HW,
SwanStation). Patikrinau REALIAIS core'ais (laikinu `cargo run --example` diagnostikos
skriptu, ištrintu po patikrinimo): PicoDrive ir Genesis Plus GX TAIP PAT deklaruoja
`cue`/`iso`/`chd`/`m3u` (jie emuliuoja Sega CD, `m3u` — savo daugiadiskių sąrašams), o MAME
plačiai deklaruoja `zip`/`7z` — TA PATI persidengianti plėtinių aibė kaip ADR-020/023
skenavimo dviprasmybėje, dabar pasirodžiusi IR core'ų pasirinkime.

**Iteracija #2 — jokio filtro (TAIP PAT ATMESTA, vartotojas iš karto pastebėjo):** pašalinus
filtravimą visiškai, `Select` rodė VISUS core'us kiekvienai platformai — vartotojas teisingai
nurodė, kad tai BLOGIAU, ne geriau: „gali pasirinkti betkoki ir neveiks". Reikėjo TIKSLAUS
sprendimo, ne pasidavimo.

**Iteracija #3 — kuruota lentelė (GALUTINIS sprendimas):** kadangi libretro API NETURI „kokias
platformas palaikau" lauko (TIK `valid_extensions`), o `.info` failai (kuriuose būtų
patikimesnis `systemname`) yra NEBŪTINAS atsisiuntimas ir šioje aplinkoje jų NĖRA nė vienam
core'ui — vienintelis TIKSLUS sprendimas yra rankiniu būdu patikrinta core'o pavadinimas
→ platformos `slug` lentelė (`known_core_platforms`, `commands/settings.rs`), TA PATI
filosofija kaip `platforms` seed'as (P5.1) su kuruotais ScreenScraper ID'ais. Nauja
`CoreInfoDto.supported_platforms: Option<Vec<String>>` — `None` reiškia „core'as
neatpažintas kuruotoje lentelėje", UI tai traktuoja kaip „nepatikrinta" (rodo VISUR, pažymėtą
„· unverified"), NE „nepalaiko nieko" (neslepia nauko/nekataloguoto core'o). Lentelė kol kas
apima TIK 9 REALIAI ŠIOJE SESIJOJE patikrintus core'us (Snes9x, bsnes-mercury, mGBA, Genesis
Plus GX, PicoDrive, Beetle PSX, Beetle PSX HW, SwanStation, MAME) — jokių nepatikrintų
spėjimų apie kitus core'us. Du nauji testai: `every_known_core_platform_slug_exists_in_the_seed_table`
(apsauga nuo netikslaus `slug` typo lentelėje) ir `psx_maps_to_exactly_the_three_real_psx_cores`
(tiesiogiai užfiksuoja ADR-024 esmę — PSX = TIK 3 core'ai, ne 6).

**Iteracija #4 — automatinis rekomenduojamo core'o priskyrimas + „None" pašalinta, kai turi
core'ą (vartotojo prašymas, 2026-08-26):** vartotojas paprašė, kad turint tinkamą core'ą,
jis būtų priskirtas AUTOMATIŠKAI (kad nereikėtų 23 platformoms rankiniu būdu suvedinėti), IR
kad „None" apskritai nebūtų rodoma kaip pasirinkimas platformoms, kurios turi bent vieną
VERIFIKUOTĄ core'ą.
- Nauja `CORE_PRIORITY_ORDER` lentelė (`commands/settings.rs`) — platformos `slug` → core'o
  pavadinimų prioritetinė tvarka, kai KELETAS core'ų ją palaiko (pvz. `psx` →
  `["SwanStation", "Beetle PSX HW", "Beetle PSX"]`). Nauja komanda `get_core_priority()` —
  GRYNAI statiniai duomenys, JOKIO `cores_dir` pakartotinio skenavimo (MAME ~400MB core'ą
  būtų brangu įkelti du kartus vien rekomendacijai apskaičiuoti) — frontend'as sujungia su JAU
  turimu `list_cores` rezultatu.
- Naujas testas `every_priority_entry_is_consistent_with_known_core_platforms` — apsauga nuo
  DVIEJŲ lentelių (`known_core_platforms`/`CORE_PRIORITY_ORDER`) išsiskyrimo (typo viename,
  bet ne kitame reikštų, kad rekomendacija tyliai niekada nesutaptų su jokiu core'u).
- `CoresPanel.svelte`: reaktyvus `$effect` (NE vienkartinis kvietimas `load()` viduje, nes
  `library.platforms` užsipildo ASINCHRONIŠKAI iš atskiro store'o ir gali dar būti tuščias,
  kai šio komponento pradinis `load()` baigiasi) automatiškai priskiria+išsaugo rekomenduojamą
  core'ą KIEKVIENAI platformai, kurios vartotojas dar NELIETĖ. Idempotentiškas (antras
  paleidimas po `preferences` pasikeitimo nieko naujo neprideda, jokios begalinės kilpos).
- **„None" IŠVIS nerodoma** platformoms, turinčioms bent vieną VERIFIKUOTĄ core'ą
  (`showNoneFor`) — tik nežinomi/unverified core'ai lieka pasirenkami kaip papildomas
  variantas, niekada nelaikomi „turi core'ą" požymiu. Kadangi automatinis priskyrimas VEIKIA
  BŪTENT toms pačioms (verifikuotoms) platformoms, konflikto tarp „vartotojas eksplicitiškai
  pasirinko None" ir „automatinis pasiūlymas" NĖRA — tokioms platformoms None niekada
  nebuvo pasiekiama pasirinktis iš viso.

---

### ADR-025 — Video/Audio panelės: skirtingi P9.1 apribojimai kiekvienam laukui, `cpal` naudojimo išplėtimas į `nullbyte-app` (P7.6 pabaiga)
**Data:** 2026-08-26 · **Statusas:** priimta

**Kontekstas:** paskutinės dvi P7.6 skiltys. Prieš rašant kodą paleidau `Explore` subagent'ą
patikrinti, KIEK iš MVP.md P7.6 „Ką daryti" sąrašo (filtras/scaling/vsync/fullscreen,
įrenginys/garsumas/buferis) jau turi VEIKIANTĮ Rust API, o kiek — tik hardkodintas konstantas
be jokio runtime hook'o net PRIEŠ P9.1 wiring'ą. Rezultatas — LABAI nevienalytis vaizdas (žr.
P7.6 „Video/Audio panelės" pastabą aukščiau pilnam sąrašui), kitaip nei Input/Cores, kur VISKAS
buvo vienodai P9.1-blokuota.

**Sprendimas:**
- `VideoSettings`/`AudioSettings` — TAS PATS vieno-JSON-rakto `settings` šablonas kaip
  `input.mapping`/`core.preferred` (`video.settings`, `audio.settings` raktai,
  `commands/settings.rs`).
- `VideoSettings::default()` TIKSLIAI atitinka `FilterMode`/`ScaleMode` `#[default]` variantus
  (Rust enum'us) — apsaugota testu `video_settings_default_matches_renderer_defaults`, kad
  UI numatytoji reikšmė NIEKADA neišsiskirtų nuo to, ką core'as realiai naudotų.
- **Nauja `cpal` priklausomybė TIESIOGIAI `crates/nullbyte-app/Cargo.toml`** (anksčiau TIK
  `nullbyte-core`) — `list_audio_devices()` naudoja `host.output_devices()` enumeraciją, kuri
  yra grynas OS užklausimas, VEIKIANTIS nepriklausomai nuo bet kokio aktyvaus garso srauto ar
  `nullbyte-emu` vaiko proceso egzistavimo. Tai NĖRA nauja priklausomybė koncepciškai (jau
  vetuota per `nullbyte-core`), tik nauja naudojimo vieta — bet vis tiek pažymima čia, kaip
  reikalauja CLAUDE.md §11.8.
- `VideoPanel.svelte`/`AudioPanel.svelte` — banner tekstas KIEKVIENAI skilčiai NURODO TIKSLIAI,
  kodėl konkretus laukas dar netaikomas (P9.1 IPC trūksta vs. trūksta paties engine mechanizmo),
  ne bendras „coming soon" ar vienas universalus perspėjimas — tikslesnė informacija vartotojui,
  kuris jau du kartus šioje sesijoje (Input, Cores) matė panašius apribojimus ir pagrįstai
  tikisi suprasti SKIRTUMĄ tarp jų.

**REALIAI patikrinta:** `cargo build/clippy/fmt -p nullbyte-app` švarūs (įsk. naują `cpal`
priklausomybę kompiliuojant `nullbyte-app`), `cargo test --workspace` 84/84 (3 nauji testai:
`video_settings_default_matches_renderer_defaults`, `video_settings_roundtrips_through_json`,
`audio_settings_default_is_system_default_device_full_volume`), `pnpm check/lint/build` 0
klaidų/warning'ų. Realaus garso įrenginių sąrašo (ar jame yra teisingi pavadinimai šioje
aplinkoje) UI patvirtinimas — vartotojo atsakomybė per `pnpm tauri dev`, kaip visada šioje
sesijoje.

**P7.6 UŽBAIGTA** (visos 6 kortelės įgyvendintos) — žr. Progreso lentelę §12 (Faza 7 dabar
100%).

---

### ADR-026 — Xbox valdiklio realus patikrinimas: D-pad-per-ašį bug'as + P4.0.2 test hook'o pašalinimas (P4.1/P4.2/P4.3)
**Data:** 2026-08-26 · **Statusas:** priimta

**Kontekstas:** vartotojas gavo Xbox Wireless Controller prieigą (prijungtas prie Mac'o) —
proga pagaliau realiai patikrinti P4.1/P4.2/P4.3 punktus, kurie nuo 2026-08-21/25 buvo
pažymėti `[!]` dėl trūkstamo antro valdiklio tipo. Kadangi tiesioginė sąveika su native
langu šioje sesijoje nepatikima (žr. atmintį), verifikacija atlikta DVIEM ETAPAIS: (1)
grynas `gilrs`-lygio signalo patikrinimas be jokio winit lango (laikinas `cargo run
--example` diagnostikos skriptas `nullbyte-core/examples/gamepad_probe.rs`, ištrintas po
naudojimo — pakartotinai naudoja JAU esančius `GamepadThread`/`default_gamepad_mapping`,
jokio naujo domeno kodo), (2) realaus žaidimo patvirtinimas vartotojo PAČIO terminale.

**Radinys #1 — D-pad siunčiamas kaip ašis, ne mygtukas:** diagnostikos skriptas parodė, kad
šis Xbox valdiklis macOS D-pad'ą siunčia IŠIMTINAI kaip `gilrs::EventType::AxisChanged`
(`Axis::DPadX`/`DPadY`, švarios `-1.0`/`0.0`/`1.0` reikšmės — funkciškai skaitmeninis „hat
switch", tiesiog kitu gilrs API keliu nei `Button::DPad*`), NIEKADA kaip
`ButtonChanged(DPadUp/Down/Left/Right)`. `nullbyte-emu` realaus žaidimo įvesties kelias
(`drain_gamepad_events`) iki šiol SĄMONINGAI ignoravo `AxisChanged` (P4.2 „Ką daryti" prašė
tik mygtukų mapping'o) — praktiškai tai reiškė, kad D-pad ŠIUO valdikliu būtų VISIŠKAI
neveikęs realiame žaidime. Empiriškai patikrintas ir ženklas: `DPadY = +1.0` → UP,
`-1.0` → DOWN (priešinga standartinei ekrano/analoginio stiko Y konvencijai).

**Sprendimas #1:** nauja `mapping::dpad_axis_ids(Axis) -> Option<(u32, u32)>`
(teigiamos/neigiamos krypties `RETRO_DEVICE_ID_JOYPAD_*`) + `AXIS_DPAD_THRESHOLD = 0.5` —
`nullbyte-emu` `drain_gamepad_events` dabar apdoroja `AxisChanged` LYGIAI TAIP PAT kaip
`ButtonChanged` (nustato/išvalo teisingą bitą pagal ženklą). Du nauji unit testai
(`dpad_axis_ids_match_empirically_observed_sign_convention`, `non_dpad_axes_are_not_mapped`).
Analoginiai stikai TYČIA lieka neatvaizduoti (RetroPad D-pad skaitmeninis).

**Radinys #2 — leftover P4.0.2 test hook'as lenktyniauja su realiomis komandomis:**
tikrinant D-pad fix'ą pirmuoju bandymu (mano paties FIFO-pagrįstas rankinis paleidimas),
žaidimas neatsivėrė vaizde (juodas langas). Tyrimas atskleidė `nullbyte-emu` `resumed()`
viduje VISADA suveikiantį hardkodintą `test_core_and_rom()` hook'ą (P4.0.2 laikina
scaffolding'o liekana, kurios modulio doc PATS sakė turėjo būti pašalinta, kai P4.0.3
atnešė realų IPC `Load` srautą — bet nebuvo). Šis hook'as SIŲSDAVO SAVO `Load` komandą
VISADA lango atidarymo metu, LENKTYNIAUDAMAS su bet kokia REALIA `Load` komanda iš IPC —
dvi `Load` komandos be tarpinio `retro_unload_game()` pažeidžia CLAUDE.md §3.2 taisyklę #2
(„vienu metu tik VIENAS core"). **Pašalinta pilnai** (`test_core_and_rom()` funkcija +
kvietimo vieta `resumed()`).

**Svarbu:** Radinio #2 pašalinimas PATS SAVAIME NEIŠSPRENDĖ juodo lango — dar vienas
bandymas (agento fono procesas, be test hook'o) IR TOLIAU rodė juodą langą. Šaknis paaiškėjo
esanti MANO PAČIO testavimo aplinkoje: `cargo run`/procesas, paleistas per agento fono Bash
įrankį (ne interaktyvi vartotojo terminalo sesija), matyt neturi tinkamos CoreAudio/lango
sesijos prieigos, reikalingos audio-driven pacing kilpai judėti į priekį (žr. CLAUDE.md
§8.5) — `audio_buffer_occupancy` liko `0.0` amžinai. Kai VARTOTOJAS PATS paleido TĄ PATĮ
komandų rinkinį savo terminale — žaidimas atsivėrė IŠKART, D-pad veikė visomis 4 kryptimis.
**Pamoka:** agento paties bandymas paleisti/testuoti GUI+audio procesus fone yra NEPATIKIMAS
signalo šaltinis šiam projektui — tikras patvirtinimas visada turi ateiti iš vartotojo
paties interaktyvios sesijos (žr. atmintį „native window input").

**REALIAI patikrinta:** `cargo fmt/clippy --workspace -D warnings`, `cargo test --workspace`
(visi testai, įsk. 2 naujus), IR realus žaidimas (ActRaiser, SNES, Xbox valdiklis, vartotojo
paties terminale) — visi mygtukai + D-pad veikia teisingai.

---

### ADR-027 — Pirmas realus Linux patikrinimas: pilnas workspace + ALSA buferio dydžio bug'as (P2.3/P3.1/P4.1)
**Data:** 2026-08-26 · **Statusas:** priimta

**Kontekstas:** vartotojas turi SSH prieigą prie Arch Linux mašinos („omarchy", Hyprland/
Wayland darbastalis, AMD GPU) — pirma proga per VISĄ projekto istoriją realiai patikrinti
Linux, ne vien pasikliauti CI (`ubuntu-latest`, be display serverio/GPU/audio, žr. P0.4).
Mašinoje NEBUVO nei Rust toolchain, nei repo — abu įdiegti šios sesijos metu (`rustup`,
`git clone` iš PUBLIC GitHub repo). Sisteminiai paketai (`webkit2gtk-4.1`, `gtk3`,
`libappindicator-gtk3`, `librsvg`, `alsa-lib`, `patchelf`, `openssl`, `base-devel`) ir
`libretro-snes9x` (Arch `extra` repo turi paruoštus libretro core'us — NEREIKĖJO kompiliuoti
iš šaltinio) įdiegti PAČIO VARTOTOJO (reikėjo `sudo`, agentas neturėjo slaptažodžio).

**Radinys #1 (teigiamas) — VISAS workspace kompiliuojasi ir testuojasi švariai Linux'e
PIRMĄ KARTĄ:** `cargo fmt --check`, `cargo clippy --workspace --all-targets -D warnings`,
`cargo test --workspace` (84+80+4 = TIKSLIAI tas pats testų skaičius kaip macOS, 0 failed) —
VISI švarūs be jokių pakeitimų kode. `nullbyte-app` (su Tauri/webkit2gtk/GTK priklausomybėmis)
IR `nullbyte-emu` (su winit/wgpu/cpal) abu sukompiliavo be klaidų. Tai anuliuoja daugybę senų
„Linux — NEPATIKRINTA" pastabų P2.x/P3.x/P4.x acceptance sąrašuose (žr. jas individualiai,
sweep'intas visas MVP.md šia sesija).

**Radinys #2 (neigiamas, PATAISYTAS) — ALSA atmeta `BufferSize::Fixed`:** paleidus
`nullbyte-emu` su realiu Wayland langu (tikra vartotojo grafinė sesija — `WAYLAND_DISPLAY`/
`XDG_RUNTIME_DIR` iš `/run/user/<uid>`, SSH kaip TAS PATS vartotojas), langas atsivėrė
(realus wgpu Vulkan Surface, `adapter="AMD Radeon Graphics (RADV RENOIR)"`), BET liko juodas
— `audio_buffer_occupancy` amžinai `0.0`. `RUST_LOG=debug` atskleidė tikrą priežastį:
`nepavyko sukurti audio srauto: ... ALSA function 'snd_pcm_hw_params_set_buffer_size' failed
with error 'Invalid argument (22)'` — `audio/output.rs` PRIVERSTINIAI prašė
`cpal::BufferSize::Fixed(buffer_frames)` (apskaičiuoto iš `TARGET_LATENCY_MS = 50`), o šios
mašinos PipeWire/ALSA stack'as tokį TIKSLŲ dydį atmetė (ALSA hw_params derybos turi
papildomų period/buffer santykio apribojimų, kurių `cpal`'o pranešta `SupportedBufferSize`
riba PATI SAVAIME negarantuoja). Kadangi emuliavimo pacing YRA audio-driven (CLAUDE.md
§8.5), garso srauto neatsidarymas reiškė VISIŠKĄ emuliacijos sustojimą — juodas langas,
procesas gyvas, bet nė vieno `retro_run()` niekada neįvyko.

**Sprendimas:** `config.buffer_size = cpal::BufferSize::Default` vietoj `Fixed(buffer_frames)`
— leidžia backend'ui (ALSA/PipeWire/CoreAudio) PAČIAM parinkti TIKRAI veikiantį dydį.
SAUGU macOS atžvilgiu (jau patikrinta anksčiau su `Fixed`, dabar Default irgi veikia — žr.
patikrinimą žemiau) IR nekeičia `audio::ring::recommended_capacity()` (nullbyte'o PAČIO
lock-free žiedinio buferio, atskirto nuo OS lygio cpal srauto buferio, dydžio) — ta funkcija
IR TOLIAU naudoja `TARGET_LATENCY_MS` NEPRIKLAUSOMAI. `buffer_frames` KINTAMASIS liko (vis
dar naudojamas `scratch_capacity` — vietinio, ne-OS, apsauginio buferio — dydžiui).

**REALIAI patikrinta po pataisymo:** perkompiliuota IR macOS (`cargo test -p nullbyte-core
audio::output` švarus, `cargo clippy --workspace` švarus), IR Linux (rsync'intas pakeitimas,
perkompiliuota omarchy). Linux'e: log „cpal audio srautas paleistas" BE klaidos
(`sample_rate=44100`), realus emuliavimo ciklas veikė `measured_fps≈51.4`,
`audio_occupancy≈0.62` (sveikas, arti tikslinio 50%). Vartotojas REALIAI matė žaidimą
(ActRaiser) savo fiziniame Wayland ekrane IR patvirtino klaviatūros valdymą veikiant
(„taip"/„taip"). Šis fix'as taip pat, tikėtina, paaiškina panašią (bet TADA nediagnozuotą)
simptomatiką ankstesniuose šios sesijos macOS bandymuose per agento fono procesą (žr.
[[feedback_background_launched_emu_no_audio]]) — ten priežastis liko nepatvirtinta (CoreAudio
sesijos apribojimas buvo geriausia hipotezė, ne įrodytas faktas), bet ta pati klasė bug'o
(garso srautas neatsidaro → pacing sustoja → juodas langas) dabar Linux'e TURI konkretų,
atkuriamą, pataisytą pavyzdį.

**Sąmoningai NEPADARYTA šioje sesijoje** (žinomi likę apribojimai): X11 (tik Wayland
testuota), realus gamepad Linux'e (jokio valdiklio neturėta ant omarchy), Tauri app'o pati
UI/native dialog'ai Linux'e (testuota TIK `nullbyte-emu` tiesiogiai, ne `pnpm tauri dev`),
fullscreen toggle Linux'e.

### ADR-028 — Save state'ų `states_dir` architektūra + ranka rašytas PNG encoder'is (P8.1)
**Data:** 2026-08-26 · **Statusas:** priimta

**Kontekstas:** P8.1 reikalauja, kad `nullbyte-emu` (vaikas, DB-oblivious pagal ADR-016) sugebėtų
išsaugoti/įkelti save state'us TAM TIKRAM žaidimui, VEIKDAMAS TIK per hotkey (F5-F8/Shift+F5-F8,
MVP.md P4.4) — jokio Tauri IPC round-trip'o hotkey paspaudimo metu nėra ir neplanuojama.

**Sprendimas #1 (`EmuCommand::Load.states_dir`):** pirminis bandymas buvo padaryti
`SaveState`/`LoadState` struct-like variantais su pilnu `path`/`thumb_path` iš tėvo KIEKVIENAM
hotkey paspaudimui — atmesta, nes realus kviečiantysis (`nullbyte-emu` hotkey handleris) NETURI
tėvo po ranka tuo metu. Vietoj to `EmuCommand::Load` gavo naują PRIVALOMĄ `states_dir: PathBuf`
lauką — tėvas išsprendžia žaidimo katalogą VIENĄ kartą, `Load` metu; vaikas tada patį `{slot}.state`/
`.png` kelią sudaro LOKALIAI (`states_dir.join(...)`), be jokio IPC papildomai. `SaveState(u8)`/
`LoadState(u8)` liko PAPRASTI `(slot)` tuple variantai — nulinis laido pokytis jiems patiems.
Kadangi `Load` gavo naują PRIVALOMĄ lauką, `IPC_PROTOCOL_VERSION` pakelta į `2` (žr. `ipc.rs` doc).

**Sprendimas #2 (ranka rašytas PNG encoder'is vietoj `png` crate priklausomybės):** vartotojui
pasiūlyta rinktis tarp `png` crate (pilnai funkcionalus, bet nauja produkcinė priklausomybė) ir
minimalaus ranka rašyto encoder'io (PNG signatūra + IHDR/IDAT/IEND, `crc32fast` — JAU esama
priklausomybė — per chunk'ą, zlib apvalkalas su DEFLATE „stored" (nekompresuotais) blokais, ranka
rašytas Adler-32). Vartotojas pasirinko ranka rašytą variantą (žr. ADR-021 precedentą — panašus
sprendimas anksčiau). `png` crate PRIDĖTAS TIK kaip **dev-dependency** — vienintelis naudojimas
testuose, roundtrip'inant encoder'io išvestį per REALŲ, patikimą decoder'į (ta pati kategorija
kaip `zip` dev-dependency `nullbyte-app` teste fixture'ams kurti). Produkciniame binare `png`
crate NIEKADA nekompiliuojamas.

**Patikrinta:** `cargo test --workspace` — 88 nullbyte-core testai (0 failed), įskaitant
`png_encoder::tests::roundtrips_through_a_real_png_decoder` (17×13 šachmatų lentelė) ir
`roundtrips_a_frame_larger_than_one_stored_block` (300×300, priverčia multi-block DEFLATE kelią),
bei `savestate::tests::save_then_load_on_a_fresh_core_restores_identical_state` (realus snes9x
core + realus SNES ROM, NAUJAS `CoreHandle` simuliuoja procesą iš naujo, `serialize()` išvestis
identiška baitas-į-baitą). `cargo clippy --workspace --all-targets -D warnings` švarus.
`crates/nullbyte-app/src/db/save_states.rs` (CRUD, `UNIQUE(game_id, slot)` upsert) — 5 testai,
visi praeina, bet modulis kol kas be kviečiančiojo (`#![allow(dead_code)]`, žr. P8.1 pastabą) —
laukia P9.1 paleidimo pipeline'o, kad būtų iš ko realiai kviesti.

### ADR-029 — SRAM: atskiras `core::sram` modulis + antras `IPC_PROTOCOL_VERSION` pakėlimas (P8.2)
**Data:** 2026-08-26 · **Statusas:** priimta

**Kontekstas:** P8.2 (in-game save'ai, CLAUDE.md §8.8) pirminiame MVP.md juodraštyje buvo
priskirtas TAM PAČIAM failui kaip P8.1 (`core/savestate.rs`). CLAUDE.md §8.8 pati eksplicitiškai
sako „Atskirai nuo save state'ų" — patikrinus abi operacijas paaiškėjo, kad jos NIEKUR
nesikerta kode (skirtingi libretro simboliai — `retro_get_memory_data`/`size`, ne
`retro_serialize`/`unserialize`; skirtinga panaudojimo semantika — SRAM atsinaujina
PROGRESYVIAI visos sesijos metu, save state'as „užšaldo" VIENĄ tašką), tad sukurtas ATSKIRAS
`crates/nullbyte-core/src/core/sram.rs` modulis. Bendra tik `savestate::write_atomic` (dabar
`pub(super)`) — abu moduliai naudoja TĄ PATĮ `.tmp` → `rename` šabloną, nėra prasmės dubliuoti.

**`CoreHandle::sram()`/`sram_mut()`:** `&[u8]`/`&mut [u8]` tiesiogiai virš core'o
`retro_get_memory_data`/`size()` grąžintos rodyklės — `None`, jei `size == 0` ARBA rodyklė
`NULL` (daug core'ų, pvz. arcade, SRAM tiesiog neturi — CLAUDE.md §8.8 tai numato kaip normalų
atvejį, ne klaidą). `sram_mut()` reikalavo `#[allow(clippy::mut_from_ref)]` — clippy įtaria
klasikinį aliasing pažeidimą (`&mut` kilęs iš `&self`), bet realybėje `&self` tik suteikia
prieigą prie core'o SYMBOLS funkcijų rodyklių, ne prie pačių duomenų; realų aliasing draudimą
užtikrina CLAUDE.md §3.2 taisyklė #1 (visi `retro_*` kvietimai TIK iš emuliavimo gijos, tad
niekada nėra dviejų gyvų šio buferio nuorodų vienu metu).

**`EmuCommand::Load.sram_path` + trečias `IPC_PROTOCOL_VERSION` pakėlimas:** analogiškai P8.1
`states_dir` sprendimui — bet SKIRTINGAI: SRAM turi TIK VIENĄ failą vienam žaidimui (ne
sloto-priklausomą pavadinimą), tad tėvas siunčia PILNĄ, jau išspręstą `sram_path: PathBuf`
(ne katalogą, iš kurio vaikas pats sudarytų pavadinimą — vaikas neturėtų spėlioti
`{rom_basename}` iš ROM'o kelio, kuris gali būti archyvo viduje ar turėti neįprastų simbolių).
Kadangi tai DAR VIENAS naujas PRIVALOMAS laukas `Load` variante, `IPC_PROTOCOL_VERSION` pakelta
iš `2` į `3` (žr. `ipc.rs` doc — antras šio konstantos pakėlimas per tą pačią dieną, abu kartus
dėl TO PATIES `EmuCommand::Load` varianto, bet skirtingų laukų).

**Periodinio flush'o dirty-check:** MVP.md „Ką daryti" reikalauja įrašyti „kas 30 s IR kai
turinys pasikeitė" — įgyvendinta kaip `RunnerState.last_saved_sram: Option<Vec<u8>>`
(paskutinio SĖKMINGAI įrašyto turinio kopija). Kas `SRAM_SAVE_INTERVAL` (30s) `run_loop`
palygina dabartinį `core.sram()` su šia kopija — jei sutampa, PRALEIDŽIA rašymą (vengia
nereikalingo disko I/O, kai žaidėjas tiesiog stovi meniu ar neišsaugo). **Uždarant žaidimą**
(`cleanup`) šis dirty-check SĄMONINGAI APEINAMAS — visada įrašoma BESĄLYGIŠKAI, PRIEŠ
`unload_game()`/`deinit()`, kad paskutinės (galimai < 30s senumo) žaidėjo progreso sekundės
niekada nebūtų prarastos vien todėl, kad periodinis taimeris dar nespėjo suveikti.

**Patikrinta:** `cargo test --workspace` — 90 nullbyte-core testų (0 failed, +2 nuo P8.1),
įskaitant `core::sram::tests::save_then_load_on_a_fresh_core_restores_identical_sram_prefix`
(realus snes9x core + realus SNES ROM, KURIS REALIAI praneša SRAM `size > 0` — ne dirbtinis
mock'as; rankiniu būdu užrašomas atpažįstamas baitų šablonas TIESIOG į core'o SRAM, `save_sram`
→ NAUJAS `CoreHandle` → `load_sram` → baitas-į-baitą sutampa) ir
`load_sram_with_missing_file_is_a_silent_noop_not_an_error` (nauja sesija be ankstesnio
`.srm` — NĖRA klaida, skirtingai nuo `savestate::load_state`, kur trūkstamas failas YRA
klaida, nes vartotojas eksplicitiškai paprašė konkretaus slot'o). `cargo clippy --workspace
--all-targets -D warnings` švarus. **NEpatikrinta:** realus end-to-end per gyvą procesą/
tikrą in-game save meniu — laukia P9.1 (ta pati situacija kaip P8.1, žr. ADR-028).

### ADR-030 — Žaidimo paleidimo srautas: `EmuClient` oneshot handshake + `game_id`-keyed keliai (P9.1)
**Data:** 2026-08-26 · **Statusas:** priimta

**Kontekstas:** P9.1 pirmą kartą realiai sujungia core'o pasirinkimą (P7.6), žaidimo DB
įrašą (P5.4) ir `nullbyte-emu` vaiko procesą (`crate::ipc::EmuClient`, veikęs nuo P4.0.3 tik
kaip handshake+fire-and-forget siųstuvas). Iki šiol `EmuClient::spawn` grąžindavo `Result<Self,
AppError>` IŠKART po sėkmingo protokolo handshake'o — jokio būdo sužinoti, ar VĖLIAU nusiųstas
`Load` realiai pavyko, ar core'as atmetė ROM'ą/trūko BIOS'o (ta informacija atkeliauja
ASINCHRONIŠKAI, per `EmuStatus::Error` stdout eilutę).

**Sprendimas #1 (`oneshot` handshake pirmam Loaded/Error):** `EmuClient::spawn` dabar grąžina
`(Self, oneshot::Receiver<EmuStatus>)` — `drain_loop` (VISADA veikianti fono užduotis, žr.
`crate::ipc` modulio doc) PIRMĄ gautą `EmuStatus::Loaded`/`Error` nusiunčia per šį kanalą, o
`commands::emulator::start_game` juo REALIAI LAUKIA (`.await`), prieš grąžindama rezultatą
UI. Tai paverčia P9.1 acceptance „Trūkstamas core → suprantamas pranešimas" TIKRAI TIESA net
klaidoms, kurios paaiškėja TIK core'ui bandant įkelti ROM'ą (ne vien akivaizdžioms spawn-metu
klaidoms). VĖLESNI (po pirmojo atsakymo) `EmuStatus::Error` — persiunčiami į frontend'ą kaip
Tauri event'as `"game-error"`, PAKARTOTINAI naudojant JAU egzistuojantį `AppError`'io
`{kind, message}` Serialize impl'ą (konvertuota per `serde_json::Value`, nes `Emitter::emit`
reikalauja `Serialize + Clone`, o `AppError` pati NĖRA `Clone` — apgaubia `std::io::Error`
ir pan.) — jokio naujo suplokštinimo tipo/kodo.

**Sprendimas #2 (`on_terminated` callback proceso pabaigai):** `EmuClient::spawn` dabar taip
pat ima `on_terminated: impl FnOnce() + Send + 'static`, kviečiamą TIKSLIAI VIENĄ kartą, kai
vaiko procesas PILNAI baigia darbą (`CommandEvent::Terminated` ARBA kanalas užsidaro be jo).
Tai VIENINTELIS patikimas „žaidimo sesija pasibaigė" signalas (CLAUDE.md §10: NE PID
pollinimas) — `start_game` juo `db::games::record_play(id, elapsed_seconds)` ir atlaisvina
`AppState::emu_session`, kad kitas `start_game` galėtų vėl paleisti. Veikia NEPRIKLAUSOMAI
nuo to, KAIP procesas baigėsi — vartotojas uždarė `nullbyte-emu` langą PATS (winit
`WindowEvent::CloseRequested` → `event_loop.exit()` → `process::exit()`), ARBA tėvas iškvietė
`shutdown_gracefully()` (stdin EOF, tas pats mechanizmas kaip P4.0.4 orphan apsauga).

**Sprendimas #3 (`game_id`-keyed `states_dir`/`sram_path`, ne `rom_basename`):** MVP.md P8.1/
P8.2 juodraštis siūlė `states_dir()/{game_id}_{slot}.state` (jau `game_id`-keyed) ir
`saves_dir()/{rom_basename}.srm` (NE) — pastarasis būtų turėjęs kolizijos riziką (du žaidimai
skirtinguose kataloguose gali dalintis TUO PAČIU ROM failo vardu). `paths::game_states_dir`/
`game_sram_path` (nauji) abu naudoja `game_id` — DB primary key, garantuotai unikalus ir
stabilus, skirtingai nuo failo vardo.

**Sprendimas #4 (viena sesija vienu metu, aiški klaida — ne tylus pakeitimas):** ADR-016
numato VIENĄ vaiko procesą vienam paleidimui, bet NEIŠSPRENDŽIA, kas vyksta, jei
`start_game` iškviečiamas, kol ankstesnė sesija dar veikia. P9.1 apimtyje pasirinkta
PAPRASČIAUSIA, saugiausia elgsena: antras kvietimas grąžina aiškią klaidą („žaidimas jau
paleistas — pirma uždarykite jį"), NE tyliai nutraukia/pakeičia senąją sesiją. Kelių vienu
metu veikiančių žaidimų palaikymas — NEĮTRAUKTAS į MVP apimtį (žr. §1.3 „NEĮEINA").
`stop_game`/`is_game_running` komandos pridėtos kaip atsarginis kelias (naudoja TIK
`shutdown_gracefully()` — jokio „force kill" UI veiksmo šioje sesijoje neprašyta).

**Patikrinta:** `cargo test --workspace` — nullbyte-app 93 testai (0 failed, +4 nuo P8.2:
`get_platform_slug`, du `resolve_preferred_core_path`, per-game `paths` testas).
`cargo clippy --workspace --all-targets -D warnings` švarus. `pnpm check`/`lint`/`build`
švarūs. `pnpm run build:sidecar` perkompiliuotas (IPC laidas nepakito nuo P8.2 —
`IPC_PROTOCOL_VERSION` LIEKA `3`, P9.1 nepridėjo naujų PRIVALOMŲ `EmuCommand` laukų). **REALIAI
patikrinta VARTOTOJO (2026-08-26):** `pnpm tauri dev` paleista, „Play" paspaustas tikru
žaidimu bibliotekoje — pilnas ciklas (core parinkimas → `nullbyte-emu` spawn → handshake →
`Load` → `EmuStatus::Loaded` → langas rodo žaidimą) suveikė realiai, vartotojo patvirtinta
kaip „veikia puikiai". `play_time`/`last_played` fiksavimo (`on_terminated` → `record_play`)
ir „game-closed" event'o atskirai, per stebimą DB įrašo pasikeitimą, NEpatvirtinta — tik
netiesiogiai per `db::games::record_play_increments_count_and_time` vieneto testą.

---

## 15. Po MVP — idėjų sąrašas

> **Nedaryk nieko iš šio sąrašo, kol MVP nebaigtas.**
> Naujos idėjos rašomos čia, ne į fazių planą.

**v0.2 — Gilesnis emuliavimas**
- **Arcade (MAME-tipo) ROM setų palaikymas** (žr. ADR-023, 2026-08-26). Dabar Arcade
  platforma turi TUŠČIĄ `extensions` sąrašą — VISIŠKAI neatpažįsta jokio ROM'o, nes MAME
  romsetai yra kelių žalio chip dump'o failų rinkinys `.zip` viduje, be bendro atpažįstamo
  plėtinio, o `nullbyte-core/src/archive.rs::extract_first_match` dabar veikia TIK „rask VIENĄ
  failą archyve pagal plėtinį" modeliu (tinka GBA/Neo Geo vieno-failo formatams, netinka
  MAME). Reikėtų naujos logikos: arba (a) visą `.zip` traktuoti kaip ROM tapatybę pagal PATĮ
  archyvo vardą/hash'ą, netraukiant nė vieno vidinio failo, arba (b) MAME-specifinio ROM
  identifikavimo (DAT failai/CRC pagal chip'ą) — abu žymiai sudėtingesni nei esamas paprastas
  modelis.
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
