use std::collections::HashMap;

use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    functions::Function,
    model::Model,
};

enum Temperature {
    Kelvin,
    Celsius,
    Rankine,
    Reaumur,
    Fahrenheit,
}

// Excel: every pair has its own last-digit form; no single pivot reproduces them (probes/convert-
// targets and probes/convert-temperature-sweep, excel-graph; docs/excel-semantics/engineering.md
// "CONVERT temperature"). Rankine and Fahrenheit are linked by `459.67`, the Rankine/Kelvin ratio
// is `1.8` (a multiply or divide, never `9/5`), and F to Reaumur is `(v - 32) * 0.8 / 1.8`.
fn convert_temperature(
    value: f64,
    from_temperature: Temperature,
    to_temperature: Temperature,
) -> f64 {
    use Temperature::{Celsius, Fahrenheit, Kelvin, Rankine, Reaumur};
    match (from_temperature, to_temperature) {
        (Celsius, Celsius)
        | (Kelvin, Kelvin)
        | (Rankine, Rankine)
        | (Reaumur, Reaumur)
        | (Fahrenheit, Fahrenheit) => value,
        (Celsius, Fahrenheit) => value * 1.8 + 32.0,
        (Celsius, Kelvin) => value + 273.15,
        (Celsius, Rankine) => (value + 273.15) * 1.8,
        (Celsius, Reaumur) => value * 0.8,
        (Fahrenheit, Celsius) => (value - 32.0) / 1.8,
        (Fahrenheit, Kelvin) => (value - 32.0) / 1.8 + 273.15,
        (Fahrenheit, Rankine) => value + 459.67,
        (Fahrenheit, Reaumur) => (value - 32.0) * 0.8 / 1.8,
        (Kelvin, Celsius) => value - 273.15,
        (Kelvin, Fahrenheit) => (value - 273.15) * 1.8 + 32.0,
        (Kelvin, Rankine) => value * 1.8,
        (Kelvin, Reaumur) => (value - 273.15) * 0.8,
        (Rankine, Celsius) => value / 1.8 - 273.15,
        (Rankine, Fahrenheit) => value - 459.67,
        (Rankine, Kelvin) => value / 1.8,
        (Rankine, Reaumur) => (value / 1.8 - 273.15) * 0.8,
        (Reaumur, Celsius) => value * 1.25,
        (Reaumur, Fahrenheit) => value * 2.25 + 32.0,
        (Reaumur, Kelvin) => value * 1.25 + 273.15,
        (Reaumur, Rankine) => (value * 1.25 + 273.15) * 1.8,
    }
}

impl Model {
    /// `CONVERT(number, from_unit, to_unit)`: every argument lifts over an array and refuses a
    /// multi-cell range (`ArgKind::LiftArrays`, SPEC-0011; probes/eng-argument-modes B14:E14,
    /// probes/scalar-lift-open-a Y8-Y10).
    pub(crate) fn fn_convert(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Convert, args, cell, &mut |model, values| {
            model.convert_of(values, cell)
        })
    }

    /// CONVERT of one element: the number, then the two units, read in order.
    fn convert_of(&self, values: &[CalcResult], cell: CellReferenceIndex) -> CalcResult {
        let [value, from_unit, to_unit] = values else {
            return CalcResult::new_args_number_error(cell);
        };
        let value = match self.cast_number_ref(value, cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let from_unit = match Self::cast_string_ref(from_unit, cell) {
            Ok(s) => s,
            Err(error) => return error,
        };
        let to_unit = match Self::cast_string_ref(to_unit, cell) {
            Ok(s) => s,
            Err(error) => return error,
        };
        let prefix = HashMap::from([
            ("Y", 1E+24),
            ("Z", 1E+21),
            ("E", 1000000000000000000.0),
            ("P", 1000000000000000.0),
            ("T", 1000000000000.0),
            ("G", 1000000000.0),
            ("M", 1000000.0),
            ("k", 1000.0),
            ("h", 100.0),
            ("da", 10.0),
            ("e", 10.0),
            ("d", 0.1),
            ("c", 0.01),
            ("m", 0.001),
            ("u", 0.000001),
            ("n", 0.000000001),
            ("p", 1E-12),
            ("f", 1E-15),
            ("a", 1E-18),
            ("z", 1E-21),
            ("y", 1E-24),
            ("Yi", 2.0_f64.powf(80.0)),
            ("Ei", 2.0_f64.powf(70.0)),
            ("Yi", 2.0_f64.powf(80.0)),
            ("Zi", 2.0_f64.powf(70.0)),
            ("Ei", 2.0_f64.powf(60.0)),
            ("Pi", 2.0_f64.powf(50.0)),
            ("Ti", 2.0_f64.powf(40.0)),
            ("Gi", 2.0_f64.powf(30.0)),
            ("Mi", 2.0_f64.powf(20.0)),
            ("ki", 2.0_f64.powf(10.0)),
        ]);

        let mut units = HashMap::new();

        let weight = HashMap::from([
            ("g", 1.0),
            ("sg", 14593.902937206363),
            ("lbm", 453.59237),
            ("u", 1.660538782E-24),
            ("ozm", 28.349523125),
            ("grain", 0.06479891),
            ("cwt", 45359.237),
            ("shweight", 45359.237),
            ("uk_cwt", 50802.34544),
            ("lcwt", 50802.34544),
            ("stone", 6350.29318),
            ("ton", 907184.74),
            ("brton", 1016046.9088), // g-sheets has a different value for this
            ("LTON", 1016046.9088),
            ("uk_ton", 1016046.9088),
        ]);

        units.insert("weight", weight);

        let distance = HashMap::from([
            ("m", 1.0),
            ("mi", 1609.344),
            ("Nmi", 1852.0),
            ("in", 0.0254),
            ("ft", 0.3048),
            ("yd", 0.9144),
            ("ang", 0.0000000001),
            ("ell", 1.143),
            ("ly", 9460730472580800.0),
            ("parsec", 3.0856775812815532e+16),
            ("pc", 3.0856775812815532e+16),
            ("Picapt", 0.0003527777777777778),
            ("Pica", 0.0003527777777777778),
            ("pica", 0.004233333333333334),
            ("survey_mi", 1609.3472186944373),
        ]);
        units.insert("distance", distance);
        let time = HashMap::from([
            ("yr", 31557600.0),
            ("day", 86400.0),
            ("d", 86400.0),
            ("hr", 3600.0),
            ("mn", 60.0),
            ("min", 60.0),
            ("sec", 1.0),
            ("s", 1.0),
        ]);

        units.insert("time", time);

        let pressure = HashMap::from([
            ("Pa", 1.0),
            ("p", 1.0),
            ("atm", 101325.0),
            ("at", 101325.0),
            ("mmHg", 133.322),
            ("psi", 6894.757293168362),
            ("Torr", 133.32236842105263),
        ]);
        units.insert("pressure", pressure);
        // Force factors are in dynes (N = 1e5): Excel's internal base, the only one that reproduces
        // every cached cell (docs/excel-semantics/engineering.md "CONVERT constants and evaluation
        // order"). Every factor here and below is the double fitted to CONVERT.xlsx (T-0922).
        let force = HashMap::from([
            ("N", 100000.0),
            ("dyn", 1.0),
            ("dy", 1.0),
            ("lbf", 444822.16152605),
            ("pond", 980.665),
        ]);
        units.insert("force", force);

        // Energy factors are in ergs (J = 1e7), Excel's internal base (see `force`).
        let energy = HashMap::from([
            ("J", 10000000.0),
            ("e", 1.0),
            ("c", 41840000.0),
            ("cal", 41868000.0),
            ("eV", 1.602176487e-12),
            ("ev", 1.602176487e-12),
            ("HPh", 26845195376961.73),
            ("hh", 26845195376961.73),
            ("Wh", 36000000000.0),
            ("wh", 36000000000.0),
            ("flb", 13558179.483314004),
            ("BTU", 10550558526.2),
            ("btu", 10550558526.2),
        ]);
        units.insert("energy", energy);

        let power = HashMap::from([
            ("HP", 745.6998715822702),
            ("h", 745.6998715822702),
            ("PS", 735.49875),
            ("W", 1.0),
            ("w", 1.0),
        ]);
        units.insert("power", power);

        let magnetism = HashMap::from([("T", 1.0), ("ga", 0.0001)]);
        units.insert("magnetism", magnetism);

        // Volume factors are in litres (m3 = 1000), Excel's internal base (see `force`).
        let volume = HashMap::from([
            ("tsp", 0.00492892159375),
            ("tspm", 0.005),
            ("tbs", 0.01478676478125),
            ("oz", 0.0295735295625),
            ("cup", 0.2365882365),
            ("pt", 0.473176473),
            ("us_pt", 0.473176473),
            ("uk_pt", 0.56826125),
            ("qt", 0.946352946),
            ("uk_qt", 1.1365225),
            ("gal", 3.785411784),
            ("uk_gal", 4.54609),
            ("l", 1.0),
            ("L", 1.0),
            ("lt", 1.0),
            ("ang3", 1e-27),
            ("ang^3", 1e-27),
            ("barrel", 158.987294928),
            ("bushel", 35.23907016688),
            ("ft3", 28.316846592),
            ("ft^3", 28.316846592),
            ("in3", 0.016387064),
            ("in^3", 0.016387064),
            ("ly3", 8.467866646237152e+50),
            ("ly^3", 8.467866646237152e+50),
            ("m3", 1000.0),
            ("m^3", 1000.0),
            ("mi3", 4168181825440.5796),
            ("mi^3", 4168181825440.5796),
            ("yd3", 764.554857984),
            ("yd^3", 764.554857984),
            ("Nmi3", 6352182208000.0),
            ("Nmi^3", 6352182208000.0),
            ("Picapt3", 4.39039566186557e-08),
            ("Picapt^3", 4.39039566186557e-08),
            ("Pica3", 4.39039566186557e-08),
            ("Pica^3", 4.39039566186557e-08),
            ("GRT", 2831.6846592),
            ("regton", 2831.6846592),
            ("MTON", 1132.67386368),
        ]);
        units.insert("volume", volume);

        // Area factors are in square angstroms (m2 = 1e20), Excel's internal base (see `force`).
        let area = HashMap::from([
            ("uk_acre", 4.0468564224e+23),
            ("us_acre", 4.046872609874252e+23),
            ("ang2", 1.0),
            ("ang^2", 1.0),
            ("ar", 1e+22),
            ("ft2", 9.290304e+18),
            ("ft^2", 9.290304e+18),
            ("ha", 1e+24),
            ("in2", 6.4516e+16),
            ("in^2", 6.4516e+16),
            ("ly2", 8.950542107481889e+51),
            ("ly^2", 8.950542107481889e+51),
            ("m2", 1e+20),
            ("m^2", 1e+20),
            ("Morgen", 2.5e+23),
            ("mi2", 2.589988110336e+26),
            ("mi^2", 2.589988110336e+26),
            ("Nmi2", 3.429904e+26),
            ("Nmi^2", 3.429904e+26),
            ("Picapt2", 12445216049382.715),
            ("Pica2", 12445216049382.715),
            ("Pica^2", 12445216049382.715),
            ("Picapt^2", 12445216049382.715),
            ("yd2", 8.3612736e+19),
            ("yd^2", 8.3612736e+19),
        ]);
        units.insert("area", area);

        let information = HashMap::from([("bit", 1.0), ("byte", 8.0)]);
        units.insert("information", information);

        let speed = HashMap::from([
            ("admkn", 0.5147733333333333),
            ("kn", 0.5144444444444445),
            ("m/h", 0.0002777777777777778),
            ("m/hr", 0.0002777777777777778),
            ("m/s", 1.0),
            ("m/sec", 1.0),
            ("mph", 0.44704),
        ]);
        units.insert("speed", speed);

        let temperature = HashMap::from([
            ("C", 1.0),
            ("cel", 1.0),
            ("F", 1.0),
            ("fah", 1.0),
            ("K", 1.0),
            ("kel", 1.0),
            ("Rank", 1.0),
            ("Reau", 1.0),
        ]);
        units.insert("temperature", temperature);

        // only some units admit prefixes (the is no kC, kilo Celsius, for instance)
        let mks = [
            "Pa", "p", "atm", "at", "mmHg", "g", "u", "m", "ang", "ly", "parsec", "pc", "ang2",
            "ang^2", "ar", "m2", "m^2", "N", "dyn", "dy", "pond", "J", "e", "c", "cal", "eV", "ev",
            "Wh", "wh", "W", "w", "T", "ga", "uk_pt", "l", "L", "lt", "ang3", "ang^3", "m3", "m^3",
            "bit", "byte", "m/h", "m/hr", "m/s", "m/sec", "mph", "K", "kel",
        ];
        let volumes = ["ang3", "ang^3", "m3", "m^3"];

        // We need all_units to make sure tha pc is interpreted as parsec and not pico centimeters
        // We could have this list hard coded, of course.
        let mut all_units = Vec::new();
        for unit in units.values() {
            for &unit_name in unit.keys() {
                all_units.push(unit_name);
            }
        }

        // A unit's factor (in its class's internal base) and its prefix scale are kept apart:
        // Excel applies the prefix last (see the end of this function).
        let mut to_unit_factor = 1.0;
        let mut from_unit_factor = 1.0;
        let mut to_unit_scale = 1.0;
        let mut from_unit_scale = 1.0;

        // kind of units (weight, distance, time, ...)
        let mut to_unit_kind = "";
        let mut from_unit_kind = "";

        let mut to_unit_name = "";
        let mut from_unit_name = "";

        for (&name, unit) in &units {
            for (&unit_name, unit_value) in unit {
                if let Some(pk) = from_unit.strip_suffix(unit_name) {
                    if pk.is_empty() {
                        from_unit_kind = name;
                        from_unit_factor = *unit_value;
                        from_unit_name = unit_name;
                    } else if let Some(modifier) = prefix.get(pk) {
                        if mks.contains(&unit_name) && !all_units.contains(&from_unit.as_str()) {
                            // We make sure:
                            // 1. It is a unit that admits a modifier (like metres or grams)
                            // 2. from_unit is not itself a unit
                            let scale = if name == "area" && unit_name != "ar" {
                                // 1 km2 is actually 10^6 m2
                                *modifier * modifier
                            } else if name == "volume" && volumes.contains(&unit_name) {
                                // don't look at me I don't make the rules!
                                *modifier * modifier * modifier
                            } else {
                                *modifier
                            };
                            from_unit_kind = name;
                            from_unit_factor = *unit_value;
                            from_unit_scale = scale;
                            from_unit_name = unit_name;
                        }
                    }
                }
                if let Some(pk) = to_unit.strip_suffix(unit_name) {
                    if pk.is_empty() {
                        to_unit_kind = name;
                        to_unit_factor = *unit_value;
                        to_unit_name = unit_name;
                    } else if let Some(modifier) = prefix.get(pk) {
                        if mks.contains(&unit_name) && !all_units.contains(&to_unit.as_str()) {
                            let scale = if name == "area" && unit_name != "ar" {
                                *modifier * modifier
                            } else if name == "volume" && volumes.contains(&unit_name) {
                                *modifier * modifier * modifier
                            } else {
                                *modifier
                            };
                            to_unit_kind = name;
                            to_unit_factor = *unit_value;
                            to_unit_scale = scale;
                            to_unit_name = unit_name;
                        }
                    }
                }
                if !from_unit_kind.is_empty() && !to_unit_kind.is_empty() {
                    break;
                }
            }
            if !from_unit_kind.is_empty() && !to_unit_kind.is_empty() {
                break;
            }
        }
        if from_unit_kind != to_unit_kind {
            return CalcResult::new_error(Error::NA, cell, String::new());
        }

        // Let's check if it is temperature;
        if from_unit_kind.is_empty() {
            return CalcResult::new_error(Error::NA, cell, String::new());
        }

        if from_unit_kind == "temperature" {
            // Temperature requires formula conversion
            // Kelvin (K, k), Celsius (C,cel), Rankine (Rank), Réaumur (Reau)
            let to_temperature = match to_unit_name {
                "K" | "kel" => Temperature::Kelvin,
                "C" | "cel" => Temperature::Celsius,
                "Rank" => Temperature::Rankine,
                "Reau" => Temperature::Reaumur,
                "F" | "fah" => Temperature::Fahrenheit,
                _ => {
                    return CalcResult::new_error(Error::ERROR, cell, String::new());
                }
            };
            let from_temperature = match from_unit_name {
                "K" | "kel" => Temperature::Kelvin,
                "C" | "cel" => Temperature::Celsius,
                "Rank" => Temperature::Rankine,
                "Reau" => Temperature::Reaumur,
                "F" | "fah" => Temperature::Fahrenheit,
                _ => {
                    return CalcResult::new_error(Error::ERROR, cell, String::new());
                }
            };
            let t = convert_temperature(
                value * (from_unit_scale * from_unit_factor),
                from_temperature,
                to_temperature,
            ) / (to_unit_scale * to_unit_factor);
            return CalcResult::Number(t);
        }
        // Excel: the value is multiplied by the from-factor, divided by the to-factor, and only then
        // scaled by the prefixes, in that order (docs/excel-semantics/engineering.md "CONVERT
        // constants and evaluation order"; ironcalc-calc ENGINEERING/CONVERT.xlsx, e.g. Volume!J17
        // `CONVERT(0.0002,"Mang3","m3")` = 2.0000000000000002E-16).
        // Excel: `1E290 m2 -> m2`, `1E300 ha -> m2` and `1E302 J -> J` do not overflow although
        // `value * F_from` does (probes/convert-targets): when the product is infinite the
        // division goes first. The target prefix is applied as a reciprocal multiply
        // (`CONVERT(0.0002,"m3","Mm3")` is 2.0000000000000003E-22, not 2E-22).
        let product = value * from_unit_factor;
        let scaled = if product.is_infinite() {
            value / to_unit_factor * from_unit_factor
        } else {
            product / to_unit_factor
        };
        // Excel: `CONVERT(1E290,"Ym","Ym")` is 1E+290 and `CONVERT(1E300,"Ym","Ym")` 1E+300 although
        // `value * 1E24` overflows (probes/convert-temperature-ov00, -ov01): the prefixes are combined
        // into one ratio when applying the from-prefix first would overflow. Not an "always combine"
        // rule: `CONVERT(1E300,"Ym","m")` is still `#NUM!` (ov02).
        let from_scaled = scaled * from_unit_scale;
        let result = if from_scaled.is_infinite() {
            scaled * (from_unit_scale * (1.0 / to_unit_scale))
        } else {
            from_scaled * (1.0 / to_unit_scale)
        };
        CalcResult::Number(result)
    }
}
