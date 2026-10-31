//! The milestone exit's R1(c) measures (decision 0188, T-3427).

use super::*;

/// Decision 0188 R1(c): the probe row counts only the cells R1(c) names; the visibly-off total
/// stays beside it (T-3427).
#[test]
fn the_probe_measure_reads_the_r1c_count() {
    let mut r = m9_report(73_160, 0, false);
    let probe = r.probe_milestones.last_mut().unwrap();
    probe.visibly_off = 3;
    probe.phase1b = 2;
    probe.m14_owned = 1;
    let row = m9(&r);
    assert_eq!(
        (row.probe_visibly_off, row.probe_r1c_off, row.probe_phase1b),
        (Some(3), Some(0), Some(2))
    );
    assert_eq!(row.probes_met, Some(true));
    let mut md = String::new();
    write_markdown(&summarize(&r), &mut md);
    assert!(md.contains("| 0 (of 3 visibly off) [0] met |"), "{md}");
    r.probe_milestones.last_mut().unwrap().r1c_off = 1;
    assert_eq!(m9(&r).probes_met, Some(false));
}

/// The ironcalc-calc measure counts a visibly-off cell unless R1(c) classes it otherwise; an
/// unclassified cell counts, so nothing reads as met early (T-3427).
#[test]
fn the_ironcalc_calc_measure_reads_the_r1c_count() {
    // 1,005 cells less 5 deviations: the allowance is floor(0.0005 * 1,000) = 0.
    let mut r = m9_report(1_005, 3, false);
    let row = m9(&r);
    assert_eq!((row.calc_visibly_off, row.calc_r1c_off), (Some(3), Some(3)));
    assert_eq!(row.calc_met, Some(false));
    r.r1c = crate::r1c::R1cCells::from_classes([
        ("x!S!A1", crate::r1c::R1c::Phase1b),
        ("x!S!A2", crate::r1c::R1c::R3Pass),
        ("x!S!A3", crate::r1c::R1c::Phase1b),
    ]);
    let row = m9(&r);
    assert_eq!((row.calc_visibly_off, row.calc_r1c_off), (Some(3), Some(0)));
    assert_eq!(row.calc_met, Some(true));
    let mut md = String::new();
    write_markdown(&summarize(&r), &mut md);
    assert!(md.contains("0/0 of 1000 (of 3 visibly off)"), "{md}");
}
