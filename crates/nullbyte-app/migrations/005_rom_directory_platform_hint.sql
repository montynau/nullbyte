-- P7.5 / ADR-020: leidžia vartotojui eksplicitiškai nurodyti katalogo platformą pridedant,
-- pašalinant dviprasmiškumą tarp platformų, kurios dalinasi tuos pačius archyvo vidinius
-- plėtinius (PSX/Saturn/SegaCD visos priima .cue/.iso/.chd — realus testas: 3 PSX žaidimai
-- klaidingai atsidūrė po Sega CD). NULL = automatinis nustatymas pagal plėtinį (senas
-- elgesys — veikia gerai vienareikšmiams plėtiniams kaip .sfc/.nes).
ALTER TABLE rom_directories ADD COLUMN platform_id INTEGER REFERENCES platforms(id);
