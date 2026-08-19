<div align="center">

# Nullbyte

**Modernus retro žaidimų emuliavimo frontend'as macOS ir Linux**

Nullbyte įkelia libretro core'us ir suteikia jiems UI, kokio retro emuliacija nusipelnė.

[![Rust](https://img.shields.io/badge/Rust-1.82+-orange?logo=rust)](https://www.rust-lang.org)
[![Tauri](https://img.shields.io/badge/Tauri-v2-24C8DB?logo=tauri)](https://v2.tauri.app)
[![Svelte](https://img.shields.io/badge/Svelte-5-FF3E00?logo=svelte)](https://svelte.dev)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

</div>

---

## Kas tai

Nullbyte yra **frontend'as**, ne emuliatorius. Jis nesukuria savo emuliavimo kodo — vietoj to
įkelia [libretro](https://docs.libretro.com) core'us (tas pačias bibliotekas, kurias naudoja
RetroArch) ir apvelka jas greitu, tvarkingu, klaviatūra valdomu UI.

Idėja paprasta: RetroArch turi geriausią emuliavimo ekosistemą, bet jo sąsaja sukurta
televizoriui ir žaidimų konsolėms. OpenEmu turi gražią sąsają, bet veikia tik macOS ir naudoja
savo uždarą core sistemą, kurios beveik niekas nebeprižiūri. Nullbyte bando paimti geriausias
abiejų puses.

### Iš kur pavadinimas

**null** + **byte** — nulinis baitas, žemiausias duomenų lygis. Seni žaidimai ir buvo tik
baitai atmintyje; Nullbyte grąžina juos į ekraną. Logotipe — `0x00`.

### Kuo skiriasi

| | RetroArch | OpenEmu | **Nullbyte** |
|---|---|---|---|
| Core sistema | libretro | sava `.oecoreplugin` | **libretro** |
| Platformos | visos | tik macOS | **macOS + Linux** |
| UI technologija | savas menu driver | AppKit / SwiftUI | **Svelte 5 + Tailwind** |
| Metaduomenys | thumbnails repo | OpenVGDB (offline) | **ScreenScraper API** |
| Gameplay video | ne | ne | **taip** |
| ROM atpažinimas | pavadinimu | pavadinimu / hash | **CRC32 + MD5 + SHA1** |

---

## Funkcijos

### Biblioteka

- **Automatinis ROM skenavimas** — nurodai katalogus, Nullbyte suranda ir atpažįsta žaidimus
- **Hash pagrįstas atpažinimas** — CRC32, MD5 ir SHA1, ne spėliojimas pagal failo pavadinimą
- **Archyvų palaikymas** — `.zip` ir `.7z` skaitomi tiesiogiai, be išpakavimo
- **Gameplay video preview** — užvedus pelę ant žaidimo groja trumpas gameplay įrašas
- **Viršeliai, screenshot'ai, logotipai, aprašymai** — automatiškai iš ScreenScraper
- **Greita paieška ir filtravimas** — pagal platformą, žanrą, metus, paskutinį žaidimą
- **Virtualizuotas grid'as** — sklandu net su tūkstančiais žaidimų

### Emuliavimas

- **Bet koks libretro core** — jei veikia RetroArch'e, veiks ir čia
- **Tikslus timing** — kadrų dažnis imamas iš core (SNES 60.098 Hz, ne apvalinta 60)
- **Audio-driven sinchronizacija** su dynamic rate control — be traškesių ir drifto
- **GPU atvaizdavimas** per wgpu (Metal macOS, Vulkan Linux)
- **Save states** su preview paveiksliuku
- **Automatinis SRAM išsaugojimas** — progresas neprapuola
- **Gamepad palaikymas** — bet koks USB / Bluetooth valdiklis per `gilrs`

### Sąsaja

- Tamsi tema kaip numatytoji
- Klaviatūros navigacija ir command palette
- Nustatymai be XML ir be konfigūracijos failų redagavimo ranka

---

## Palaikomos platformos

Nullbyte palaiko viską, kam yra libretro core'as. Testuojami ir dokumentuojami:

| Konsolė | Rekomenduojamas core |
|---|---|
| Nintendo (NES / Famicom) | Nestopia UE, FCEUmm |
| Super Nintendo (SNES) | Snes9x, bsnes-mercury |
| Nintendo 64 | Mupen64Plus-Next, ParaLLEl N64 |
| GameCube / Wii | Dolphin |
| Game Boy / Color | Gambatte, SameBoy |
| Game Boy Advance | mGBA |
| Nintendo DS | melonDS, DeSmuME |
| Sega Master System / Game Gear | Genesis Plus GX |
| Sega Genesis / Mega Drive / CD | Genesis Plus GX, PicoDrive |
| Sega 32X | PicoDrive |
| Sega Saturn | Beetle Saturn, YabaSanshiro |
| Sony PlayStation | Beetle PSX, SwanStation |
| Sony PSP | PPSSPP |
| Atari 2600 | Stella |
| Atari 7800 | ProSystem |
| Atari 800 / 5200 | Atari800 |
| PC Engine / TurboGrafx-16 | Beetle PCE |
| Neo Geo / Arcade | FinalBurn Neo, MAME |
| Vectrex | vecx |
| Intellivision | FreeIntv |
| Magnavox Odyssey² | O2EM |

> Nullbyte nepateikia core'ų. Juos atsisiunti pats iš
> [buildbot.libretro.com](https://buildbot.libretro.com/nightly/) arba per savo sistemos
> paketų valdyklę.

---

## Sistemos reikalavimai

| | Minimalūs | Rekomenduojami |
|---|---|---|
| **macOS** | 12 Monterey | 14 Sonoma+ |
| **Linux** | glibc 2.31+, Vulkan 1.1 | Vulkan 1.3, Wayland arba X11 |
| **CPU** | dviejų branduolių x86-64 arba Apple Silicon | 4+ branduoliai |
| **RAM** | 4 GB | 8 GB (GameCube/PSP core'ams) |
| **GPU** | bet kokia su Metal / Vulkan palaikymu | — |

---

## Diegimas

### Paruošti build'ai

Atsisiųsk iš [Releases](https://github.com/USERNAME/nullbyte/releases):

- **macOS:** `Nullbyte_x.y.z_universal.dmg` (Intel + Apple Silicon)
- **Linux:** `nullbyte_x.y.z_amd64.AppImage` arba `.deb`

macOS pirmą kartą paleidžiant: `System Settings → Privacy & Security → Open Anyway`
(programa dar nėra notarizuota).

Linux AppImage:

```bash
chmod +x nullbyte_x.y.z_amd64.AppImage
./nullbyte_x.y.z_amd64.AppImage
```

---

## Kūrimas (development)

### Priklausomybės

**Bendra:**

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Node.js 20+ ir pnpm
npm install -g pnpm
```

**macOS:**

```bash
xcode-select --install
```

**Linux (Debian / Ubuntu):**

```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev \
  libasound2-dev libudev-dev
```

**Linux (Fedora):**

```bash
sudo dnf install webkit2gtk4.1-devel openssl-devel curl wget file \
  libappindicator-gtk3-devel librsvg2-devel alsa-lib-devel systemd-devel
sudo dnf group install "C Development Tools and Libraries"
```

**Linux (Arch):**

```bash
sudo pacman -S webkit2gtk-4.1 base-devel curl wget file openssl \
  libappindicator-gtk3 librsvg alsa-lib
```

### Paleidimas

```bash
git clone https://github.com/USERNAME/nullbyte.git
cd nullbyte

pnpm install
cp .env.example .env       # įrašyk savo ScreenScraper credentials

pnpm tauri dev
```

### Naudingos komandos

```bash
pnpm dev            # tik frontend, be Tauri (greičiau UI darbams)
pnpm tauri dev      # pilnas dev režimas
pnpm tauri build    # produkcinis build'as
pnpm check          # svelte-check + tsc
pnpm lint           # eslint + prettier --check
pnpm format         # prettier --write

cargo test    --manifest-path src-tauri/Cargo.toml
cargo clippy  --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo fmt     --manifest-path src-tauri/Cargo.toml
```

---

## Konfigūracija

### Core'ų katalogas

Nullbyte ieško libretro core'ų šiuose keliuose:

| Platforma | Kelias |
|---|---|
| macOS | `~/Library/Application Support/Nullbyte/cores/` |
| Linux | `~/.local/share/nullbyte/cores/` |

Papildomus katalogus gali nurodyti nustatymuose. Atsisiuntę core'ą (`*_libretro.dylib` /
`*_libretro.so`), tiesiog įdėkite jį ten — Nullbyte aptiks automatiškai.

### BIOS failai

Kai kurioms sistemoms (PlayStation, Saturn, PC Engine CD) reikia originalių BIOS failų:

| Platforma | Kelias |
|---|---|
| macOS | `~/Library/Application Support/Nullbyte/system/` |
| Linux | `~/.local/share/nullbyte/system/` |

### ScreenScraper

Metaduomenims ir video reikia [ScreenScraper](https://www.screenscraper.fr) paskyros.
Registracija nemokama; be paskyros kvota praktiškai nulinė.

`.env` failas:

```env
SCREENSCRAPER_DEV_ID=tavo_dev_id
SCREENSCRAPER_DEV_PASSWORD=tavo_dev_slaptazodis
```

Vartotojo login/slaptažodis įvedamas aplikacijos nustatymuose ir saugomas lokaliai.

> Dev credentials gaunami parašius ScreenScraper administratoriams jų forume.
> Jie **niekada** nepatenka į repozitoriją.

### Duomenys

| Platforma | Duomenų katalogas |
|---|---|
| macOS | `~/Library/Application Support/Nullbyte/` |
| Linux | `~/.local/share/nullbyte/` |

```
nullbyte/
├── nullbyte.db        # SQLite: biblioteka, nustatymai, metaduomenų cache
├── cores/             # libretro core'ai
├── system/            # BIOS failai
├── saves/             # SRAM (.srm)
├── states/            # save states
└── media/             # viršeliai, screenshot'ai, video
```

---

## Architektūra

```
┌──────────────────────────────────────────────────┐
│  Svelte 5 + shadcn-svelte + Tailwind (WebView)   │
└─────────────────────┬────────────────────────────┘
                      │ Tauri v2 IPC
┌─────────────────────▼────────────────────────────┐
│  Rust                                            │
│  ├── libloading  →  libretro core (.dylib/.so)   │
│  ├── wgpu        →  vaizdo atvaizdavimas         │
│  ├── cpal        →  garso išvestis               │
│  ├── gilrs       →  gamepad įvestis              │
│  ├── rusqlite    →  SQLite biblioteka            │
│  └── reqwest     →  ScreenScraper API            │
└──────────────────────────────────────────────────┘
```

Emuliavimas vyksta dedikuotoje gijoje; vaizdas ir garsas keliauja per lock-free buferius,
kad UI niekada nestabdytų emuliacijos. Detaliau — [CLAUDE.md](CLAUDE.md).

---

## Roadmap

### MVP (v0.1)

- [x] Projekto sprendimai ir dokumentacija
- [ ] libretro core įkėlimas ir paleidimas
- [ ] Vaizdas (wgpu) ir garsas (cpal)
- [ ] Gamepad ir klaviatūros įvestis
- [ ] ROM skenavimas ir SQLite biblioteka
- [ ] ScreenScraper metaduomenys + video preview
- [ ] Save states ir SRAM
- [ ] Nustatymų ekranas

Detalus planas — [MVP.md](MVP.md).

### v0.2

- [ ] Core options UI (per-core nustatymai)
- [ ] Shader'iai (CRT, scanlines, xBRZ)
- [ ] Netplay
- [ ] Rewind
- [ ] Achievements (RetroAchievements)

### v0.3+

- [ ] Core downloader tiesiai iš aplikacijos
- [ ] Playlist'ai ir kolekcijos
- [ ] Statistikos (žaidimo laikas, dažniausiai žaisti)
- [ ] Šviesi tema
- [ ] Lokalizacija
- [ ] Windows palaikymas

---

## Teisinė informacija

**Nullbyte yra legali programinė įranga.** Emuliatoriai ir emuliavimo frontend'ai yra teisėti
daugumoje jurisdikcijų.

Tačiau:

- **Nullbyte nepateikia, nedistribuoja ir nenurodo, kur gauti ROM failų ar BIOS.**
- ROM failų atsisiuntimas iš interneto be originalaus žaidimo nuosavybės daugumoje šalių
  yra autorių teisių pažeidimas.
- BIOS failai yra gamintojų autorių teisių objektas ir turi būti išgauti iš jūsų pačių įrangos.
- Naudotojas pats atsako už tai, kad jo turimi ROM ir BIOS failai būtų įgyti teisėtai.

Prašymai pridėti ROM šaltinius bus uždaromi be diskusijų.

---

## Prisidėjimas

Pull request'ai laukiami. Prieš pradedant:

1. Perskaityk [CLAUDE.md](CLAUDE.md) — ten yra architektūra ir konvencijos
2. Patikrink [MVP.md](MVP.md) — gal tai jau suplanuota
3. Didesniems pakeitimams pirma atidaryk issue

Reikalavimai PR'ui:

```bash
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test   --manifest-path src-tauri/Cargo.toml
pnpm check
pnpm lint
```

Commit'ai — [Conventional Commits](https://www.conventionalcommits.org).

---

## Padėkos

- [libretro / RetroArch](https://www.libretro.com) — core ekosistema, be kurios šio projekto nebūtų
- [OpenEmu](https://openemu.org) — įkvėpimas, kaip emuliavimo UI *gali* atrodyti
- [ScreenScraper](https://www.screenscraper.fr) — metaduomenų ir media duomenų bazė
- [Tauri](https://tauri.app), [Svelte](https://svelte.dev), [shadcn-svelte](https://shadcn-svelte.com)
- Visiems core kūrėjams — Snes9x, mGBA, Genesis Plus GX, Mupen64Plus, Beetle, Dolphin, PPSSPP ir kitiems

---

## Licencija

MIT — žr. [LICENSE](LICENSE).

Libretro core'ai turi savo atskiras licencijas (dažniausiai GPL). Nullbyte juos įkelia
dinamiškai vykdymo metu ir nedistribuoja.
