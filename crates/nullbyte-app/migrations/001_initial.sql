-- P5.1: pradinė schema (CLAUDE.md §3.1, MVP.md P5.1).
--
-- PRAGMA journal_mode/foreign_keys NĖRA čia — jos taikomos KIEKVIENAM prisijungimui atskirai
-- (CLAUDE.md §10 „SQLite": „Įjunk PRAGMA journal_mode = WAL; ir PRAGMA foreign_keys = ON;
-- prie kiekvieno prisijungimo"), ne vieną kartą migracijos metu — `journal_mode` iš tikrųjų
-- yra pastovus DB failo atributas (užtenka kartą), bet `foreign_keys` yra PER-CONNECTION
-- nustatymas SQLite (numatytai OFF kiekvienam naujam ryšiui), tad jį reikia nustatyti kaskart
-- atsidarant `Connection`, ne migracijos SQL faile — žr. `db::migrations::run`.

CREATE TABLE platforms (
    id                INTEGER PRIMARY KEY,
    slug              TEXT NOT NULL UNIQUE,
    name              TEXT NOT NULL,
    screenscraper_id  INTEGER,
    extensions        TEXT NOT NULL
);

CREATE TABLE games (
    id                INTEGER PRIMARY KEY,
    platform_id       INTEGER NOT NULL REFERENCES platforms(id),
    title             TEXT NOT NULL,
    sort_title        TEXT NOT NULL,
    rom_path          TEXT NOT NULL UNIQUE,
    rom_size          INTEGER NOT NULL,
    archive_inner     TEXT,
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
    cover_path        TEXT,
    screenshot_path   TEXT,
    wheel_path        TEXT,
    video_path        TEXT,
    scrape_status     TEXT NOT NULL DEFAULT 'pending',
    scraped_at        INTEGER,
    last_played       INTEGER,
    play_count        INTEGER NOT NULL DEFAULT 0,
    play_time_seconds INTEGER NOT NULL DEFAULT 0,
    favorite          INTEGER NOT NULL DEFAULT 0,
    added_at          INTEGER NOT NULL,
    file_mtime        INTEGER NOT NULL
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
    slot         INTEGER NOT NULL,
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
    hash_key   TEXT PRIMARY KEY,
    response   TEXT,
    found      INTEGER NOT NULL,
    fetched_at INTEGER NOT NULL
);

-- Seed platformos — README.md „Works during the MVP (software rendering)" sąrašas
-- (jau nuspręstas produkto apimties dokumentas, ne šios migracijos sprendimas).
-- `screenscraper_id` reikšmės PATIKRINTOS prieš community-sourced ScreenScraper systemeid
-- lentelę (gist.github.com/dollerbill/86162c5cb249d79ef01a9ad2c691d29d, 2026-08-25) — TAI NĖRA
-- oficialus ScreenScraper API atsakas (tam reikia devid/devpassword, P6.1 dar nepradėtas).
-- Platformoms, kurių ID NEPAVYKO patikrinti šiame šaltinyje, sąmoningai paliktas NULL, o ne
-- spėjama reikšmė (žr. CLAUDE.md/atminties taisyklę „Verify external API refs") — P6.1 API
-- klientas juos patvirtins/pataisys prieš tikrą API atsakymą.
INSERT INTO platforms (slug, name, screenscraper_id, extensions) VALUES
    ('nes',           'Nintendo Entertainment System', 3,    'nes,fds'),
    ('snes',          'Super Nintendo Entertainment System', 4, 'sfc,smc,fig'),
    ('gb',            'Game Boy',                       9,    'gb'),
    ('gbc',           'Game Boy Color',                 10,   'gbc'),
    ('gba',           'Game Boy Advance',                12,   'gba'),
    ('nds',           'Nintendo DS',                     15,   'nds'),
    ('mastersystem',  'Sega Master System',              2,    'sms'),
    ('gamegear',      'Sega Game Gear',                  21,   'gg'),
    ('genesis',       'Sega Genesis / Mega Drive',        1,    'md,smd,bin,gen'),
    ('segacd',        'Sega CD',                          20,   'cue,iso,chd'),
    ('sega32x',       'Sega 32X',                         19,   '32x'),
    ('saturn',        'Sega Saturn',                      22,   'cue,iso,chd'),
    ('psx',           'Sony PlayStation',                 57,   'cue,bin,chd,pbp,m3u'),
    ('atari2600',     'Atari 2600',                       26,   'a26,bin'),
    ('atari7800',     'Atari 7800',                       NULL, 'a78'),
    ('atari800',      'Atari 800',                        NULL, 'atr,xex'),
    ('atari5200',     'Atari 5200',                       NULL, 'a52'),
    ('pcengine',      'PC Engine / TurboGrafx-16',         31,   'pce'),
    ('neogeo',        'Neo Geo',                          NULL, 'neo,zip'),
    ('arcade',        'Arcade',                           NULL, 'zip'),
    ('vectrex',       'Vectrex',                          102,  'vec'),
    ('intellivision', 'Intellivision',                    NULL, 'int,bin'),
    ('odyssey2',      'Magnavox Odyssey²',                NULL, 'bin');
