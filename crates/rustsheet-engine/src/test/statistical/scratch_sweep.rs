use statrs::distribution::{Beta, ChiSquared, ContinuousCDF, Gamma, LogNormal, Normal};
use std::sync::mpsc;
use std::time::Duration;

fn run(name: String, f: impl FnOnce() -> f64 + Send + std::panic::UnwindSafe + 'static) {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let r = std::panic::catch_unwind(f);
        let _ = tx.send(r.ok());
    });
    match rx.recv_timeout(Duration::from_secs(2)) {
        Ok(Some(v)) => println!("OK    {name} -> {v:e}"),
        Ok(None) => println!("PANIC {name}"),
        Err(_) => println!("HANG  {name}"),
    }
}

#[test]
fn scratch_sweep() {
    let ps = [1e-300, 1e-200, 1e-100, 1e-20, 1e-10, 1e-5, 0.5, 1.0 - 1e-5, 1.0 - 1e-10, 1.0 - 1e-16];
    for &p in &ps {
        run(format!("normal(0,1) p={p:e}"), move || Normal::new(0.0, 1.0).unwrap().inverse_cdf(p));
        run(format!("normal(10,0.001) p={p:e}"), move || Normal::new(10.0, 0.001).unwrap().inverse_cdf(p));
        run(format!("lognormal(0,1) p={p:e}"), move || LogNormal::new(0.0, 1.0).unwrap().inverse_cdf(p));
        for df in [1.0, 2.0, 10.0, 100.0, 1e4, 1e6, 1e10] {
            run(format!("chisq({df}) p={p:e}"), move || ChiSquared::new(df).unwrap().inverse_cdf(p));
        }
        for (a, r) in [(0.5, 1.0), (1.0, 1.0), (5.0, 1.0), (100.0, 1.0), (1e4, 1.0), (1e-3, 1.0), (5.0, 1e-3)] {
            run(format!("gamma({a},{r}) p={p:e}"), move || Gamma::new(a, r).unwrap().inverse_cdf(p));
        }
        for (a, b) in [(0.5, 0.5), (1.0, 1.0), (2.0, 5.0), (100.0, 100.0), (1e4, 1.0), (1e-3, 1e-3), (1e6, 1e6)] {
            run(format!("beta({a},{b}) p={p:e}"), move || Beta::new(a, b).unwrap().inverse_cdf(p));
        }
    }
}
