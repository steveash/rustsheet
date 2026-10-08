#![allow(clippy::unwrap_used, reason = "tests")]

//! Metamorphic family (T-1149, SPEC-0004 test plan): `SUM((r1,r2,..))` must equal
//! `SUM(r1,r2,..)` for same-sheet, non-3-D references, overlapping areas counted once per area
//! (probes/multiarea-agg H1:H3; docs/excel-semantics/references.md "Multi-area references in
//! functions"). No oracle needed: the two cells of each pair must agree with each other.

use crate::test::util::new_empty_model;

const PAIRS: usize = 120;

/// Small deterministic generator (SplitMix64); the family is reproducible from `SEED`.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

const SEED: u64 = 0x5EED_1149;
const ROWS: u64 = 8;
const COLS: u64 = 4;

fn col_name(c: u64) -> char {
    char::from(b'A' + u8::try_from(c).unwrap())
}

/// A random single cell, range or whole-column reference inside the data block.
fn operand(rng: &mut Rng) -> String {
    match rng.below(4) {
        0 => format!("{}{}", col_name(rng.below(COLS)), 1 + rng.below(ROWS)),
        1 | 2 => {
            let (c1, c2) = (rng.below(COLS), rng.below(COLS));
            let (r1, r2) = (1 + rng.below(ROWS), 1 + rng.below(ROWS));
            format!(
                "{}{}:{}{}",
                col_name(c1.min(c2)),
                r1.min(r2),
                col_name(c1.max(c2)),
                r1.max(r2)
            )
        }
        _ => {
            let c = col_name(rng.below(COLS));
            format!("{c}:{c}")
        }
    }
}

#[test]
fn union_sum_equals_separate_arguments() {
    let mut model = new_empty_model();
    let mut rng = Rng(SEED);
    // Numbers, blanks, text and booleans: SUM skips the last three in references.
    for r in 1..=ROWS {
        for c in 0..COLS {
            let cell = format!("{}{r}", col_name(c));
            match rng.below(8) {
                0 => {}
                1 => model._set(&cell, "\"x\""),
                2 => model._set(&cell, "TRUE"),
                _ => model._set(&cell, &(rng.below(2000) as i64 - 1000).to_string()),
            }
        }
    }
    let mut pairs = Vec::new();
    for i in 0..PAIRS {
        let n = 2 + rng.below(2) as usize;
        let mut operands: Vec<String> = (0..n).map(|_| operand(&mut rng)).collect();
        if i % 5 == 0 {
            // Force overlapping areas: the same operand twice.
            operands[1] = operands[0].clone();
        }
        let joined = operands.join(",");
        let (union_cell, plain_cell) = (format!("G{}", i + 1), format!("H{}", i + 1));
        model._set(&union_cell, &format!("=SUM(({joined}))"));
        model._set(&plain_cell, &format!("=SUM({joined})"));
        pairs.push((union_cell, plain_cell, joined));
    }
    model.evaluate();
    for (union_cell, plain_cell, joined) in pairs {
        assert_eq!(
            model._get_text(&union_cell),
            model._get_text(&plain_cell),
            "SUM(({joined})) vs SUM({joined})"
        );
    }
}
