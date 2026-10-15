use crate::calc_result::Range;
use crate::{
    calc_result::CalcResult,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    model::Model,
};

/// The largest `n` whose `n!` is finite in a double (170! is about 7.3e306).
const MAX_FACTORIAL_ARGUMENT: u64 = 170;

/// `log10(k)` for k = 1..=170, each the correctly rounded double. The platform `log10` is not
/// correctly rounded for every k (glibc's is one ulp off at 11, 40, 43, 53, 85, 90, 113, 119, 121
/// and 160), and [`multinomial_value`] carries its rounding into the result.
const LOG10_OF: [f64; 170] = [
    0.0,
    std::f64::consts::LOG10_2,
    0.47712125471966244,
    0.6020599913279624,
    0.6989700043360189,
    0.7781512503836436,
    0.8450980400142568,
    0.9030899869919435,
    0.9542425094393249,
    1.0,
    1.0413926851582251,
    1.0791812460476249,
    1.1139433523068367,
    1.146128035678238,
    1.1760912590556813,
    1.2041199826559248,
    1.2304489213782739,
    1.255272505103306,
    1.2787536009528289,
    1.3010299956639813,
    1.3222192947339193,
    1.3424226808222062,
    1.3617278360175928,
    1.380211241711606,
    1.3979400086720377,
    1.414973347970818,
    1.4313637641589874,
    1.4471580313422192,
    1.462397997898956,
    1.4771212547196624,
    1.4913616938342726,
    1.505149978319906,
    1.5185139398778875,
    1.5314789170422551,
    1.5440680443502757,
    1.5563025007672873,
    1.568201724066995,
    1.5797835966168101,
    1.591064607026499,
    1.6020599913279623,
    1.6127838567197355,
    1.6232492903979006,
    1.6334684555795864,
    1.6434526764861874,
    1.6532125137753437,
    1.662757831681574,
    1.6720978579357175,
    1.6812412373755872,
    1.6901960800285136,
    1.6989700043360187,
    1.7075701760979363,
    1.7160033436347992,
    1.724275869600789,
    1.7323937598229686,
    1.7403626894942439,
    1.7481880270062005,
    1.7558748556724915,
    1.7634279935629373,
    1.7708520116421442,
    1.7781512503836436,
    1.7853298350107671,
    1.792391689498254,
    1.7993405494535817,
    1.806179973983887,
    1.8129133566428555,
    1.8195439355418688,
    1.8260748027008264,
    1.8325089127062364,
    1.8388490907372552,
    1.845098040014257,
    1.8512583487190752,
    1.8573324964312685,
    1.863322860120456,
    1.8692317197309762,
    1.8750612633917,
    1.8808135922807914,
    1.8864907251724818,
    1.8920946026904804,
    1.8976270912904414,
    1.9030899869919435,
    1.9084850188786497,
    1.9138138523837167,
    1.919078092376074,
    1.9242792860618816,
    1.9294189257142926,
    1.9344984512435677,
    1.9395192526186185,
    1.9444826721501687,
    1.9493900066449128,
    1.954242509439325,
    1.9590413923210936,
    1.9637878273455553,
    1.968482948553935,
    1.9731278535996986,
    1.9777236052888478,
    1.9822712330395684,
    1.9867717342662448,
    1.9912260756924949,
    1.99563519459755,
    2.0,
    2.0043213737826426,
    2.0086001717619175,
    2.012837224705172,
    2.0170333392987803,
    2.0211892990699383,
    2.0253058652647704,
    2.0293837776852097,
    2.03342375548695,
    2.037426497940624,
    2.041392685158225,
    2.0453229787866576,
    2.0492180226701815,
    2.0530784434834195,
    2.0569048513364727,
    2.060697840353612,
    2.0644579892269186,
    2.0681858617461617,
    2.0718820073061255,
    2.0755469613925306,
    2.0791812460476247,
    2.0827853703164503,
    2.0863598306747484,
    2.089905111439398,
    2.093421685162235,
    2.0969100130080562,
    2.100370545117563,
    2.103803720955957,
    2.1072099696478683,
    2.110589710299249,
    2.113943352306837,
    2.1172712956557644,
    2.12057393120585,
    2.123851640967086,
    2.1271047983648077,
    2.130333768495006,
    2.1335389083702174,
    2.1367205671564067,
    2.1398790864012365,
    2.143014800254095,
    2.146128035678238,
    2.1492191126553797,
    2.1522883443830563,
    2.155336037465062,
    2.1583624920952498,
    2.161368002234975,
    2.164352855784437,
    2.167317334748176,
    2.1702617153949575,
    2.173186268412274,
    2.1760912590556813,
    2.1789769472931693,
    2.1818435879447726,
    2.184691430817599,
    2.187520720836463,
    2.1903316981702914,
    2.1931245983544616,
    2.1958996524092336,
    2.1986570869544226,
    2.2013971243204513,
    2.2041199826559246,
    2.2068258760318495,
    2.2095150145426308,
    2.2121876044039577,
    2.214843848047698,
    2.2174839442139063,
    2.220108088040055,
    2.2227164711475833,
    2.225309281725863,
    2.2278867046136734,
    2.230448921378274,
];

/// `log10(k)` for 1 <= k <= 170; the caller caps the sum at 170 first (`fn_multinomial`), so any
/// other k is never asked for.
fn log10_of(k: u64) -> f64 {
    usize::try_from(k)
        .ok()
        .and_then(|k| LOG10_OF.get(k.wrapping_sub(1)))
        .copied()
        .unwrap_or(0.0)
}

/// MULTINOMIAL of `values`, whose sum is at most 170, as Excel computes it. One double running
/// sum adds `log10(k)` for k = 1..=sum, then subtracts `log10(k)` for k = 1..=n for each value
/// from the last to the first, and the result is `exp(sum * ln 10)`. Its rounding is Excel's:
/// `MULTINOMIAL(2,3)` 9.999999999999998, `(170)` 1.0000000000001974, and `(20,150)` differs from
/// `(150,20)` (docs/excel-semantics/numbers.md "MULTINOMIAL is inexact"; fit on
/// probes/multinomial-algorithm-sweep, multinomial-large-arguments and math-argument-modes,
/// T-1190: 64 of 72 cells bit-exact, the rest one ulp off, T-1600). The terms come from a table so
/// that they do not depend on the platform `log10`; the last step is the platform `exp` (glibc's
/// gives the fitted values).
fn multinomial_value(values: &[u64]) -> f64 {
    let sum: u64 = values.iter().sum();
    let mut log = 0.0;
    for k in 1..=sum {
        log += log10_of(k);
    }
    for &n in values.iter().rev() {
        for k in 1..=n {
            log -= log10_of(k);
        }
    }
    (log * std::f64::consts::LN_10).exp()
}

/// Numeric text as its number: typed (`MULTINOMIAL("2",3)` is `MULTINOMIAL(2,3)`) and in a range
/// cell (4, "2", 6 reads as 4, 2, 6), as GCD and LCM read it (probes/math-argument-modes C9, C47,
/// C52; docs/excel-semantics/coercion.md "GCD / LCM / MULTINOMIAL range cells"). Other text stays
/// text, which is `#VALUE!`.
fn numeric_text_as_number(model: &Model, value: CalcResult) -> CalcResult {
    match value {
        CalcResult::String(text) => match model.cast_number(&text) {
            Some(number) => CalcResult::Number(number),
            None => CalcResult::String(text),
        },
        other => other,
    }
}

fn negative_error(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(
        Error::NUM,
        cell,
        "MULTINOMIAL requires non-negative values".to_string(),
    )
}

impl Model {
    // ── MULTINOMIAL ───────────────────────────────────────────────────────────

    /// `=MULTINOMIAL(number1, [number2], ...)`
    ///
    /// Returns `(n1 + n2 + ... + nk)! / (n1! * n2! * ... * nk!)`.
    /// All arguments must be non-negative integers (or numbers truncated to int).
    pub(crate) fn fn_multinomial(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }

        let mut values: Vec<u64> = Vec::new();
        // The type check over every argument wins over a negative number whatever the order
        // (gen-m6/2c1a6e93a1974f48 Calc!A35, docs/excel-semantics/coercion.md), so a negative
        // only sets this flag and `#NUM!` is returned after the last argument.
        let negative_seen = std::cell::Cell::new(false);

        for (index, arg) in args.iter().enumerate() {
            let evaluated = self.evaluate_node_in_context(arg, cell);
            let evaluated = numeric_text_as_number(self, evaluated);
            match evaluated {
                CalcResult::Number(v) => {
                    if v < 0.0 {
                        negative_seen.set(true);
                    } else {
                        values.push(v.trunc() as u64);
                    }
                }
                CalcResult::Range { left, right } => {
                    // A cell past the used area is empty, which counts as 0 here
                    // and changes neither the sum nor the product of factorials.
                    if let Err(error) = self.for_each_cell_or_error_in_areas(
                        &[Range { left, right }],
                        cell,
                        |model, value| match numeric_text_as_number(model, value) {
                            error @ CalcResult::Error { .. } => Err(error),
                            CalcResult::Number(v) if v < 0.0 => {
                                negative_seen.set(true);
                                Ok(())
                            }
                            CalcResult::Number(v) => {
                                values.push(v.trunc() as u64);
                                Ok(())
                            }
                            CalcResult::EmptyCell | CalcResult::EmptyArg => {
                                values.push(0);
                                Ok(())
                            }
                            _ => Err(CalcResult::new_error(
                                Error::VALUE,
                                cell,
                                "MULTINOMIAL requires numeric values".to_string(),
                            )),
                        },
                    ) {
                        return error;
                    }
                }
                CalcResult::Array(arr) => {
                    for row in arr {
                        for node in row {
                            match node {
                                ArrayNode::Number(v) => {
                                    if v < 0.0 {
                                        negative_seen.set(true);
                                    } else {
                                        values.push(v.trunc() as u64);
                                    }
                                }
                                ArrayNode::Empty => values.push(0),
                                ArrayNode::Error(e) => {
                                    return CalcResult::Error {
                                        error: e,
                                        origin: cell,
                                        message: String::new(),
                                    }
                                }
                                _ => {
                                    return CalcResult::new_error(
                                        Error::VALUE,
                                        cell,
                                        "MULTINOMIAL requires numeric values".to_string(),
                                    )
                                }
                            }
                        }
                    }
                }
                // Excel: an omitted first argument is `#N/A`, a later one is skipped
                // (probes/gcd-lcm-multinomial-negative-precedence E51:E71).
                CalcResult::EmptyArg if index == 0 => {
                    return CalcResult::new_error(
                        Error::NA,
                        cell,
                        "MULTINOMIAL: empty first argument".to_string(),
                    )
                }
                CalcResult::EmptyArg => {}
                CalcResult::EmptyCell => values.push(0),
                err @ CalcResult::Error { .. } => return err,
                _ => {
                    return CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "MULTINOMIAL requires numeric values".to_string(),
                    )
                }
            }
        }

        if negative_seen.get() {
            return negative_error(cell);
        }
        // Excel: `#NUM!` once the arguments add up to more than 170, where `sum!` overflows a
        // double: `MULTINOMIAL(171)`, `(170,1)`, `(86,86)` and `(1E+15)` are `#NUM!`, while
        // `(170)`, `(169,1)` and `(85,85)` have values (probes/multinomial-large-arguments,
        // T-1197). Checking first also bounds the log-factorial loops below to 170 steps
        // (CLAUDE.md directive 5); the sum saturates, so huge arguments cannot overflow it.
        let sum = values.iter().fold(0u64, |acc, &v| acc.saturating_add(v));
        if sum > MAX_FACTORIAL_ARGUMENT {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "MULTINOMIAL: result overflow".to_string(),
            );
        }
        let result = multinomial_value(&values);
        if !result.is_finite() {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "MULTINOMIAL: result overflow".to_string(),
            );
        }
        CalcResult::Number(result)
    }
}
