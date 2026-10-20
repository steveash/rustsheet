// T-0969 attempt 6 probe harness (2026-10-20). Not compiled: to rerun, copy it to
// crates/rustsheet-engine/src/functions/statistical/t_probe.rs, add `#[cfg(test)] mod t_probe;` to
// that module's mod.rs, and run
//   TCELLS=docs/research/t0969/tcells.tsv cargo test --release -p rustsheet_engine --lib t_probe -- --nocapture
// tcells.tsv: function, t, df, flag, Excel value, engine value (main a15f80ae) for every literal-argument
// T.DIST / T.DIST.RT / T.DIST.2T / TDIST cell in eval/runs/latest/report.json (excel-graph values).
#![allow(clippy::all, clippy::pedantic, missing_docs, unused)]
use super::gamma_x87::excel_gamma;
use super::gammaln::excel_gammaln as gl;
use crate::arithmetic::{extended_product as mul, extended_quotient as div, extended_sqrt as sqrt, extended_sum as add, power};
use crate::x87_exp::excel_exp as exp;
use crate::x87_log10::excel_ln as ln;

fn cells(kind: &str) -> Vec<(f64, f64, f64)> {
    let path = std::env::var("TCELLS").unwrap();
    let text = std::fs::read_to_string(path).unwrap();
    text.lines().filter_map(|l| {
        let f: Vec<&str> = l.split('\t').collect();
        let k = format!("{}|{}", f[0], f[3]);
        (k == kind).then(|| (f[1].parse().unwrap(), f[2].parse::<f64>().unwrap().trunc(), f[4].parse().unwrap()))
    }).collect()
}
fn ulps(a: f64, b: f64) -> i64 { (a.to_bits() as i64 - b.to_bits() as i64) }
fn sub(a: f64, b: f64) -> f64 { add(a, -b) }
fn pw(x: f64, y: f64) -> f64 { power(x, y).unwrap_or(f64::NAN) }
const PI: f64 = std::f64::consts::PI;

fn score(name: &str, cs: &[(f64, f64, f64)], f: &dyn Fn(f64, f64) -> f64) {
    let mut exact = 0; let mut w4 = 0; let mut worst = 0i64;
    for &(t, n, e) in cs {
        let v = f(t, n);
        let u = ulps(v, e).abs();
        if u == 0 { exact += 1 } if u <= 4 { w4 += 1 } worst = worst.max(u);
    }
    println!("{name:40} exact {exact:3}/{} <=4 {w4:3} worst {worst}", cs.len());
}

#[test]
fn probe_density() {
    let cs = cells("T.DIST|FALSE");
    let h = |n: f64| (n + 1.0) / 2.0;
    let ratios: Vec<(&str, Box<dyn Fn(f64) -> f64>)> = vec![
        ("exp(gl-gl)", Box::new(move |n| exp(sub(gl(h(n)), gl(n / 2.0))))),
        ("g/g", Box::new(move |n| div(excel_gamma(h(n)), excel_gamma(n / 2.0)))),
        ("exp(gl)/exp(gl)", Box::new(move |n| div(exp(gl(h(n))), exp(gl(n / 2.0))))),
    ];
    let norms: Vec<(&str, Box<dyn Fn(f64, f64) -> f64>)> = vec![
        ("/sqrt(n*pi)", Box::new(|r, n| div(r, sqrt(mul(n, PI))))),
        ("/sqrt(pi*n)", Box::new(|r, n| div(r, sqrt(mul(PI, n))))),
        ("/sqrt(n)/sqrt(pi)", Box::new(|r, n| div(div(r, sqrt(n)), sqrt(PI)))),
        ("/(sqrt(n)*sqrt(pi))", Box::new(|r, n| div(r, mul(sqrt(n), sqrt(PI))))),
        ("*1/sqrt(n*pi)", Box::new(|r, n| mul(r, div(1.0, sqrt(mul(n, PI)))))),
    ];
    let powers: Vec<(&str, Box<dyn Fn(f64, f64) -> f64>)> = vec![
        ("pow(1+t2/n,-h)", Box::new(move |t, n| pw(add(1.0, div(mul(t, t), n)), -h(n)))),
        ("pow(1+t^2/n,-h)", Box::new(move |t, n| pw(add(1.0, div(pw(t, 2.0), n)), -h(n)))),
        ("1/pow(1+t2/n,h)", Box::new(move |t, n| div(1.0, pw(add(1.0, div(mul(t, t), n)), h(n))))),
        ("pow(n/(n+t2),h)", Box::new(move |t, n| pw(div(n, add(n, mul(t, t))), h(n)))),
        ("exp(-h ln(1+t2/n))", Box::new(move |t, n| exp(-mul(h(n), ln(add(1.0, div(mul(t, t), n))))))),
        ("exp(-h*ln((n+t2)/n))", Box::new(move |t, n| exp(-mul(h(n), ln(div(add(n, mul(t, t)), n)))))),
        ("exp(h*ln(n/(n+t2)))", Box::new(move |t, n| exp(mul(h(n), ln(div(n, add(n, mul(t, t)))))))),
        ("pow(1+t2/n,-(n+1)/2)'", Box::new(move |t, n| pw(add(1.0, div(mul(t, t), n)), -(div(add(n, 1.0), 2.0))))),
    ];
    for (rn, r) in &ratios { for (nn, no) in &norms { for (pn, p) in &powers {
        score(&format!("{rn} {nn} {pn}"), &cs, &|t, n| mul(no(r(n), n), p(t, n)));
    }}}
    for (rn, r) in &ratios { for (pn, p) in &powers {
        score(&format!("{rn} * pow / sqrt(n pi) {pn}"), &cs, &|t, n| div(mul(r(n), p(t, n)), sqrt(mul(n, PI))));
    }}
    // single exp
    score("exp(gl-gl-h ln(1+t2/n))/sqrt(n pi)", &cs, &|t, n| div(exp(sub(sub(gl(h(n)), gl(n / 2.0)), mul(h(n), ln(add(1.0, div(mul(t, t), n)))))), sqrt(mul(n, PI))));
    score("exp(gl-gl-.5ln(n pi)-h ln(..))", &cs, &|t, n| exp(sub(sub(sub(gl(h(n)), gl(n / 2.0)), mul(0.5, ln(mul(n, PI)))), mul(h(n), ln(add(1.0, div(mul(t, t), n)))))));
    score("ours", &cs, &|t, n| super::student_t::student_t_density(t, n).unwrap_or(f64::NAN));
}

#[test]
fn probe_constant() {
    let cs: Vec<_> = cells("T.DIST|FALSE").into_iter().filter(|c| c.0 == 0.0).collect();
    let h = |n: f64| (n + 1.0) / 2.0;
    let sp = sqrt(PI);
    let lnb = |n: f64| sub(add(gl(n / 2.0), gl(0.5)), gl(h(n)));
    let cands: Vec<(&str, Box<dyn Fn(f64) -> f64>)> = vec![
        ("1/(sqrt n * B) B=exp(lnb)", Box::new(move |n| div(1.0, mul(sqrt(n), exp(lnb(n)))))),
        ("1/(sqrt n * B) B=g g/g", Box::new(move |n| div(1.0, mul(sqrt(n), div(mul(excel_gamma(n / 2.0), excel_gamma(0.5)), excel_gamma(h(n))))))),
        ("exp(-lnb)/sqrt n", Box::new(move |n| div(exp(-lnb(n)), sqrt(n)))),
        ("exp(-lnb - .5 ln n)", Box::new(move |n| exp(sub(-lnb(n), mul(0.5, ln(n)))))),
        ("exp(gl h - gl a - .5 ln(n pi))", Box::new(move |n| exp(sub(sub(gl(h(n)), gl(n / 2.0)), mul(0.5, ln(mul(n, PI))))))),
        ("exp(gl h - gl a - .5 ln n - .5 ln pi)", Box::new(move |n| exp(sub(sub(sub(gl(h(n)), gl(n / 2.0)), mul(0.5, ln(n))), mul(0.5, ln(PI)))))),
        ("exp(gl h - gl a - ln(sqrt(n pi)))", Box::new(move |n| exp(sub(sub(gl(h(n)), gl(n / 2.0)), ln(sqrt(mul(n, PI))))))),
        ("exp(gl h - (gl a + .5 ln(n pi)))", Box::new(move |n| exp(sub(gl(h(n)), add(gl(n / 2.0), mul(0.5, ln(mul(n, PI)))))))),
        ("g h/(sqrt(n pi) g a)", Box::new(move |n| div(excel_gamma(h(n)), mul(sqrt(mul(n, PI)), excel_gamma(n / 2.0))))),
        ("g h/sqrt(n pi)/g a", Box::new(move |n| div(div(excel_gamma(h(n)), sqrt(mul(n, PI))), excel_gamma(n / 2.0)))),
        ("g h/(g a sqrt(n pi))", Box::new(move |n| div(excel_gamma(h(n)), mul(excel_gamma(n / 2.0), sqrt(mul(n, PI)))))),
        ("g h/(g a sqrt n sqrt pi)", Box::new(move |n| div(excel_gamma(h(n)), mul(mul(excel_gamma(n / 2.0), sqrt(n)), sp)))),
        ("exp(gl h-gl a)/sqrt(n pi)", Box::new(move |n| div(exp(sub(gl(h(n)), gl(n / 2.0))), sqrt(mul(n, PI))))),
        ("exp(gl h)/(exp(gl a) sqrt(n pi))", Box::new(move |n| div(exp(gl(h(n))), mul(exp(gl(n / 2.0)), sqrt(mul(n, PI)))))),
        ("exp(gl h-gl a-gl .5)/sqrt n", Box::new(move |n| div(exp(sub(sub(gl(h(n)), gl(n / 2.0)), gl(0.5))), sqrt(n)))),
    ];
    for (name, c) in &cands {
        let mut s = String::new();
        let mut ex = 0;
        for &(_, n, e) in &cs { let u = ulps(c(n), e); if u == 0 { ex += 1 } s += &format!(" {n}:{u}"); }
        println!("{name:40} {ex}/{} {s}", cs.len());
    }
}

#[test]
fn probe_tail_shape() {
    let cs: Vec<_> = cells("T.DIST|FALSE").into_iter().filter(|c| c.0.abs() >= 10.0 && c.1 >= 2.0).collect();
    let h = |n: f64| (n + 1.0) / 2.0;
    let ours = |t: f64, n: f64| super::student_t::student_t_density(t, n).unwrap_or(f64::NAN);
    let cands: Vec<(&str, Box<dyn Fn(f64, f64) -> f64>)> = vec![
        ("ours", Box::new(ours)),
        ("c*exp(-h ln(1+t2/n))", Box::new(move |t, n| mul(ours(0.0, n), exp(-mul(h(n), ln(add(1.0, div(mul(t, t), n)))))))),
        ("c*exp(-h ln((n+t2)/n))", Box::new(move |t, n| mul(ours(0.0, n), exp(-mul(h(n), ln(div(add(n, mul(t, t)), n))))))),
        ("c*exp(h ln(n/(n+t2)))", Box::new(move |t, n| mul(ours(0.0, n), exp(mul(h(n), ln(div(n, add(n, mul(t, t))))))))),
        ("c*exp(h (ln n - ln(n+t2)))", Box::new(move |t, n| mul(ours(0.0, n), exp(mul(h(n), sub(ln(n), ln(add(n, mul(t, t))))))))),
        ("c*exp(-h*log1p)", Box::new(move |t, n| mul(ours(0.0, n), exp(-mul(h(n), (t * t / n).ln_1p()))))),
        ("c*pow(1+t2/n,-h)", Box::new(move |t, n| mul(ours(0.0, n), pw(add(1.0, div(mul(t, t), n)), -h(n))))),
        ("c/pow(1+t2/n,h)", Box::new(move |t, n| div(ours(0.0, n), pw(add(1.0, div(mul(t, t), n)), h(n))))),
        ("c*pow(n/(n+t2),h)", Box::new(move |t, n| mul(ours(0.0, n), pw(div(n, add(n, mul(t, t))), h(n))))),
        ("c*exp(-(n+1) ln(..)/2)", Box::new(move |t, n| mul(ours(0.0, n), exp(-div(mul(add(n, 1.0), ln(add(1.0, div(mul(t, t), n)))), 2.0))))),
        ("c*exp(-((n+1) ln(..))/2)'", Box::new(move |t, n| mul(ours(0.0, n), exp(mul(-0.5, mul(add(n, 1.0), ln(add(1.0, div(mul(t, t), n))))))))),
    ];
    let mut head = String::from("                              ");
    for &(t, n, _) in &cs { head += &format!(" {:>9}", format!("{n}/{t}")); }
    println!("{head}");
    for (name, c) in &cands {
        let mut s = String::new();
        for &(t, n, e) in &cs { s += &format!(" {:>9}", ulps(c(t, n), e)); }
        println!("{name:30}{s}");
    }
}

#[test]
fn probe_n2() {
    let cs: Vec<_> = cells("T.DIST|FALSE").into_iter().filter(|c| c.1 == 2.0).collect();
    let cands: Vec<(&str, Box<dyn Fn(f64, f64) -> f64>)> = vec![
        ("pow(2+t2,-1.5)", Box::new(|t, n| pw(add(n, mul(t, t)), -1.5))),
        ("exp(-1.5 ln(2+t2))", Box::new(|t, n| exp(-mul(1.5, ln(add(n, mul(t, t))))))),
        ("exp(-1.5 lnlibm(2+t2))", Box::new(|t, n| exp(-mul(1.5, add(n, mul(t, t)).ln())))),
        ("libm exp(-1.5 ln(2+t2))", Box::new(|t, n| (-mul(1.5, ln(add(n, mul(t, t))))).exp())),
        ("libm exp(-1.5 lnlibm(2+t2))", Box::new(|t, n| (-mul(1.5, add(n, mul(t, t)).ln())).exp())),
        ("1/((2+t2)*sqrt(2+t2))", Box::new(|t, n| { let s = add(n, mul(t, t)); div(1.0, mul(s, sqrt(s))) })),
        ("ln-domain full", Box::new(|t, n| exp(sub(sub(sub(gl(1.5), gl(1.0)), mul(0.5, ln(mul(n, PI)))), mul(1.5, ln(add(1.0, div(mul(t, t), n)))))))),
    ];
    for (name, c) in &cands {
        let mut s = String::new(); let mut ex = 0;
        for &(t, n, e) in &cs { let u = ulps(c(t, n), e); if u == 0 { ex += 1 } s += &format!(" {t}:{u}"); }
        println!("{name:34} {ex}/{} {s}", cs.len());
    }
}

#[test]
fn probe_general() {
    let cs = cells("T.DIST|FALSE");
    let h = |n: f64| (n + 1.0) / 2.0;
    let lp = |t: f64, n: f64| mul(h(n), ln(add(n, mul(t, t))));
    let lpl = |t: f64, n: f64| mul(h(n), add(n, mul(t, t)).ln());
    let lnpi = ln(PI);
    let cands: Vec<(&str, Box<dyn Fn(f64, f64) -> f64>)> = vec![
        ("exp(gl h - gl a - .5lnpi + a ln n - h ln(n+t2))", Box::new(move |t, n| exp(sub(add(sub(sub(gl(h(n)), gl(n / 2.0)), mul(0.5, lnpi)), mul(n / 2.0, ln(n))), lp(t, n))))),
        ("exp(a ln n + gl h - gl a - .5lnpi - hl)", Box::new(move |t, n| exp(sub(sub(sub(add(mul(n / 2.0, ln(n)), gl(h(n))), gl(n / 2.0)), mul(0.5, lnpi)), lp(t, n))))),
        ("exp(gl h - gl a - gl .5 + a ln n - hl)", Box::new(move |t, n| exp(sub(add(sub(sub(gl(h(n)), gl(n / 2.0)), gl(0.5)), mul(n / 2.0, ln(n))), lp(t, n))))),
        ("exp(a ln n - hl + gl h - gl a - gl.5)", Box::new(move |t, n| exp(sub(sub(add(sub(mul(n / 2.0, ln(n)), lp(t, n)), gl(h(n))), gl(n / 2.0)), gl(0.5))))),
        ("exp(a ln n - hl - lnB)", Box::new(move |t, n| exp(sub(sub(mul(n / 2.0, ln(n)), lp(t, n)), sub(add(gl(n / 2.0), gl(0.5)), gl(h(n))))))),
        ("exp(a ln n - lnB - hl)", Box::new(move |t, n| exp(sub(sub(mul(n / 2.0, ln(n)), sub(add(gl(n / 2.0), gl(0.5)), gl(h(n)))), lp(t, n))))),
        ("exp(-lnB + a ln n - hl)", Box::new(move |t, n| exp(sub(add(-sub(add(gl(n / 2.0), gl(0.5)), gl(h(n))), mul(n / 2.0, ln(n))), lp(t, n))))),
        ("exp(gl h-gl a-gl .5)*n^a*exp(-hl)", Box::new(move |t, n| mul(mul(exp(sub(sub(gl(h(n)), gl(n / 2.0)), gl(0.5))), pw(n, n / 2.0)), exp(-lp(t, n))))),
        ("exp(a ln n - hl)/B", Box::new(move |t, n| div(exp(sub(mul(n / 2.0, ln(n)), lp(t, n))), exp(sub(add(gl(n / 2.0), gl(0.5)), gl(h(n))))))),
        ("exp(a ln n - hl)*exp(gl h - gl a - gl.5)", Box::new(move |t, n| mul(exp(sub(mul(n / 2.0, ln(n)), lp(t, n))), exp(sub(sub(gl(h(n)), gl(n / 2.0)), gl(0.5)))))),
        ("exp(a ln n - hl)*g h/(g a sqrt pi)", Box::new(move |t, n| mul(exp(sub(mul(n / 2.0, ln(n)), lp(t, n))), div(excel_gamma(h(n)), mul(excel_gamma(n / 2.0), sqrt(PI)))))),
        ("exp(a ln n - hl)*g h/g a/sqrt pi", Box::new(move |t, n| div(mul(exp(sub(mul(n / 2.0, ln(n)), lp(t, n))), div(excel_gamma(h(n)), excel_gamma(n / 2.0))), sqrt(PI)))),
        ("exp(a ln n - hl)/B(g)", Box::new(move |t, n| div(exp(sub(mul(n / 2.0, ln(n)), lp(t, n))), div(mul(excel_gamma(n / 2.0), excel_gamma(0.5)), excel_gamma(h(n)))))),
        ("libmln exp(gl h - gl a - gl.5 + a ln n - hl)", Box::new(move |t, n| exp(sub(add(sub(sub(gl(h(n)), gl(n / 2.0)), gl(0.5)), mul(n / 2.0, n.ln())), lpl(t, n))))),
    ];
    for (name, c) in &cands { score(name, &cs, c.as_ref()); }
    for (name, c) in &cands {
        let mut s = String::new();
        for &(t, n, e) in &cs { if t == 0.0 || t == 10.0 { s += &format!(" {n}/{t}:{}", ulps(c(t, n), e)); } }
        println!("{name:44} {s}");
    }
}

#[test]
fn probe_k() {
    let ks: [(f64, f64, f64); 7] = [(1.0, 0.3183098861837907, -1.1447298858494002), (3.0, 3.307973372530752, 1.1963357277127096), (4.0, 12.0, 2.4849066497880004), (5.0, 47.450836227811806, 3.859694148247577), (10.0, 123046.875, 11.720320659316299), (30.0, 3.10936039576862e+22, 51.79128909038456), (100.0, 3.979461869358938e+100, 231.63965590085562)];
    let all = cells("T.DIST|FALSE");
    for &(n, k, lk) in &ks {
        let cs: Vec<_> = all.iter().copied().filter(|c| c.1 == n).collect();
        let h = (n + 1.0) / 2.0;
        let cands: Vec<(&str, Box<dyn Fn(f64) -> f64>)> = vec![
            ("K*exp(-h ln(n+t2))", Box::new(move |t| mul(k, exp(-mul(h, ln(add(n, mul(t, t)))))))),
            ("exp(-h ln(n+t2))*K", Box::new(move |t| mul(exp(-mul(h, ln(add(n, mul(t, t))))), k))),
            ("K/exp(h ln(n+t2))", Box::new(move |t| div(k, exp(mul(h, ln(add(n, mul(t, t)))))))),
            ("exp(lnK - h ln(n+t2))", Box::new(move |t| exp(sub(lk, mul(h, ln(add(n, mul(t, t)))))))),
            ("K*pow(n+t2,-h)", Box::new(move |t| mul(k, pw(add(n, mul(t, t)), -h)))),
        ];
        for (name, c) in &cands {
            let mut s = String::new(); let mut ex = 0;
            for &(t, _, e) in &cs { let u = ulps(c(t), e); if u == 0 { ex += 1 } s += &format!(" {t}:{u}"); }
            println!("n={n} {name:24} {ex}/{} {s}", cs.len());
        }
    }
}

fn perms(v: Vec<usize>) -> Vec<Vec<usize>> {
    if v.len() <= 1 { return vec![v]; }
    let mut out = vec![];
    for i in 0..v.len() { let mut r = v.clone(); let x = r.remove(i); for mut p in perms(r) { p.insert(0, x); out.push(p); } }
    out
}

#[test]
fn probe_perm() {
    let cs: Vec<_> = cells("T.DIST|FALSE").into_iter().filter(|c| c.1 >= 2.0 && c.1 <= 100.0).collect();
    let h = |n: f64| (n + 1.0) / 2.0;
    type T = Box<dyn Fn(f64, f64) -> f64>;
    let pis: Vec<(&str, T)> = vec![
        ("-.5lnpi", Box::new(|_, _| -mul(0.5, ln(PI)))),
        ("-gl.5", Box::new(|_, _| -gl(0.5))),
        ("-lnsqrtpi", Box::new(|_, _| -ln(sqrt(PI)))),
        ("-0.5723649429247001", Box::new(|_, _| -0.572_364_942_924_700_1)),
    ];
    let pows: Vec<(&str, T, T)> = vec![
        ("a ln n, -h ln(n+t2)", Box::new(|_, n| mul(n / 2.0, ln(n))), Box::new(move |t, n| -mul(h(n), ln(add(n, mul(t, t)))))),
        ("-a ln(1+t2/n), -.5 ln(n+t2)", Box::new(|t, n| -mul(n / 2.0, ln(add(1.0, div(mul(t, t), n))))), Box::new(|t, n| -mul(0.5, ln(add(n, mul(t, t)))))),
        ("-h ln(1+t2/n), -.5 ln n", Box::new(move |t, n| -mul(h(n), ln(add(1.0, div(mul(t, t), n))))), Box::new(|_, n| -mul(0.5, ln(n)))),
        ("a ln(n/(n+t2)), -.5 ln(n+t2)", Box::new(|t, n| mul(n / 2.0, ln(div(n, add(n, mul(t, t)))))), Box::new(|t, n| -mul(0.5, ln(add(n, mul(t, t)))))),
        ("h ln(n/(n+t2)), -.5 ln n", Box::new(move |t, n| mul(h(n), ln(div(n, add(n, mul(t, t)))))), Box::new(|_, n| -mul(0.5, ln(n)))),
    ];
    let mut results = vec![];
    for (pn, pi) in &pis { for (wn, w1, w2) in &pows {
        let terms: Vec<&T> = vec![w1, w2, pi];
        let glh: T = Box::new(move |_, n| gl(h(n)));
        let gla: T = Box::new(|_, n| -gl(n / 2.0));
        let all: Vec<&T> = vec![terms[0], terms[1], terms[2], &glh, &gla];
        for p in perms((0..5).collect()) {
            let f = |t: f64, n: f64| { let mut s = all[p[0]](t, n); for &i in &p[1..] { s = add(s, all[i](t, n)); } exp(s) };
            let mut ex = 0; let mut w4 = 0;
            for &(t, n, e) in &cs { let u = ulps(f(t, n), e).abs(); if u == 0 { ex += 1 } if u <= 4 { w4 += 1 } }
            results.push((ex, w4, format!("{pn} | {wn} | {p:?}")));
        }
    }}
    results.sort();
    for r in results.iter().rev().take(15) { println!("{} {} /{} {}", r.0, r.1, cs.len(), r.2); }
}

#[test]
fn probe_n2_tail() {
    let mut cs: Vec<(f64, f64, f64)> = cells("T.DIST.RT|").into_iter().filter(|c| c.1 == 2.0 && c.0 > 0.0).collect();
    cs.extend(cells("T.DIST.2T|").into_iter().filter(|c| c.1 == 2.0 && c.0 > 0.0).map(|(t, n, e)| (t, n, e / 2.0)));
    let cands: Vec<(&str, Box<dyn Fn(f64) -> f64>)> = vec![
        ("ours", Box::new(|t| super::student_t::student_t_tails(t, 2.0).1)),
        ("0.5-0.5t/sqrt(2+t2)", Box::new(|t| sub(0.5, mul(0.5, div(t, sqrt(add(2.0, mul(t, t)))))))),
        ("(1-t/sqrt(2+t2))/2", Box::new(|t| div(sub(1.0, div(t, sqrt(add(2.0, mul(t, t))))), 2.0))),
        ("0.5-t/(2sqrt(2+t2))", Box::new(|t| sub(0.5, div(t, mul(2.0, sqrt(add(2.0, mul(t, t)))))))),
        ("0.5(1-t exp(-.5ln(2+t2)))", Box::new(|t| mul(0.5, sub(1.0, mul(t, exp(-mul(0.5, ln(add(2.0, mul(t, t)))))))))),
        ("1/(sqrt(2+t2)(sqrt(2+t2)+t))", Box::new(|t| { let s = sqrt(add(2.0, mul(t, t))); div(1.0, mul(s, add(s, t))) })),
        ("1/((2+t2)+t sqrt(2+t2))", Box::new(|t| { let s = sqrt(add(2.0, mul(t, t))); div(1.0, add(add(2.0, mul(t, t)), mul(t, s))) })),
        ("0.5-0.5 sqrt(t2/(2+t2))", Box::new(|t| sub(0.5, mul(0.5, sqrt(div(mul(t, t), add(2.0, mul(t, t)))))))),
        ("0.5-0.5 sqrt(1-2/(2+t2))", Box::new(|t| sub(0.5, mul(0.5, sqrt(sub(1.0, div(2.0, add(2.0, mul(t, t))))))))),
    ];
    for (name, c) in &cands {
        let mut s = String::new(); let mut ex = 0;
        for &(t, _, e) in &cs { let u = ulps(c(t), e); if u == 0 { ex += 1 } s += &format!(" {t}:{u}"); }
        println!("{name:32} {ex}/{} {s}", cs.len());
    }
}

#[test]
fn probe_rounded_y() {
    let mut cs: Vec<(f64, f64, f64)> = cells("T.DIST.RT|").into_iter().filter(|c| c.0 > 0.0).collect();
    cs.extend(cells("T.DIST.2T|").into_iter().filter(|c| c.0 > 0.0).map(|(t, n, e)| (t, n, e / 2.0)));
    let ours = |t: f64, n: f64| super::student_t::student_t_tails(t, n).1;
    let ry = |t: f64, n: f64| { let y = n / (n + t * t); super::student_t::student_t_tail_at(y, n) };
    let mut rows = std::collections::BTreeMap::new();
    for &(t, n, e) in &cs {
        if t * t >= n { continue; }
        let a = ulps(ours(t, n), e); let b = ulps(ry(t, n), e);
        rows.entry(n as u64).or_insert_with(Vec::new).push(format!("{t}:{a}/{b}"));
    }
    for (n, r) in rows { println!("n={n} {}", r.join(" ")); }
}

#[test]
fn probe_c_scan() {
    let ks: [(f64, f64); 6] = [(3.0, 1.1963357277127096), (4.0, 2.4849066497880004), (5.0, 3.859694148247577), (10.0, 11.720320659316299), (30.0, 51.79128909038456), (100.0, 231.63965590085562)];
    let all = cells("T.DIST|FALSE");
    for &(n, lk) in &ks {
        let cs: Vec<_> = all.iter().copied().filter(|c| c.1 == n).collect();
        let h = (n + 1.0) / 2.0;
        let forms: Vec<(&str, Box<dyn Fn(f64, f64) -> f64>)> = vec![
            ("exp(C - h ln(n+t2))", Box::new(move |c, t| exp(sub(c, mul(h, ln(add(n, mul(t, t)))))))),
            ("exp(C)*exp(-h ln(n+t2))", Box::new(move |c, t| mul(c.exp(), exp(-mul(h, ln(add(n, mul(t, t)))))))),
            ("exp(C - h ln(1+t2/n))", Box::new(move |c, t| exp(sub(c, mul(h, ln(add(1.0, div(mul(t, t), n)))))))),
        ];
        for (name, f) in &forms {
            let base = if name.contains("1+t2/n") { lk - h * n.ln() } else { lk };
            let mut best = (0, 0i64);
            for d in -3000i64..=3000 {
                let c = f64::from_bits((base.to_bits() as i64 + d) as u64);
                let ex = cs.iter().filter(|&&(t, _, e)| f(c, t) == e).count();
                if ex > best.0 { best = (ex, d); }
            }
            let c = f64::from_bits((base.to_bits() as i64 + best.1) as u64);
            let s: Vec<String> = cs.iter().map(|&(t, _, e)| format!("{t}:{}", ulps(f(c, t), e))).collect();
            println!("n={n} {name:26} best {}/{} at d={} {}", best.0, cs.len(), best.1, s.join(" "));
        }
    }
}
