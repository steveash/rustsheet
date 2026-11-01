mod bessel;
mod bit_operations;
mod complex;
mod convert;
mod misc;
mod number_basis;
pub(crate) mod transcendental;

/// The argument past which Excel's engineering functions treat `exp` as overflowing, a constant in
/// Excel's code below the true overflow of `exp` at `ln(DBL_MAX)` = 709.7827 (EXP(709.75) itself is
/// finite). BESSELI is `#NUM!` at every order once |x| exceeds it (`BESSELI(709.7,0)` answers,
/// `BESSELI(709.70001,0)` is `#NUM!`; probes/besseli-overflow-cut-finest), and IMSIN and IMCOS once
/// the imaginary part does, whatever the real part (`IMSIN("0.5+709.7i")` answers,
/// `IMSIN("0.5+709.700000000001i")` is `#NUM!`; probes-numeric/t3616-imsin-imcos-positive-cut-probe).
/// engineering.md "BESSELI overflow point" and "IMSIN and IMCOS positive overflow cut".
const ENGINEERING_EXP_CUT: f64 = 709.7;
