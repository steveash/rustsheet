#![allow(clippy::unwrap_used)]
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

#[test]
fn function_iterator() {
    // Checks that `impl_function_lookup!`, which generates `Function::into_iter`, lists
    // every variant of the enum: Rust cannot iterate an enum's variants, so this reads the
    // enum's variants from this file and compares the counts.
    let file = File::open("src/functions/mod.rs").unwrap();
    let reader = BufReader::new(file);
    let mut start = false;
    let mut list = Vec::new();

    for line in reader.lines() {
        let text = line.unwrap();
        let text = text.trim().trim_end_matches(',');
        if text == "pub enum Function {" {
            start = true;
            continue;
        }
        if start {
            if text == "}" {
                break;
            }
            if text.starts_with("//") {
                // skip comments
                continue;
            }
            if text.is_empty() {
                // skip empty lines
                continue;
            }
            list.push(text.to_owned());
        }
    }
    assert_eq!(list.len(), crate::functions::Function::into_iter().count());
}
