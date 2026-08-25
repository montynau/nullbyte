//! ScreenScraper `jeuInfos.php` JSON atsako struct'ai (MVP.md P6.1).
//!
//! Struktūra PATIKRINTA REALIU API atsakymu (Super Metroid, SNES, `crc=AD2CBF9C`,
//! 2026-08-25) — NE spėta iš dokumentacijos (žr. atminties taisyklę „Verify external API
//! refs" — tas pats principas galioja API atsakymo formai, ne tik pavieniams laukams).
//! Visi ID laukai — `String`, ne skaičiai: ScreenScraper JUOS VISUS grąžina kaip JSON
//! string'us (pvz. `"id": "286"`), net kai reikšmė skaitinė.
//!
//! **Kartais masyvas, kartais objektas** (CLAUDE.md §9.1 įspėjimas) — realiame atsakyme
//! `noms`/`dates`/`synopsis`/`classifications`/`genres`/`medias` visi buvo masyvai, bet
//! žinoma PHP→JSON serializacijos ypatybė (masyvas su VIENU elementu kartais suplokštėja į
//! pavienį objektą) reiškia, kad kito žaidimo/lauko atveju tas pats laukas gali ateiti kaip
//! objektas. [`OneOrMany`] apsaugo nuo abiejų atvejų vienu metu.
//!
//! **NENUMANOMA reikšmė svarbiam atvejui:** kai ROM'as nerandamas, API grąžina **HTTP 404
//! su PAPRASTU TEKSTU** (`"Erreur : Rom/Iso/Dossier non trouvée !"`), NE JSON — net kai
//! `output=json` prašomas eksplicitiškai. Tai tvarkoma HTTP lygmenyje
//! (`screenscraper.rs::lookup_game`), ne čia, PRIEŠ bandant JSON parsinimą.

#![allow(dead_code)] // pilnai išnaudos P6.2/P6.4 (rate limiting, scraping orkestracija)

use serde::Deserialize;

/// Priima arba pavienį `T`, arba `Vec<T>` (žr. modulio doc). `into_vec()` visada grąžina
/// vienodą formą tolimesniam kodui, nepriklausomai nuo to, ką API šįkart grąžino.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

impl<T> OneOrMany<T> {
    pub fn into_vec(self) -> Vec<T> {
        match self {
            OneOrMany::One(item) => vec![item],
            OneOrMany::Many(items) => items,
        }
    }
}

impl<T> Default for OneOrMany<T> {
    fn default() -> Self {
        OneOrMany::Many(Vec::new())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct JeuInfosResponse {
    #[allow(dead_code)] // `header.success`/`error` naudingi debug'inant, ne būtini P6.1 logikai.
    pub header: Header,
    /// `None`, jei API šio rakto visai negrąžino — retas, bet leistinas atvejis.
    pub response: Option<JeuInfosBody>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Header {
    /// `"true"`/`"false"` STRING, ne bool — API ypatybė.
    #[serde(default)]
    pub success: String,
    #[serde(default)]
    pub error: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JeuInfosBody {
    pub jeu: Jeu,
    /// PATIKRINTA REALIU `jeuInfos.php` atsakymu (2026-08-25, ne tik `ssuserInfos.php`) —
    /// abu laukai yra KIEKVIENAME sėkmingame atsakyme, ne tik dedikuotame kvotos endpoint'e.
    /// `Option`, ne privalomas — apsauga, jei API kada nors juos praleistų (P6.2 „blogas
    /// JSON nesulaužo" principas taikomas ir čia).
    #[serde(default)]
    pub serveurs: Option<Serveurs>,
    #[serde(default)]
    pub ssuser: Option<SsUser>,
}

/// Serverio būvis — `closefornomember`/`closeforleecher` yra „API uždaryta" signalas
/// (MVP.md P6.2 „Ką daryti": „429/430/`API closed`").
#[derive(Debug, Clone, Deserialize)]
pub struct Serveurs {
    #[serde(default)]
    pub closefornomember: String,
    #[serde(default)]
    pub closeforleecher: String,
}

/// Vartotojo kvota — `maxthreads` semaforo dydžiui, `requeststoday`/`maxrequestsperday`
/// UI kvotos indikatoriui (MVP.md P6.2 „Ką daryti").
#[derive(Debug, Clone, Deserialize)]
pub struct SsUser {
    #[serde(default)]
    pub maxthreads: String,
    #[serde(default)]
    pub requeststoday: String,
    #[serde(default)]
    pub maxrequestsperday: String,
}

/// `{region, text}` — naudojama `noms`/`dates`.
#[derive(Debug, Clone, Deserialize)]
pub struct RegionText {
    #[serde(default)]
    pub region: String,
    pub text: String,
}

/// `{langue, text}` — naudojama `synopsis`/`Genre.noms`.
#[derive(Debug, Clone, Deserialize)]
pub struct LangText {
    #[serde(default)]
    pub langue: String,
    pub text: String,
}

/// `{id, text}` — naudojama `developpeur`/`editeur`.
#[derive(Debug, Clone, Deserialize)]
pub struct IdText {
    #[allow(dead_code)]
    pub id: String,
    pub text: String,
}

/// `{text}` — naudojama `joueurs`/`note`.
#[derive(Debug, Clone, Deserialize)]
pub struct TextOnly {
    pub text: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Genre {
    #[serde(default)]
    pub noms: OneOrMany<LangText>,
}

/// Vienas media įrašas (`medias` masyvo elementas) — `type` diskriminuoja (`"box-2D"`,
/// `"video"`, `"ss"`, ir t.t., žr. CLAUDE.md §9.2 lentelę), likę laukai bendri visiems tipams.
#[derive(Debug, Clone, Deserialize)]
pub struct Media {
    #[serde(rename = "type")]
    pub media_type: String,
    #[serde(default)]
    pub region: Option<String>,
    pub url: String,
    #[serde(default)]
    pub format: Option<String>,
}

/// Konkretus SUTAPĘS ROM įrašas (`jeu.rom`, PAVIENIS objektas — ne visų žinomų romset'ų
/// sąrašas, tas yra `jeu.roms`, kurio P6.1 nenaudoja). Tik laukai, reikalingi patvirtinti,
/// kad rastas TEISINGAS įrašas — likusieji (kalbos, regionai ir pan.) tyliai ignoruojami
/// (serde numatytasis elgesys nežinomiems laukams — tai IR YRA „blogas JSON nesulaužo").
#[derive(Debug, Clone, Deserialize)]
pub struct RomMatch {
    #[serde(default)]
    pub romcrc: Option<String>,
    #[serde(default)]
    pub rommd5: Option<String>,
    #[serde(default)]
    pub romsha1: Option<String>,
    #[serde(default)]
    pub romfilename: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Jeu {
    #[allow(dead_code)]
    pub id: String,
    #[serde(default)]
    pub noms: OneOrMany<RegionText>,
    #[serde(default)]
    pub synopsis: OneOrMany<LangText>,
    #[serde(default)]
    pub developpeur: Option<IdText>,
    #[serde(default)]
    pub editeur: Option<IdText>,
    #[serde(default)]
    pub joueurs: Option<TextOnly>,
    #[serde(default)]
    pub note: Option<TextOnly>,
    #[serde(default)]
    pub dates: OneOrMany<RegionText>,
    #[serde(default)]
    pub genres: OneOrMany<Genre>,
    #[serde(default)]
    pub medias: OneOrMany<Media>,
    #[serde(default)]
    pub rom: Option<RomMatch>,
}
