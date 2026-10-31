mod arabic_roman;
mod ceiling_floor;
mod combinatorics;
mod decimal_text;
mod gcd_lcm;
mod hyperbolic;
mod mathematical;
mod matrix;
mod mmult;
mod multinomial;
mod product;
mod random;
mod sequence;
mod seriessum;
mod sum;
pub(crate) mod sum_prefix;
pub(crate) use sum::SumParts;
mod sumproduct;
mod trig;
pub(crate) use mathematical::{excel_atan2, percent_of};
pub(crate) use trig::{
    excel_fsin, excel_quarter_turn_remainder, excel_sin_cos, excel_tan, EXCEL_INNER_TRIG_LIMIT,
    EXCEL_TRIG_LIMIT,
};
