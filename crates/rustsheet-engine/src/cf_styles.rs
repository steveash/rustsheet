//! Reading a cell's style with its conditional-formatting overlay. The conditional formats are
//! evaluated on the first read after a load, an `evaluate()` or `set_theme` (SPEC-0021, T-0468),
//! not by those.

use crate::cf_types::{CfCellResult, CfDataBar, CfIcon, CfRating, ExtendedStyle};
use crate::Model;

impl Model {
    /// Returns the extended cell style for (`sheet`, `row`, `column`), including any
    /// conditional-formatting overlay. The conditional formats are evaluated here, on the first
    /// read after a load, an `evaluate()` or `set_theme`, not by those (SPEC-0021).
    pub fn get_extended_style_for_cell(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
    ) -> Result<ExtendedStyle, String> {
        if self.cf_stale {
            self.evaluate_conditional_formatting();
        }
        let base = self.get_style_for_cell(sheet, row, column)?;
        let extended = match self.cf_cache.get(&(sheet, row, column)) {
            Some(c) => c,
            None => {
                return Ok(ExtendedStyle {
                    style: base,
                    icon: None,
                    data_bar: None,
                    rating: None,
                })
            }
        };
        let mut style = base;
        let mut icon = None;
        let mut data_bar = None;
        let mut rating = None;
        // `extended` contains the cached conditional-formatting results for this cell in the
        // order produced by the evaluator. Dxf results accumulate as we iterate; for exclusive
        // types (ColorScale, DataBar, Icon, Rating), the last assignment in this evaluated
        // result order wins.
        for cf_result in extended {
            match cf_result {
                CfCellResult::Dxf(dxf_id) => {
                    if let Some(dxf) = self.workbook.styles.dxfs.get(*dxf_id as usize) {
                        style = dxf.apply_to(&style);
                    }
                }
                CfCellResult::ColorScale(color) => {
                    style.fill.color = color.clone();
                }
                CfCellResult::DataBar {
                    positive_color,
                    negative_color,
                    is_gradient,
                    value,
                    axis_position,
                    show_value,
                } => {
                    data_bar = Some(CfDataBar {
                        positive_color: positive_color.clone(),
                        negative_color: negative_color.clone(),
                        is_gradient: *is_gradient,
                        value: *value,
                        axis_position: *axis_position,
                        show_value: *show_value,
                    });
                }
                CfCellResult::Icon {
                    icon: cf_icon,
                    color,
                    show_value,
                } => {
                    icon = Some(CfIcon {
                        icon: cf_icon.clone(),
                        color: color.clone(),
                        show_value: *show_value,
                    });
                }
                CfCellResult::Rating {
                    icon: cf_icon,
                    count,
                    max,
                    color,
                    show_value,
                } => {
                    rating = Some(CfRating {
                        icon: cf_icon.clone(),
                        count: *count,
                        max: *max,
                        color: color.clone(),
                        show_value: *show_value,
                    });
                }
            }
        }
        Ok(ExtendedStyle {
            style,
            icon,
            data_bar,
            rating,
        })
    }

    /// Counts an evaluation of the conditional formats, for the tests of SPEC-0021.
    #[cfg(test)]
    pub(crate) fn count_cf_run(&mut self) {
        self.cf_runs += 1;
    }
}
