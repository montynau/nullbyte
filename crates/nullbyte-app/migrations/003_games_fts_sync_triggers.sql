-- P5.4: `games_fts` (001_initial.sql) yra EXTERNAL CONTENT FTS5 lentelė
-- (`content='games', content_rowid='id'`) — SQLite NEsinchronizuoja tokių lentelių
-- automatiškai; be šių trigerių `games_fts` VISADA liktų tuščia, nepriklausomai nuo to, kiek
-- žaidimų yra `games`, o paieška tyliai negrąžintų NIEKO (jokios klaidos, tik neteisingas
-- rezultatas — pavojingiausia klaidų rūšis). Standartinis SQLite FTS5 external-content
-- sinchronizavimo šablonas (žr. https://www.sqlite.org/fts5.html #External Content Tables).
CREATE TRIGGER games_fts_after_insert AFTER INSERT ON games BEGIN
    INSERT INTO games_fts(rowid, title, description) VALUES (new.id, new.title, new.description);
END;

CREATE TRIGGER games_fts_after_delete AFTER DELETE ON games BEGIN
    INSERT INTO games_fts(games_fts, rowid, title, description)
    VALUES ('delete', old.id, old.title, old.description);
END;

CREATE TRIGGER games_fts_after_update AFTER UPDATE ON games BEGIN
    INSERT INTO games_fts(games_fts, rowid, title, description)
    VALUES ('delete', old.id, old.title, old.description);
    INSERT INTO games_fts(rowid, title, description) VALUES (new.id, new.title, new.description);
END;

-- Backfill — jei šis serveris jau turėjo `games` eilučių PRIEŠ šią migraciją (trigeriai
-- veikia tik BŪSIMIEMS pakeitimams, ne retroaktyviai).
INSERT INTO games_fts(rowid, title, description) SELECT id, title, description FROM games;
