#![allow(clippy::unwrap_used)]

//! IFS and SWITCH return the chosen value argument as is, so a range value spills
//! (T-0272; probes/ifs-switch-array-values).

use crate::test::util::new_empty_model;

#[test]
fn range_values_spill() {
    let mut model = new_empty_model();
    for (cell, value) in [("A1", "1"), ("B1", "2"), ("A2", "3"), ("B2", "4")] {
        model._set(cell, value);
    }
    model._set("D1", "=IFS(TRUE,A1:B2)");
    model._set("D4", "=IFS(FALSE,1,TRUE,A1:B2)");
    model._set("G1", "=SWITCH(1,1,A1:B2)");
    model._set("G4", "=SWITCH(2,1,5,A1:B2)");
    model._set("J1", "=SWITCH(9,1,5,A1:B2)");
    model.evaluate();

    // Each formula spills the 2x2 range 1,2;3,4 (oracle D1:E2, D4:E5, G1:H2, G4:H5, J1:K2)
    for (column, row) in [("D", 1), ("D", 4), ("G", 1), ("G", 4), ("J", 1)] {
        let next = char::from(column.as_bytes()[0] + 1);
        assert_eq!(model._get_text(&format!("{column}{row}")), "1");
        assert_eq!(model._get_text(&format!("{next}{row}")), "2");
        assert_eq!(model._get_text(&format!("{column}{}", row + 1)), "3");
        assert_eq!(model._get_text(&format!("{next}{}", row + 1)), "4");
    }
}
