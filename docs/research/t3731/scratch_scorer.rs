// T-3731 attempt 2: the throwaway in-crate scorer (not compiled; kept as a record for attempt 3).
// To run it, append it to crates/rustsheet-engine/src/functions/financial/cumulative.rs, make
// annuity.rs `payment_pair` pub(super), write the TSV it reads (suite case, cell, function, args,
// Excel's value; one line per CUMPRINC/PPMT/PMT/CUMIPMT cell of probes/fin-cumipmt-tiny-rate-structure,
// fin-cumprinc-single-period-rate-scan and fin-cumprinc-dyadic-rates) to the path below, and run
// `cargo test -q -p rustsheet_engine scratch_score -- --nocapture`. It scores type 0 single-period
// CUMPRINC (425 cells); the ROW lines print the signed ulps of candidates 1, 13 and 20 on the rate scan.
#[cfg(test)]
mod scratch_t3731 {
    use super::*;

    fn ulps(a: f64, b: f64) -> i64 {
        (a.to_bits() as i64) - (b.to_bits() as i64)
    }

    #[test]
    fn scratch_score() {
        let path = "/tmp/claude-0/-home-user-rustsheet/2a30942d-0a16-55b2-8ac8-b7908236872e/scratchpad/cum.tsv";
        let Ok(text) = std::fs::read_to_string(path) else { return };
        let names = ["engine", "pmt*exp(-jlog1p)", "pmt*xexp(-jlog1p)", "pmt*xexp(-jL)", "pmt*powi", "pmt-r*B(pow)", "pmt-r*B(exp L)", "pmt/(1+r)^j powf", "loop dbl", "loop x87", "loop x87 sum", "loop dbl B-=", "loop x87 prin store", "pair*exp(-jlog1p) x87", "pair loop x87", "pair*xexp(-jL) x87", "FVdiff dbl", "FVdiff x87", "FVdiff x87 xexp", "ppmt-kernel L=log1p", "ppmt-kernel A(L),Q(log1p)", "pmt-ipmt", "pmt-ipmt log1p", "ppmt (ours)", "ppmt xL", "ppmt xL A(L)", "ppmt xL Q only", "pa", "pb", "pc", "pd", "pe xexp lg", "pf pv*r/-xexpm1(lg)"];
        let mut score = vec![[0usize; 4]; names.len()];
        let mut total = 0;
        for line in text.lines() {
            let f: Vec<&str> = line.split('\t').collect();
            if f[2] != "CUMPRINC" { continue; }
            let a: Vec<f64> = f[3].split(',').map(|s| s.parse().unwrap()).collect();
            let excel: f64 = f[4].parse().unwrap();
            let (r, n, pv, s, e, t) = (a[0], a[1], a[2], a[3], a[4], a[5] == 1.0);
            if s != e || t { continue; }
            total += 1;
            let p = Periods { rate: r, nper: n, pv, start: s, end: e, period_start: t };
            let pmt = compute_payment(r, n, pv, 0.0, t).unwrap();
            let pair = super::super::annuity::payment_pair(r, n, pv, 0.0, t).unwrap();
            let j = n - s + 1.0;
            let l1 = r.ln_1p();
            let lg = growth_log(r);
            let k1 = s - 1.0;
            let xl = {
                use crate::x87_register::Register as R;
                let sr = R::load(r / (2.0 + r));
                let sq = sr.mul(sr);
                let (mut pw, mut sm) = (sr, sr);
                for k in 1..=64u32 { pw = pw.mul(sq); let nx = sm.add(pw.div(R::load(f64::from(2 * k + 1)))); if nx.same_value(sm) { break } sm = nx; }
                sm.add(sm)
            };
            let xmul = |m: f64| { use crate::x87_register::Register as R; xl.mul(R::load(m)).store() };
            let bpow = pv * (1.0 + r).powf(k1) + pmt * (((1.0 + r).powf(k1) - 1.0) / r);
            let bexp = pv * (k1 * lg).exp() + pmt * ((k1 * lg).exp_m1() / r);
            let cands = [
                p.principal().unwrap(),
                pmt * (-j * l1).exp(),
                pmt * excel_exp(-j * l1),
                pmt * excel_exp(-j * lg),
                pmt * (1.0 + r).powi(-(j as i32)),
                pmt + r * bpow,
                pmt + r * bexp,
                pmt / (1.0 + r).powf(j),
                {
                    let mut b = pv; let mut sum = 0.0;
                    let mut k = 1.0;
                    while k <= e { let prin = pmt + b * r; b += prin; if k >= s { sum += prin } k += 1.0; }
                    sum
                },
                {
                    use crate::x87_register::Register as R;
                    let (rr, pm) = (R::load(r), R::load(pmt));
                    let mut b = R::load(pv); let mut sum = R::load(0.0);
                    let mut k = 1.0;
                    while k <= e { let prin = pm.add(b.mul(rr)); b = b.add(prin); if k >= s { sum = sum.add(prin) } k += 1.0; }
                    sum.store()
                },
                {
                    use crate::x87_register::Register as R;
                    let (rr, pm) = (R::load(r), R::load(pmt));
                    let mut b = R::load(pv); let mut sum = 0.0;
                    let mut k = 1.0;
                    while k <= e { let prin = pm.add(b.mul(rr)); b = R::load(b.add(prin).store()); if k >= s { sum += prin.store() } k += 1.0; }
                    sum
                },
                {
                    let mut b = pv; let mut sum = 0.0;
                    let mut k = 1.0;
                    while k <= e { let int = b * r; let prin = pmt + int; b = b + pmt + int; if k >= s { sum += prin } k += 1.0; }
                    sum
                },
                {
                    use crate::x87_register::Register as R;
                    let (rr, pm) = (R::load(r), R::load(pmt));
                    let mut b = R::load(pv); let mut sum = 0.0;
                    let mut k = 1.0;
                    while k <= e { let prin = R::load(pm.add(b.mul(rr)).store()); b = b.add(prin); if k >= s { sum += prin.store() } k += 1.0; }
                    sum
                },
                {
                    use crate::x87_register::Register as R;
                    R::from_double_double(pair).mul(R::load((-j * l1).exp())).store()
                },
                {
                    use crate::x87_register::Register as R;
                    let (rr, pm) = (R::load(r), R::from_double_double(pair));
                    let mut b = R::load(pv); let mut sum = R::load(0.0);
                    let mut k = 1.0;
                    while k <= e { let prin = pm.add(b.mul(rr)); b = b.add(prin); if k >= s { sum = sum.add(prin) } k += 1.0; }
                    sum.store()
                },
                {
                    use crate::x87_register::Register as R;
                    R::from_double_double(pair).mul(R::load(excel_exp(-j * lg))).store()
                },
                {
                    let bal = |k: f64| { let g = (k * l1).exp(); pv * g + pmt * ((k * l1).exp_m1() / r) };
                    -(bal(s - 1.0) - bal(e))
                },
                {
                    use crate::x87_register::Register as R;
                    let bal = |k: f64| { let g = R::load((k * l1).exp()); R::load(pv).mul(g).add(R::load(pmt).mul(R::load((k * l1).exp_m1()).div(R::load(r)))) };
                    bal(e).add(bal(s - 1.0).neg()).store()
                },
                {
                    use crate::x87_register::Register as R;
                    let bal = |k: f64| { let g = R::load(excel_exp(k * lg)); R::load(pv).mul(g).add(R::load(pmt).mul(R::load(excel_expm1(k * lg)).div(R::load(r)))) };
                    bal(e).add(bal(s - 1.0).neg()).store()
                },
                {
                    let a = pv / -(-n * l1).exp_m1();
                    -(a * excel_exp(-j * l1) * r)
                },
                {
                    let a = pv / -excel_expm1(-n * lg);
                    -(a * excel_exp(-j * l1) * r)
                },
                {
                    let a = pv / -excel_expm1(-n * lg);
                    let ipmt = -((a * -excel_expm1(-j * lg)) * r);
                    pmt - ipmt
                },
                {
                    let a = pv / -excel_expm1(-n * lg);
                    let ipmt = -((a * -excel_expm1(-j * l1)) * r);
                    pmt - ipmt
                },
                {
                    let a = pv / -excel_expm1(-n * lg);
                    -(a * excel_exp(-j * lg) * r)
                },
                {
                    let a = pv / -excel_expm1(xmul(-n));
                    -(a * excel_exp(xmul(-j)) * r)
                },
                {
                    let a = pv / -excel_expm1(-n * lg);
                    -(a * excel_exp(xmul(-j)) * r)
                },
                {
                    pmt * excel_exp(xmul(-j))
                },
                -(pv * r / -(-n * l1).exp_m1()) * (-j * l1).exp(),
                -(pv / -(-n * l1).exp_m1() * r) * (-j * l1).exp(),
                -(pv * r / (1.0 - (-n * l1).exp())) * (-j * l1).exp(),
                -(pv * r / (1.0 - (1.0 + r).powf(-n))) * (1.0 + r).powf(-j),
                -(pv * r / -excel_expm1(-n * lg)) * excel_exp(-j * lg),
                -(pv * r / -excel_expm1(-n * lg)) * (-j * l1).exp(),
            ];
            if f[0] == "fin-cumprinc-single-period-rate-scan" {
                println!("ROW {r:e} {s} {} {} {}", ulps(cands[1], excel), ulps(cands[13], excel), ulps(cands[20], excel));
            }
            for (i, c) in cands.iter().enumerate() {
                let u = ulps(*c, excel).unsigned_abs();
                if u == 0 { score[i][0] += 1 }
                if u <= 1 { score[i][1] += 1 }
                if u <= 4 { score[i][2] += 1 }
            }
        }
        for (i, n) in names.iter().enumerate() {
            println!("{n:22} exact {:4} <=1 {:4} <=4 {:4} of {total}", score[i][0], score[i][1], score[i][2]);
        }
    }
}
