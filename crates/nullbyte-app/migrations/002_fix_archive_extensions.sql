-- P5.3: pataisyti P5.1 seed duomenų spragą, kurią atskleidė realus skenavimo testas
-- (`scan_real_fixtures_is_fast`) — bare `zip` buvo priskirtas KELIOMS platformoms vienu
-- metu, tad P5.3 skeneris (grynai extension-based, be katalogo konteksto — `rom_directories`
-- schema NETURI `platform_id` stulpelio) negalėjo vienareikšmiškai nustatyti, kuriai
-- platformai priklauso konkretus `.zip`.
--
-- PSX/Saturn/SegaCD REALIAI dažnai platinami kaip vieno disko atvaizdas `.zip` archyve
-- (žr. tikrus test fixture'us `nullbyte-core/roms/psx/*.zip`) — jiems `zip`/`7z` PRIKLAUSO
-- `extensions` sąraše, ir šis modelis tinka (vienas VIDINIS failas archyve, `archive.rs`
-- `extract_first_match` prielaida).
--
-- Neo Geo/Arcade — PRIEŠINGAI, `zip` PAŠALINAMAS. Tai buvo per anksti pridėta P5.1 metu:
-- realūs MAME/Neo Geo romset'ai turi DAUGYBĘ atskirų ROM chip failų VIENAME `.zip`, ne vieną
-- „pagrindinį" failą (tas pats radinys, kurį atskleidė šios sesijos MAME rankinis testas,
-- žr. P4.0.5/P4.2 istoriją) — `archive.rs` „vieno vidinio failo" modelis jiems netinka, tad
-- teigti, kad mokame juos skenuoti/hash'uoti, būtų klaidinga, kol tam neatsiras atskiras
-- (post-MVP) apdorojimas.
UPDATE platforms SET extensions = 'cue,bin,chd,pbp,m3u,zip,7z' WHERE slug = 'psx';
UPDATE platforms SET extensions = 'cue,iso,chd,zip,7z' WHERE slug = 'saturn';
UPDATE platforms SET extensions = 'cue,iso,chd,zip,7z' WHERE slug = 'segacd';
UPDATE platforms SET extensions = 'neo' WHERE slug = 'neogeo';
UPDATE platforms SET extensions = '' WHERE slug = 'arcade';
