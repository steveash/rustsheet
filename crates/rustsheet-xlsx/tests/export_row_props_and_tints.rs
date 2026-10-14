#![allow(clippy::unwrap_used, reason = "tests")]

use rustsheet_engine::types::Color;
use rustsheet_engine::Model;
use rustsheet_xlsx::export::save_to_xlsx;
use rustsheet_xlsx::import::load_from_xlsx;

const TINT: f64 = 0.300_000_000_000_000_04;

fn round_trip(model: &Model, path: &str) -> Model {
    save_to_xlsx(model, path).unwrap();
    let loaded = load_from_xlsx(path, "en", "UTC", "en").unwrap();
    std::fs::remove_file(path).unwrap();
    loaded
}

/// Ported-From: ironcalc@306daf40924b72a1b98cbda45500e9ba582777b4 — a theme tint is written in
/// shortest round-trip form, so it survives export and import bit for bit.
#[test]
fn theme_tint_survives_a_round_trip() {
    let mut model = Model::new_empty("m", "en", "UTC", "en").unwrap();
    model.set_user_input(0, 1, 1, "x".to_string()).unwrap();
    let mut style = model.get_style_for_cell(0, 1, 1).unwrap();
    style.fill.color = Color::Theme(4, TINT);
    model.set_cell_style(0, 1, 1, &style).unwrap();
    let loaded = round_trip(&model, "temp_theme_tint_round_trip.xlsx");
    let fill = loaded.get_style_for_cell(0, 1, 1).unwrap().fill.color;
    assert_eq!(fill, Color::Theme(4, TINT));
}

/// Ported-From: ironcalc@f428b2ebbc57fcf292f3e67027b24e1c866cb7c4 — a row without cells keeps
/// its custom height or hidden flag on export.
#[test]
fn empty_rows_keep_height_and_hidden_flag() {
    let mut model = Model::new_empty("m", "en", "UTC", "en").unwrap();
    model.set_user_input(0, 1, 1, "x".to_string()).unwrap();
    model.set_user_input(0, 9, 1, "y".to_string()).unwrap();
    model.set_row_height(0, 5, 40.0).unwrap();
    model.set_row_hidden(0, 7, true).unwrap();
    let loaded = round_trip(&model, "temp_empty_row_props_round_trip.xlsx");
    assert!((loaded.get_row_height(0, 5).unwrap() - 40.0).abs() < 1e-9);
    assert!(loaded.is_row_hidden(0, 7).unwrap());
    assert!(!loaded.is_row_hidden(0, 5).unwrap());
}
