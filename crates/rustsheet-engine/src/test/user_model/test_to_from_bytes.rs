#![allow(clippy::unwrap_used)]

use crate::{test::util::new_empty_model, UserModel};

#[test]
fn basic() {
    let mut model1 = UserModel::from_model(new_empty_model());
    let width = model1.get_column_width(0, 3).unwrap() * 3.0;
    model1.set_columns_width(0, 3, 3, width).unwrap();
    model1.set_user_input(0, 1, 2, "Hello IronCalc!").unwrap();

    let model_bytes = model1.to_bytes();

    let model2 = UserModel::from_bytes(&model_bytes, "en").unwrap();

    assert_eq!(model2.get_column_width(0, 3), Ok(width));
    assert_eq!(
        model2.get_formatted_cell_value(0, 1, 2),
        Ok("Hello IronCalc!".to_string())
    );
}

#[test]
fn errors() {
    let model_bytes = "Early in the morning, late in the century, Cricklewood Broadway.".as_bytes();
    assert_eq!(
        &UserModel::from_bytes(model_bytes, "en").unwrap_err(),
        "Error parsing workbook: invalid packing"
    );
}

#[test]
fn language() {
    let mut model = UserModel::from_model(new_empty_model());
    model.set_user_input(0, 1, 1, "=NOW()").unwrap();
    model
        .set_user_input(0, 1, 2, "=SUM(1.234, 3.4, T1:T3, {1,2.4,3})")
        .unwrap();
    model.set_language("fr").unwrap();
    model.set_locale("fr").unwrap();
    let model_bytes = model.to_bytes();

    let model2 = UserModel::from_bytes(&model_bytes, "es").unwrap();
    // Check that the formula has been localized to Spanish
    assert_eq!(model2.get_cell_content(0, 1, 1), Ok("=AHORA()".to_string()));
    assert_eq!(
        model2.get_cell_content(0, 1, 2),
        Ok("=SUMA(1,234;3,4;T1:T3;{1;2,4;3})".to_string())
    );
}

#[test]
fn formulatext_english() {
    let mut model = UserModel::from_model(new_empty_model());
    model.set_user_input(0, 1, 1, "=SUM(1, 2, 3)").unwrap();
    model.set_user_input(0, 1, 2, "=FORMULATEXT(A1)").unwrap();

    model.set_language("de").unwrap();

    assert_eq!(
        model.get_formatted_cell_value(0, 1, 2),
        Ok("=SUM(1,2,3)".to_string())
    );
}

/// Bytes of `A1: ="ab"` saved before cached text became `Text` (T-1577, SPEC-0010 step 3): the
/// stored format changed, and an older save must be an `Err`, never a panic.
const BYTES_BEFORE_TEXT_TYPE: [&str; 16] = [
    "0000010241310000065368656574310101ff0400040200000000000000000000",
    "0000000000000000000000000000000000000000000000000000000000000000",
    "0000000000000000000000000000000000000000000000000000000000000000",
    "0000000000000000000000000000000000000000000000000000000000000000",
    "0000000000000000000000000000000000000000000104010504000400030261",
    "62060106010104226162220401000200000400040001040004010401040a0f04",
    "01040104010401010000000100000000040c0205496e74657204020002080100",
    "0000000000000104000400040004000101010101010104000400040004000400",
    "000000000000000001066e6f726d616c0400040000056d6f64656c0355544302",
    "656e000f49726f6e43616c63205368656574730731302e303030300d49726f6e",
    "43616c6320557365720d49726f6e43616c63205573657214323032322d31312d",
    "30385431313a31333a32385a14323032322d31312d30385431313a31333a3238",
    "5a000104000400042003045802064f6666696365072330303030303007234646",
    "4646464607233434353436410723453745364536072334343732433407234544",
    "3744333107234135413541350723464643303030072335423942443507233730",
    "414434370723303536334331072339353446373200",
];

#[test]
fn bytes_from_before_the_text_type_are_an_error() {
    let hex: String = BYTES_BEFORE_TEXT_TYPE.concat();
    let bytes: Vec<u8> = (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect();
    assert!(UserModel::from_bytes(&bytes, "en").is_err());
}

#[test]
fn cached_lone_surrogate_survives_bitcode() {
    use crate::types::{FormulaValue, SpillValue};
    use crate::Text;
    // No function produces a lone surrogate yet (T-1579), so encode the cached values directly.
    let lone = Text::from_utf16(vec![0xDE00, 0x62]);
    let formula: FormulaValue =
        bitcode::decode(&bitcode::encode(&FormulaValue::Text(lone.clone()))).unwrap();
    assert_eq!(formula, FormulaValue::Text(lone.clone()));
    let spill: SpillValue =
        bitcode::decode(&bitcode::encode(&SpillValue::Text(lone.clone()))).unwrap();
    assert_eq!(spill, SpillValue::Text(lone));
}
