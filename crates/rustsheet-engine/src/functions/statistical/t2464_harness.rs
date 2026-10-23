//! T-2464 research harness (an ignored test): evaluates Excel's elementary-function models on raw doubles for
//! docs/research/t2464/excelmath.py. Input file (env `T2464_IN`): lines `OP HEXBITS`; output (env `T2464_OUT`): one
//! hex result per line. It does nothing unless both variables are set.
use std::io::Write;

#[test]
#[ignore]
fn t2464_kernels() {
    let (Ok(input), Ok(output)) = (std::env::var("T2464_IN"), std::env::var("T2464_OUT")) else {
        return;
    };
    let text = std::fs::read_to_string(input).unwrap_or_default();
    let mut out = std::io::BufWriter::new(std::fs::File::create(output).expect("out"));
    for line in text.lines() {
        let mut it = line.split_whitespace();
        let (Some(op), Some(hex)) = (it.next(), it.next()) else {
            continue;
        };
        let x = f64::from_bits(u64::from_str_radix(hex, 16).unwrap_or(0));
        let r = match op {
            "EXP" => crate::x87_exp::excel_exp(x),
            "LN" => crate::x87_log10::excel_ln(x),
            "LOG10" => crate::x87_log10::excel_log10(x),
            "GAMMALN" => super::gammaln::excel_gammaln(x),
            "GAMMA" => super::gamma_x87::excel_gamma(x),
            _ => f64::NAN,
        };
        writeln!(out, "{:016x}", r.to_bits()).ok();
    }
}
