// BAHTTEXT

use super::text_arg::broadcast_text;
use crate::{
    calc_result::CalcResult,
    expressions::{
        parser::{ArrayNode, Node},
        types::CellReferenceIndex,
    },
    model::Model,
    number_format::significant_15,
};

const DIGITS: [&str; 10] = [
    "ศูนย์",
    "หนึ่ง",
    "สอง",
    "สาม",
    "สี่",
    "ห้า",
    "หก",
    "เจ็ด",
    "แปด",
    "เก้า",
];
/// Words for the tens, hundreds, thousands, ten-thousands and hundred-thousands places.
const PLACES: [&str; 6] = ["", "สิบ", "ร้อย", "พัน", "หมื่น", "แสน"];

/// Appends the reading of up to six digits (most significant first). A 1 in the units place
/// reads เอ็ด whenever anything was already written, 2 in the tens place reads ยี่
/// (probes/text-bahttext-wide, -wide2; docs/excel-semantics/text.md#bahttext).
fn push_chunk(out: &mut String, digits: &[u8]) {
    for (i, &d) in digits.iter().enumerate() {
        let place = digits.len() - 1 - i;
        match (d, place) {
            (0, _) => {}
            (1, 0) if !out.is_empty() => out.push_str("เอ็ด"),
            (1, 1) => out.push_str("สิบ"),
            (2, 1) => out.push_str("ยี่สิบ"),
            _ => {
                out.push_str(DIGITS[usize::from(d)]);
                out.push_str(PLACES[place]);
            }
        }
    }
}

/// The reading of a digit string (most significant first, no leading zero unless it is "0"):
/// groups of six digits joined by ล้าน, even over an all-zero group (1e12 is หนึ่งล้านล้าน).
fn push_integer(out: &mut String, digits: &[u8]) {
    let first = digits.len() % 6;
    let (head, tail) = digits.split_at(first);
    let mut groups: Vec<&[u8]> = Vec::new();
    if !head.is_empty() {
        groups.push(head);
    }
    groups.extend(tail.chunks(6));
    for (i, group) in groups.iter().enumerate() {
        if i > 0 {
            out.push_str("ล้าน");
        }
        push_chunk(out, group);
    }
}

/// BAHTTEXT of a finite number: the value is cut to 15 significant digits, then rounded half
/// away from zero to two decimals (satang); the sign prints as ลบ even when the value rounds to
/// zero (probes/text-bahttext-wide, -wide2).
fn baht_text(value: f64) -> String {
    let negative = value < 0.0;
    // (integer digits, fraction digits) of |value| cut to 15 significant digits.
    let (mut baht, mut frac): (Vec<u8>, Vec<u8>) = if value == 0.0 {
        (vec![0], Vec::new())
    } else {
        let (mantissa, exponent) = significant_15(value.abs(), true);
        let digits: Vec<u8> = format!("{mantissa:015}")
            .bytes()
            .map(|b| b - b'0')
            .collect();
        if exponent >= 14 {
            let mut int = digits;
            int.resize(usize::try_from(exponent).unwrap_or(0) + 1, 0);
            (int, Vec::new())
        } else if exponent >= 0 {
            let split = usize::try_from(exponent).unwrap_or(0) + 1;
            (digits[..split].to_vec(), digits[split..].to_vec())
        } else {
            let mut frac = vec![0; usize::try_from(-exponent - 1).unwrap_or(0)];
            frac.extend(digits);
            (vec![0], frac)
        }
    };
    frac.resize(3, 0);
    let mut satang = u32::from(frac[0]) * 10 + u32::from(frac[1]);
    if frac[2] >= 5 {
        satang += 1;
    }
    if satang == 100 {
        satang = 0;
        // Carry into the baht digits.
        let mut carry = true;
        for d in baht.iter_mut().rev() {
            if *d == 9 {
                *d = 0;
            } else {
                *d += 1;
                carry = false;
                break;
            }
        }
        if carry {
            baht.insert(0, 1);
        }
    }
    while baht.len() > 1 && baht[0] == 0 {
        baht.remove(0);
    }

    let mut out = String::new();
    let baht_is_zero = baht == [0];
    if !baht_is_zero || satang == 0 {
        // Satang alone has no baht part (0.5 is "ห้าสิบสตางค์").
        if baht_is_zero {
            out.push_str(DIGITS[0]);
        } else {
            push_integer(&mut out, &baht);
        }
        out.push_str("บาท");
    }
    if satang == 0 {
        out.push_str("ถ้วน");
    } else {
        // Satang are read on their own, so a lone 1 is หนึ่ง (5.01 reads ห้าบาทหนึ่งสตางค์).
        let mut words = String::new();
        push_chunk(&mut words, &[(satang / 10) as u8, (satang % 10) as u8]);
        out.push_str(&words);
        out.push_str("สตางค์");
    }
    if negative {
        out.insert_str(0, "ลบ");
    }
    out
}

impl Model {
    /// BAHTTEXT(number) — Thai text for a baht amount. Lifts over a range or array argument
    /// (probes/text-bahttext-form4, -form5) and coerces its argument like a number: text
    /// "5" and TRUE work, "abc" and "" are `#VALUE!`, an empty cell is 0.
    pub(crate) fn fn_bahttext(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let number = match self.text_arg(&args[0], cell) {
            Ok(n) => n,
            Err(e) => return e,
        };
        broadcast_text(cell, &[&number], |i, j| {
            let v = self.array_node_to_number(number.elem(i, j))?;
            Ok(ArrayNode::String(baht_text(v).into()))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::baht_text;

    #[test]
    fn reads_like_excel() {
        assert_eq!(baht_text(5.0), "ห้าบาทถ้วน");
        assert_eq!(baht_text(0.0), "ศูนย์บาทถ้วน");
        assert_eq!(baht_text(1_000_001.0), "หนึ่งล้านเอ็ดบาทถ้วน");
        assert_eq!(baht_text(1e12), "หนึ่งล้านล้านบาทถ้วน");
        assert_eq!(baht_text(0.995), "หนึ่งบาทถ้วน");
        assert_eq!(baht_text(-0.001), "ลบศูนย์บาทถ้วน");
        assert_eq!(baht_text(-5.5), "ลบห้าบาทห้าสิบสตางค์");
        assert_eq!(baht_text(0.21), "ยี่สิบเอ็ดสตางค์");
        assert_eq!(baht_text(1.05), "หนึ่งบาทห้าสตางค์");
    }
}
