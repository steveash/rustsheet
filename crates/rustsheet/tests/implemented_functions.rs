//! `implemented_functions` lists the engine's functions by en-US name, sorted (T-0020).

#[test]
fn lists_en_us_names_sorted_without_duplicates() {
    let names = rustsheet::implemented_functions();
    assert!(names.iter().any(|n| n == "SUM"));
    assert!(names.iter().any(|n| n == "ERROR.TYPE"));
    assert!(names.windows(2).all(|w| w[0] < w[1]), "sorted and unique");
}
