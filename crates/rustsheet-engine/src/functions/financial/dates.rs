//! Date arguments of the financial functions: Excel reads a 1904 serial one day late
//! (`DateSystem::financial_1900_serial`; docs/excel-semantics/dates.md "The 1904 date system:
//! financial day counts read every date one day late").

use chrono::Datelike;

use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, types::CellReferenceIndex},
    formatter::dates::from_excel_date,
    model::Model,
};

/// Whether `end_date` is at most a year after `start_date` (1900 serials), the TBILL* limit.
pub(super) fn is_less_than_one_year(start_date: i64, end_date: i64) -> Result<bool, String> {
    let end = from_excel_date(end_date)?;
    let start = from_excel_date(start_date)?;
    if end_date - start_date < 365 {
        return Ok(true);
    }
    let end_year = end.year();
    let start_year = start.year();
    if end_year == start_year {
        return Ok(true);
    }
    if end_year != start_year + 1 {
        return Ok(false);
    }
    let start_month = start.month();
    let end_month = end.month();
    if end_month < start_month {
        return Ok(true);
    }
    if end_month > start_month {
        return Ok(false);
    }
    // we are one year later same month
    let start_day = start.day();
    let end_day = end.day();
    Ok(end_day <= start_day)
}

impl Model {
    /// The floored 1900 serial a financial function's day counts read for the date argument
    /// `serial`: a day late in a 1904 workbook (`DateSystem::financial_1900_serial`).
    pub(super) fn financial_date(&self, serial: f64) -> i64 {
        self.workbook
            .settings
            .date_system
            .financial_1900_serial(serial)
            .floor() as i64
    }

    /// A date argument of a financial function, as `financial_date` reads it. Booleans are
    /// rejected; ACCRINT, ACCRINTM and AMORLINC accept them, so they call `get_number` and
    /// `financial_date` themselves.
    pub(super) fn get_financial_date(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<i64, CalcResult> {
        let serial = self.get_number_no_bools(node, cell)?;
        Ok(self.financial_date(serial))
    }

    /// The first two arguments, settlement and maturity, as `get_financial_date` reads them.
    pub(super) fn get_settlement_and_maturity(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> Result<(i64, i64), CalcResult> {
        Ok((
            self.get_financial_date(&args[0], cell)?,
            self.get_financial_date(&args[1], cell)?,
        ))
    }

    /// A date a financial function computed in the `financial_date` space (COUPNCD, COUPPCD), as
    /// this workbook's serial: a day early in a 1904 workbook.
    pub(super) fn financial_date_result(&self, serial: i64) -> CalcResult {
        let date_system = self.workbook.settings.date_system;
        CalcResult::Number(date_system.serial_from_financial_1900_serial(serial) as f64)
    }
}
