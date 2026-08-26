-- P7.6 (Paths panelės dokumentavimo metu aptikta): 002_fix_archive_extensions.sql pašalino
-- `zip`/`7z` iš Neo Geo plėtinių sąrašo (liko tik `neo`), tad suarchyvuoti Neo Geo ROM'ai
-- (vienas vidinis .neo failas, tas pats vieno-failo modelis kaip GBA — žr.
-- 004_fix_gba_archive_extension.sql) VISAI neatpažįstami, net su platform_id hint'u.
-- SKIRTINGAI nuo Arcade (dabar tuščias extensions sąrašas — SĄMONINGAI NEKEIČIAMA šia
-- migracija): MAME-tipo Arcade ROM'ai neturi VIENO atpažįstamo vidinio failo pagal plėtinį
-- (keli žalio chip dump'o failai), tad vien plėtinio grąžinimas nepataisytų realaus
-- skenavimo — reikėtų naujos `extract_first_match` logikos. Neo Geo `.neo` formatas TOKIOS
-- problemos neturi (vienas failas, kaip GBA), tad čia paprastas fix'as.
UPDATE platforms SET extensions = 'neo,zip,7z' WHERE slug = 'neogeo';
