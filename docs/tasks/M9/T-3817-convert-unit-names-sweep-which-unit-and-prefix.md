---
id: T-3817
title: "CONVERT unit names: sweep which unit and prefix strings Excel accepts (prefixes per unit, binary prefixes, time units, powers, case and aliases)"
milestone: M9
lane: functions-engineering
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
CONVERT accepts exactly the unit strings Excel accepts and gives `#N/A` for the rest: which units take a decimal or a
binary prefix, which aliases and powers exist, and how case and surrounding characters are read, stated as one rule
per unit class in engineering.md rather than as a list of points.

## Context
Gap triage 2026-11-02 (report of `make eval` on 2026-11-02): CONVERT has no visibly-off cell without an owner (its 6
failing cells are last digits, NumericUlp, the work of T-2046 and T-2047), so this is an evidence gap found by reading the code against the evidence.
The CONVERT evidence is about factors, not about which strings parse: ironcalc-calc ENGINEERING/CONVERT.xlsx (509
Excel-cached cells, 2-3 per unit, a few prefixed), probes/convert-targets (non-base and prefixed targets), and the
temperature prefix probes (probes/convert-temperature-prefix-a/-b: only `K` and `kel` take a prefix). Across every
suite, the prefixed unit strings with Excel cells are about a dozen (`km`, `kg`, `mm`, `cm`, `Ym`, `Zm`, `km2`, `km3`,
`Mm3`, `kJ`, `mJ`, `kdyn`, `kbyte`, `kibyte`, `ml`).
The engine's acceptance rule is a hand list (`fn_convert`, crates/rustsheet-engine/src/functions/engineering/convert.rs):
`mks` names the units that admit a prefix, and any of them takes any prefix of one table that holds both the decimal
and the binary prefixes (`ki` .. `Yi`; the table literal also lists `Yi` and `Ei` twice with different powers, so the
later entry wins). None of this is evidenced beyond the strings above. Unprobed, with one cell each (CONVERT(1,
<unit>, <its base unit>), and the same unit as the target):
- binary prefixes on units other than bit and byte (`kim`, `Mig`, `kiJ`) and decimal prefixes on bit and byte (`Mbit`,
  `Gbyte`);
- prefixes on units the list includes that may not take one (`katm`, `kmmHg`, `kmph`, `kuk_pt`, `kcal`, `keV`,
  `kly`, `kpc`, `kga`, `kT`) and on units it leaves out (`ms`, `msec`, `kyr`, `kday`, `khr`, `kmi`, `kin`, `kft`,
  `kHP`, `kBTU`, `klbm`, `kNmi`);
- the two-letter `da` and the `e` deka alias against units starting with `a` (`daar`, `eatm`, `dam`), and
  ambiguous parses (`pc` parsec vs pico-c, `Pa` pascal vs a prefixed unit, `nmi`, `mm`, `mmHg`, `ang` vs `Ang`);
- powers: `km2` against `km^2`, `cm3`, `ft2`, `ft^2`, `kft2`, `in3`, `mi2`, `ly3`, `kang2`, a prefixed `ar`;
- case and characters: `KM`, `Km`, `kM`, `BYTE`, `Byte`, a leading or trailing space, an empty string.
Choose valid values away from overflow so that a parse difference is not confused with a factor's last digit.
The factors themselves are evidenced ("CONVERT constants and evaluation order"): this task changes which strings
parse, never a factor.

## Acceptance criteria
- [ ] A probe case (excel-graph, canary-verified) covers the unit strings above, both as from_unit and as to_unit.
- [ ] The engine matches every probed cell; the prefix rule is stated in engineering.md per unit class with the probe
      cells cited (VERIFIED), and the duplicate entries of the prefix table are gone.
- [ ] No ironcalc-calc CONVERT.xlsx or convert-* probe cell changes verdict.
- [ ] `make check` green; eval known failures only shrink (baseline updated, no new lines); reviewers approve.

## Out of scope
- CONVERT's last digits (T-2046, T-2047), temperature forms (evidenced) and argument kinds (T-3124, T-3214).
