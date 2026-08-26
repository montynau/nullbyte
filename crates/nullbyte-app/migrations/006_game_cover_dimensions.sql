-- ADR-021: tikri viršelio paveikslėlio matmenys (P7.2 GameGrid „packed row" layout'ui) —
-- realūs ScreenScraper box-2D viršeliai turi LABAI skirtingas proporcijas tarp platformų
-- (patikrinta realiais atsisiųstais failais: PSX 680x680 kvadratas, SNES 680x497 platus,
-- Genesis 484x680 aukštas, GBA 705x700 beveik kvadratas) — jokia bendra prielaida netinka,
-- reikalingi TIKRI matmenys, nuskaitomi PNG/JPEG header'io atsisiuntimo metu
-- (scraper/image_dimensions.rs). NULL = dar neatsisiųstas/nenuskaitytas viršelis —
-- GameCard tada naudoja numatytąją 3:4 proporciją.
ALTER TABLE games ADD COLUMN cover_width INTEGER;
ALTER TABLE games ADD COLUMN cover_height INTEGER;
