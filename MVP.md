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
Failai: src-tauri/src/kelias/i/faila.rs
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
- `src-tauri/tauri.conf.json`: `productName: "Nullbyte"`, `identifier: "fr.nullbyte.app"`,
  `version: "0.1.0"`

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
**Failai:** `src-tauri/Cargo.toml`, `src-tauri/src/**/mod.rs`

**Ką daryti:**
- Į `Cargo.toml` sudėk visas priklausomybes iš `CLAUDE.md` §2
- Sukurk tuščius modulius pagal `CLAUDE.md` §4 struktūrą (kiekvienas `mod.rs` su `//!` doc)
- `error.rs`: `AppError` enum su `thiserror`, `impl serde::Serialize` (kad kirstų IPC)
- `paths.rs`: funkcijos `data_dir()`, `cores_dir()`, `system_dir()`, `saves_dir()`,
  `states_dir()`, `media_dir()`, `db_path()` — su `directories` crate arba rankiniu būdu
  pagal `CLAUDE.md` (macOS: `~/Library/Application Support/Nullbyte`, Linux: XDG)
- `state.rs`: `AppState` struct (kol kas tuščias)

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
  `src-tauri/cores/`, `src-tauri/gen/`, `*.srm`, `*.state`
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
**Failai:** `src-tauri/src/lib.rs`

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
**Failai:** `src-tauri/src/core/ffi.rs`

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
**Failai:** `src-tauri/src/core/loader.rs`

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
**Failai:** `src-tauri/src/core/info.rs`

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
**Failai:** `src-tauri/src/core/callbacks.rs`

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
**Failai:** `src-tauri/src/core/environment.rs`

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

### P1.6 — ROM įkėlimas `[ ]`

**Priklausomybės:** P1.5
**Failai:** `src-tauri/src/core/loader.rs`, `src-tauri/src/library/archive.rs`

**Ką daryti:**
- `load_game(rom_path)`:
  - jei `need_fullpath == true` → paduok kelią, `data = NULL`
  - jei `false` → įkelk failą į atmintį, paduok `data` + `size`
  - archyvams (`.zip`/`.7z`): išpakuok pirmą tinkamą plėtinį į atmintį
    (jei `need_fullpath` — išpakuok į temp failą)
- `retro_get_system_av_info()` **po** `load_game` → įsimink `fps`, `sample_rate`, `geometry`
- `unload_game()` + `deinit()` teisinga tvarka

**Acceptance:**
- [ ] SNES `.sfc` įkeliamas, `av_info.timing.fps ≈ 60.098`
- [ ] `.zip` su `.nes` viduje įkeliamas
- [ ] PS1 core su `need_fullpath` gauna kelią, ne buferį
- [ ] Blogas ROM → `AppError`, ne crash

---

### P1.7 — Emuliavimo gija ir headless loop 🔴 `[ ]`

**Priklausomybės:** P1.6
**Failai:** `src-tauri/src/core/runner.rs`

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

**Acceptance:**
- [ ] Paleidžia SNES ROM'ą ir 60 sekundžių sukasi be crash'o
- [ ] Log rodo ~60 FPS (±1)
- [ ] `Stop` komanda sustabdo švariai, be memory leak (patikrink Activity Monitor / `htop`)
- [ ] Video callback kviečiamas ~60 k./s (skaitliukas log'e)
- [ ] Audio callback duoda ~32040 sample/s SNES atveju

> **Milestone M1:** čia turi būti aišku, kad libretro integracija veikia.
> Jei ne — sustok ir spręsk, prieš eidamas į Fazę 2.

---

## 4. Faza 2 — Vaizdas 🔴

**Tikslas:** matomas žaidimo vaizdas lange.
**Rizika:** 🔴 didelė (wgpu + Tauri + platformų skirtumai). **Įvertis:** 3–4 dienos.

### P2.1 — Pikselių formatų konversija `[ ]`

**Priklausomybės:** P1.4
**Failai:** `src-tauri/src/video/pixel_format.rs`

**Ką daryti:**
- `convert_to_rgba8(src: &[u8], format: PixelFormat, width, height, pitch) -> Vec<u8>`
- Palaikyk `RGB565`, `XRGB8888`, `0RGB1555`
- **Gerbk `pitch`** (baitais, ne pikseliais — dažniausia klaida)
- Optimizuok: pre-alokuotas išvesties buferis, ne `Vec` per kadrą

**Acceptance:**
- [ ] Unit testai visiems 3 formatams su žinomomis reikšmėmis
- [ ] Testas su `pitch > width * bpp` (padding'as) duoda teisingą rezultatą
- [ ] Benchmark: 256×224 RGB565 konversija < 0.5 ms

---

### P2.2 — Triple buffer tarp gijų `[ ]`

**Priklausomybės:** P2.1
**Failai:** `src-tauri/src/video/frame_buffer.rs`

**Ką daryti:**
- Trys buferiai + atominis indeksas: emu gija rašo į „write", UI gija skaito „read"
- Emu gija niekada nelaukia; UI gija visada gauna naujausią pilną kadrą
- Kadras neša metaduomenis: `width`, `height`, `generation`

**Acceptance:**
- [ ] Testas: 2 gijos, 10 000 kadrų, jokio data race (`cargo test` + `--release`)
- [ ] Emu gija niekada neblokuojasi (matuok `write_frame` trukmę)

---

### P2.3 — Emuliatoriaus langas ir wgpu surface 🔴 `[ ]`

**Priklausomybės:** P0.3, P2.2
**Failai:** `src-tauri/src/video/renderer.rs`, `src-tauri/src/commands/emulator.rs`

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
- [ ] Atsidaro antras langas, wgpu inicializuojasi be klaidų
- [ ] Veikia macOS (Metal) — patikrinta
- [ ] Veikia Linux X11 (Vulkan) — patikrinta
- [ ] Veikia Linux Wayland arba yra dokumentuotas apėjimas
- [ ] Lango dydžio keitimas nesulaužo surface'o

---

### P2.4 — Blit pipeline ir shader'is `[ ]`

**Priklausomybės:** P2.3
**Failai:** `src-tauri/src/video/renderer.rs`, `src-tauri/src/video/shaders/blit.wgsl`

**Ką daryti:**
- `wgpu::Texture` (RGBA8) → `queue.write_texture()` iš triple buffer
- Full-screen triangle vertex shader + sampled texture fragment shader
- Sampler: `Nearest` (numatytasis, pixel-perfect) ir `Linear` (nustatymuose)
- Render loop susietas su lango redraw įvykiu

**Acceptance:**
- [ ] **Matomas SNES žaidimo vaizdas** — pirmas tikras vizualus rezultatas
- [ ] Spalvos teisingos (palygink su RetroArch screenshot'u)
- [ ] Nėra tearing'o (vsync įjungtas — `PresentMode::AutoVsync`)

> **Milestone M2:** žaidimas matomas ekrane.

---

### P2.5 — Aspect ratio, scaling, fullscreen `[ ]`

**Priklausomybės:** P2.4
**Failai:** `src-tauri/src/video/renderer.rs`

**Ką daryti:**
- Gerbk `av_info.geometry.aspect_ratio` (jei 0 → `base_width / base_height`)
- Letterbox / pillarbox su juodais kraštais
- Integer scaling režimas (nustatymuose)
- Fullscreen perjungimas: `F11` ir `Cmd+Ctrl+F` (macOS)
- `Esc` išeina iš fullscreen

**Acceptance:**
- [ ] SNES 4:3 vaizdas neištemptas 16:9 lange
- [ ] Integer scaling duoda ryškius pikselius be interpoliacijos artefaktų
- [ ] Fullscreen veikia abiejose platformose

---

## 5. Faza 3 — Garsas 🔴

**Tikslas:** garsas be traškesių, sinchronizuotas su vaizdu.
**Rizika:** 🔴 didelė (real-time constraints). **Įvertis:** 2–3 dienos.

### P3.1 — cpal išvesties srautas `[ ]`

**Priklausomybės:** P0.3
**Failai:** `src-tauri/src/audio/output.rs`

**Ką daryti:**
- Numatytasis įrenginys, `f32` arba `i16` formatas, stereo
- Buferio dydis: taikyk ~40–60 ms latency
- Klaidos callback → `tracing::error!` (ne panic)
- Įrenginio dingimas (ausinių atjungimas) → atkūrimas, ne crash

**Acceptance:**
- [ ] Sinusoidė 440 Hz groja švariai 30 s
- [ ] Ausinių atjungimas/prijungimas neuždaro programos
- [ ] Veikia macOS (CoreAudio) ir Linux (ALSA/PipeWire)

---

### P3.2 — Lock-free ring buffer `[ ]`

**Priklausomybės:** P3.1
**Failai:** `src-tauri/src/audio/ring.rs`

**Ką daryti:**
- `rtrb::RingBuffer<i16>` — producer emu gijoje, consumer cpal callback'e
- Talpa ≈ 4× buferio dydis
- Underrun → užpildyk tyla + `tracing::warn!` (throttled, ne per kadrą)
- Overrun → mesk seniausius sample'us
- `occupancy()` metodas rate control'ui

**Acceptance:**
- [ ] Jokio alokavimo cpal callback'e (patikrink kodą; `cargo` be `dhat` pakanka vizualiai)
- [ ] Underrun/overrun nesulaužo srauto
- [ ] Testas: producer/consumer skirtingais greičiais 60 s

---

### P3.3 — Resampling `[ ]`

**Priklausomybės:** P3.2
**Failai:** `src-tauri/src/audio/resampler.rs`

**Ką daryti:**
- `rubato::SincFixedIn` arba `FastFixedIn`: `av_info.timing.sample_rate` → įrenginio rate
- Testuok: 32040 → 48000 (SNES), 44100 → 48000 (Genesis), 32768 → 48000 (GBA)
- Resampling vyksta **emu gijoje**, ne audio callback'e

**Acceptance:**
- [ ] SNES garsas skamba teisingu tonu (ne per aukštai/žemai)
- [ ] Nėra aliasing artefaktų
- [ ] Resampling < 1 ms per kadrą

---

### P3.4 — Dynamic rate control ir audio-driven sync 🔴 `[ ]`

**Priklausomybės:** P3.3, P1.7
**Failai:** `src-tauri/src/audio/resampler.rs`, `src-tauri/src/core/runner.rs`

**Ką daryti:**
- Formulė iš `CLAUDE.md` §8.6: koreguok resampling ratio pagal buffer occupancy
- `MAX_DELTA = 0.005` (0.5 %) — nepastebima ausiai
- **Pakeisk P1.7 frame pacing'ą**: emu gija nebemiega fiksuotą laiką, o laukia,
  kol ring buffer'yje atsiras vietos → garsas tampa laikrodžiu
- Fast-forward režimas: išjunk rate control, mesk audio sample'us

**Acceptance:**
- [ ] **10 minučių SNES žaidimo be vieno traškesio** — pagrindinis testas
- [ ] Buffer occupancy svyruoja apie 50 %, nedreifuoja į 0 % ar 100 %
- [ ] Vaizdas ir garsas nesiskiria (lūpų sinchronizacija testuojama žaidime su kalba)
- [ ] Fast-forward veikia be crash'o

> **Milestone M3:** žaidimas veikia su vaizdu ir garsu.

---

## 6. Faza 4 — Įvestis

**Tikslas:** valdyti žaidimą gamepad'u ir klaviatūra.
**Rizika:** 🟡 vidutinė. **Įvertis:** 2 dienos.

### P4.1 — Gamepad aptikimas `[ ]`

**Priklausomybės:** P0.3
**Failai:** `src-tauri/src/input/gamepad.rs`

**Ką daryti:**
- `gilrs::Gilrs` event pump; polling emu gijoje arba atskiroje gijoje su kanalu
- Prijungimo/atjungimo įvykiai → pranešk UI per Tauri event
- Analoginių ašių deadzone (numatytoji 0.2)

**Acceptance:**
- [ ] Aptinka Xbox, DualShock 4/5, 8BitDo valdiklius
- [ ] Prijungimas veikiant nesulaužo (hot-plug)
- [ ] Veikia macOS ir Linux

---

### P4.2 — Įvesties mapping'as `[ ]`

**Priklausomybės:** P4.1
**Failai:** `src-tauri/src/input/mapping.rs`

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
**Failai:** `src-tauri/src/input/mod.rs`, `src-tauri/src/core/callbacks.rs`

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
**Failai:** `src-tauri/src/input/mod.rs`

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
**Failai:** `src-tauri/migrations/001_initial.sql`, `src-tauri/src/db/migrations.rs`, `db/models.rs`

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
**Failai:** `src-tauri/src/library/hasher.rs`

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
**Failai:** `src-tauri/src/library/scanner.rs`

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
**Failai:** `src-tauri/src/db/games.rs`, `src-tauri/src/commands/library.rs`

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
**Failai:** `src-tauri/src/scraper/screenscraper.rs`, `scraper/types.rs`

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
**Failai:** `src-tauri/src/scraper/rate_limit.rs`, `scrape_cache` lentelė

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
**Failai:** `src-tauri/src/scraper/media.rs`

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
**Failai:** `src-tauri/src/commands/scraper.rs`

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
**Failai:** `src-tauri/src/core/savestate.rs`

**Ką daryti:**
- `retro_serialize_size()` **prieš kiekvieną** išsaugojimą
- Failas: `states_dir()/{game_id}_{slot}.state`
- Metaduomenys DB: core pavadinimas + versija, laikas
- Preview paveiksliukas: paimk dabartinį kadrą iš triple buffer → PNG
- Įkeliant: jei core nesutampa — įspėjimas UI, bet leisk bandyti
- **Kviesk tik iš emuliavimo gijos, tarp `retro_run()`**

**Acceptance:**
- [ ] Save → uždaryti → paleisti → load → tas pats taškas
- [ ] 4 slot'ai + quick save nepersidengia
- [ ] Preview paveiksliukas teisingas
- [ ] Kito core state → įspėjimas, ne crash

---

### P8.2 — SRAM `[ ]`

**Priklausomybės:** P1.7
**Failai:** `src-tauri/src/core/savestate.rs`

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
| M1 | libretro core sukasi headless | 1 | 3–5 d. | ⬜ |
| M2 | Vaizdas ekrane | 2 | 3–4 d. | ⬜ |
| M3 | Vaizdas + garsas + valdymas | 3, 4 | 4–5 d. | ⬜ |
| M4 | Biblioteka su metaduomenimis | 5, 6 | 4–6 d. | ⬜ |
| M5 | **MVP** | 7, 8, 9 | 8–11 d. | ⬜ |

**Bendras įvertis: 23–32 darbo dienos** (vienam žmogui su Claude Code).

### Progreso lentelė

| Faza | Užduočių | Baigta | % |
|---|---|---|---|
| 0 — Pamatai | 5 | 5 | 100 % |
| 1 — libretro | 7 | 5 | 71 % |
| 2 — Vaizdas | 5 | 0 | 0 % |
| 3 — Garsas | 4 | 0 | 0 % |
| 4 — Įvestis | 4 | 0 | 0 % |
| 5 — DB / biblioteka | 4 | 0 | 0 % |
| 6 — ScreenScraper | 4 | 0 | 0 % |
| 7 — UI | 6 | 0 | 0 % |
| 8 — Išsaugojimai | 2 | 0 | 0 % |
| 9 — Polish | 6 | 0 | 0 % |
| **Viso** | **47** | **10** | **21 %** |

---

## 13. Rizikų registras

| ID | Rizika | Tikimybė | Poveikis | Mitigacija |
|---|---|---|---|---|
| **R1** | libretro FFI nestabilumas, segfault'ai callback'uose | Vidutinė | 🔴 Kritinis | P1.4/P1.5 daryti atsargiai, `GET_LOG_INTERFACE` pirmiausia, testuoti su 3+ core'ais anksti |
| **R2** | wgpu + Tauri langas neveikia Linux/Wayland | Vidutinė | 🔴 Kritinis | Dokumentuotas fallback į `Channel` + WebGL canvas (P2.3). Testuoti abu backend'us Fazėje 2, ne pabaigoje |
| **R3** | Garso traškesiai, kurių nepavyksta pašalinti | Vidutinė | 🟡 Didelis | Dynamic rate control yra įrodyta technika (RetroArch). Jei nepavyksta — didink buferį iki 100 ms |
| **R4** | Core'ų globalus būvis neleidžia perjungti be restarto | Aukšta | 🟡 Vidutinis | Priimtina MVP: reikalauti restarto. Dokumentuoti. Child procesas — post-MVP |
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
**Data:** 2026-08-19 · **Statusas:** priimta
**Kontekstas:** Kadrus reikia rodyti 60 k./s.
**Sprendimas:** Atskiras Tauri `Window` be webview + wgpu `Surface` per `raw-window-handle`.
**Priežastis:** Kadrų siuntimas per IPC į canvas neskaluojasi: 640×480×4 B × 60 = 73 MB/s.
Native langas duoda zero-copy GPU kelią ir vsync.
**Alternatyva (fallback):** `Channel<&[u8]>` → WebGL2 canvas. Priimtina 8/16-bit sistemoms
(256×224 ≈ 7 MB/s), bet ne N64/PSP.
**Pasekmės:** Du langai vietoj vieno. Reikia atskirai spręsti fullscreen, fokusą, hotkey'us.

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

---

## 15. Po MVP — idėjų sąrašas

> **Nedaryk nieko iš šio sąrašo, kol MVP nebaigtas.**
> Naujos idėjos rašomos čia, ne į fazių planą.

**v0.2 — Gilesnis emuliavimas**
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
