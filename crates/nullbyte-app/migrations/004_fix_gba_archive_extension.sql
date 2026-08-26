-- P7.5: GBA ROM'ai realiai dažnai platinami kaip .zip (vienas vidinis .gba failas) — tas pats
-- atvejis kaip PSX/Saturn/SegaCD (002_fix_archive_extensions.sql), tiesiog nepastebėtas tada.
-- Atrasta realaus skenavimo testu (crates/nullbyte-core/roms/gba/*.zip — visi 0 rezultatų).
UPDATE platforms SET extensions = 'gba,zip,7z' WHERE slug = 'gba';
