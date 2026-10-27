//! T-2464 research harness (an ignored test): evaluates Excel's elementary-function models on raw doubles for
//! docs/research/t2464/excelmath.py. Input file (env `T2464_IN`): lines `OP HEXBITS`; output (env `T2464_OUT`): one
//! hex result per line. `BRATIO A B X Y` (four hex doubles) writes three hex values instead: w and w1 of the
//! engine's K(a, b, x, y) (closed forms, then the ported branches, else today's kernel), the
//! `bratio::Branch` index bratio dispatches the point to, and 1 if a ported form computed it (T-2506 scores). It does nothing unless both variables
//! are set. Delete it once the engine carries the
//! kernel (T-2505) and the research no longer needs raw-double evaluation.
use std::io::Write;

#[test]
#[ignore]
fn t2464_kernels() {
    let (Ok(input), Ok(output)) = (std::env::var("T2464_IN"), std::env::var("T2464_OUT")) else {
        return;
    };
    let text = std::fs::read_to_string(input).expect("read T2464_IN");
    let mut out = std::io::BufWriter::new(std::fs::File::create(output).expect("create T2464_OUT"));
    for line in text.lines() {
        let mut it = line.split_whitespace();
        let (Some(op), Some(hex)) = (it.next(), it.next()) else {
            continue;
        };
        let bits = |h: &str| f64::from_bits(u64::from_str_radix(h, 16).expect("hex bits"));
        let x = bits(hex);
        if op == "BRATIO" {
            let rest: Vec<f64> = it.map(bits).collect();
            let (a, b, x, y) = (x, rest[0], rest[1], rest[2]);
            let ported = super::incomplete_beta::closed_form_tails(x, y, a, b)
                .or_else(|| super::bratio::ported_tails(a, b, x, y));
            let (w, w1) = ported.unwrap_or_else(|| {
                (
                    super::incomplete_beta::beta_reg_complement(x, y, a, b),
                    super::incomplete_beta::beta_reg_upper(x, y, a, b),
                )
            });
            let branch = super::bratio::dispatch(a, b, x, y).map_or(u64::MAX, |d| d.branch as u64);
            let ported = u8::from(ported.is_some());
            writeln!(out, "{:016x} {:016x} {branch} {ported}", w.to_bits(), w1.to_bits())
                .expect("write T2464_OUT");
            continue;
        }
        let r = match op {
            "EXP" => crate::x87_exp::excel_exp(x),
            "LN" => crate::x87_log10::excel_ln(x),
            "LOG10" => crate::x87_log10::excel_log10(x),
            "GAMMALN" => super::gammaln::excel_gammaln(x),
            "GAMMA" => super::gamma_x87::excel_gamma(x),
            _ => panic!("unknown op {op}"),
        };
        writeln!(out, "{:016x}", r.to_bits()).expect("write T2464_OUT");
    }
}
