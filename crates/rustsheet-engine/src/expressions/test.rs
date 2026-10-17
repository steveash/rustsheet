use super::*;

#[test]
fn test_error_codes() {
    // The kinds before the newer ones (SPEC-0003) have an `error_index`.
    let errors = &Error::ALL[..12];
    for (i, error) in errors.iter().enumerate() {
        let s = format!("{}", error);
        let index = error_index(s.clone()).unwrap();
        assert_eq!(i as i32, index);
        let s2 = error_string(i as usize).unwrap();
        assert_eq!(s, s2);
    }
}
