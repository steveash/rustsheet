---
id: T-3479
title: "F.DIST density and cdf past the 15th digit at a tiny x (T-3354 residue, Phase 1b)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
The F.DIST density and cdf cells that T-3354's two cases leave beyond half a unit in Excel's 15th significant digit
match Excel's value to 15 digits.

## Context
- Every one of these cells is inside Gate A's envelope: within 1e-9 relative of the 50-digit math (most are within
  1e-13). Excel's own error against the math is up to 1.5e-13 there. Under decision 0188 R5, last-digit work on the
  F.DIST density and cdf on cells already inside Gate A is Phase 1b, hence `defer: phase-1b`.
- T-3354 found no 5e-4 error on current main (see its Outcome). This task owns only the residue in the 15th digit.
- Most cells sit at d1 = 10 and 40 with a tiny x, where the result is far below 1, so the residue is the
  density prefactor and the incomplete beta's last bits. Related: T-2647 (density·x subnormal), T-2642 (last ulp at a
  huge df), T-3158, T-3173 and T-3174 (density after M-final), T-3364 (the extrapolated `STIRLING_ERROR` past df 56).
- Code: `fn_f_dist` in crates/rustsheet-engine/src/functions/statistical/fisher.rs, f_density.rs and the incomplete
  beta.
- probes/t3354-f-dist-tiny-x-huge-d2 (222 cells): A11, A15, A19, A23, A27, A31, A35, A39, A43, A47, A97, A137:A139,
  A141:A142, A145:A146, A148:A149, A151, A153:A166, A168:A173, A177:A182, A185:A191, A193:A240, B50:B52, B54, B58:B62,
  B65, B67:B70, B74, B76:B78, B82:B83, B86, B90, B92:B95, B97:B99, B105:B107, B110, B113:B115, B118, B121:B122, B126,
  B129:B130, B133:B134, B137, B139, B141, B143, B145, B148, B150, B153:B158, B162:B164, B170:B172, B175, B177,
  B179:B180, B182:B183, B185:B186, B188:B189, B191, B193:B199, B201:B239
- probes-numeric/t3354-f-dist-sweep (177 cells): A3:A4, A7:A9, A11:A13, A16:A17, A19:A21, A23:A24, A26:A30, A32:A33,
  A35, A38, A40, A45:A49, A55, A57, A59:A60, A65:A66, A70, A75:A77, A79, A81, A84, A89, A93, A96:A97, A99, A101:A103,
  A106:A110, A116:A117, A119, A121, A125, A129, A131, A133:A138, A140, A145, A163, A168, A177, A185, A194, B1, B4,
  B7:B13, B15:B16, B18:B19, B21:B27, B29:B33, B35:B36, B38, B40:B42, B45:B46, B48:B49, B53, B55, B57, B60, B65:B66,
  B68:B70, B72, B74, B76:B81, B85:B90, B93:B94, B96, B101:B103, B106:B107, B109:B111, B113:B114, B116, B119:B121,
  B125:B126, B129:B132, B134:B137, B140, B145, B147, B158, B163, B177, B184:B185, B187, B190, B194, B196:B197, C147,
  C168, C187
- Scorer: docs/research/t3354/score.py.

## Acceptance criteria
- [ ] Each owned cell is within half a unit of the 15th significant digit of Excel's value, or is moved to a narrower
      task with the reason recorded.
- [ ] Any fit follows decision 0188 R4 (hold-out split, fresh sweep, `FITTED` tag).
- [ ] No gated cell regresses; `make check` green; reviewers approve.

## Out of scope
- Bit parity on cells already within 15 digits.
