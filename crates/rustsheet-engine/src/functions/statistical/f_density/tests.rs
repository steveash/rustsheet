use super::*;
use crate::test::util::within_fifteen_digits;

/// (x, df1, df2, Excel's F.DIST(x,df1,df2,FALSE), M-final's value) of
/// `test/fixtures/f_density_final_hold.fixture`.
fn final_hold_rows() -> Vec<[f64; 5]> {
    let bits = |field: &str| f64::from_bits(u64::from_str_radix(field, 16).unwrap());
    include_str!("../../../test/fixtures/f_density_final_hold.fixture")
        .lines()
        .filter(|line| !line.starts_with('#') && !line.is_empty())
        .map(|line| {
            let fields: Vec<f64> = line.split_whitespace().map(bits).collect();
            fields.try_into().unwrap()
        })
        .collect()
}

// T-3034 acceptance: the port is M-final bit for bit on the untouched hold-out's 1,549 positive
// value cells, so it scores as RESULTS.md section 2 reports (126 bit-exact of 1,584 with the 35
// rule cells, which `fisher.rs` handles; 722 within 15 digits with them).
#[test]
fn final_hold_is_model_m_final() {
    let rows = final_hold_rows();
    assert_eq!(rows.len(), 1549);
    let (mut exact, mut fifteen) = (0, 0);
    for [x, d1, d2, excel, model] in rows {
        let value = excel_f_density(x, d1.trunc(), d2.trunc());
        assert_eq!(value, model, "F.DIST({x:e},{d1},{d2},FALSE)");
        exact += usize::from(value == excel);
        fifteen += usize::from(within_fifteen_digits(excel, value));
    }
    assert_eq!(exact, 91);
    assert_eq!(fifteen, 687);
}

// Bit-exact hold-out cells on both sides, and one pinned: side 1 with r on the log1p series
// (F.DIST(0.108...,3.2,212,FALSE): 3 x 0.108 <= 212).
#[test]
fn pinned_excel_cells() {
    let rows = final_hold_rows();
    let mut sides = [false; 2];
    for [x, d1, d2, excel, _] in rows {
        let (m, n) = (d1.trunc(), d2.trunc());
        if excel_f_density(x, m, n) == excel {
            sides[usize::from(m * x > n)] = true;
        }
    }
    assert_eq!(sides, [true, true]);
    assert_eq!(
        excel_f_density(f64::from_bits(0x3fbb_bc8c_2943_8d1b), 3.0, 212.0),
        f64::from_bits(0x3fe2_95ac_216c_6d67)
    );
}

// The two floors: an exponent below -800 is 0 (F.DIST(1E300,1,1,FALSE), t1864-f-pdf-edges A11),
// and a density below 2^-1022 is 0 although EXP(E) is a normal double (F.DIST(1E308,1,3,FALSE),
// probes/stat-overflow-arguments E11).
#[test]
fn floors_give_zero() {
    assert_eq!(excel_f_density(1e300, 1.0, 1.0), 0.0);
    assert_eq!(excel_f_density(1e308, 1.0, 3.0), 0.0);
    assert!(excel_f_density(1e200, 1.0, 1.0) > 0.0);
}

// Binet's function: the table and the series agree across the switch at v = 15 (both against
// mpmath's mu(15.5) = 0.005375599032926835 - 3.78e-19, the table's next entry).
#[test]
fn binet_series_continues_the_table() {
    let (hi, lo) = binet(15.5);
    assert!((hi - 0.005_375_599_032_926_835).abs() + (lo + 3.784e-19).abs() < 1e-21);
    assert_eq!(binet(0.5), BINET_TABLE[0]);
    assert_eq!(binet(15.0), BINET_TABLE[29]);
}

// The stirlerr error table, its fitted ends and its linear extrapolation past 28.
#[test]
fn stirling_error_table() {
    assert_eq!(stirling_error(0.5), STIRLING_ERROR_HALF);
    assert_eq!(stirling_error(1.0), STIRLING_ERROR_ONE);
    assert_eq!(stirling_error(1.5), -5.144);
    assert_eq!(stirling_error(28.0), -121.421);
    assert_eq!(
        stirling_error(28.5),
        STIRLING_ERROR_SLOPE * 28.5 + STIRLING_ERROR_INTERCEPT
    );
}
