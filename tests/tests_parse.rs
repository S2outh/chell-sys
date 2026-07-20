#![feature(const_trait_impl)]
#![feature(const_cmp)]
#![feature(const_default)]

use chell::*;

#[derive(ChellValue, Default, Clone, Copy)]
#[cfg_attr(feature = "ground", derive(serde::Serialize))]
pub struct TestValue {
    val: u32,
}

#[derive(ChellValue, Default, Clone, Copy)]
#[cfg_attr(feature = "ground", derive(serde::Serialize))]
pub struct TestVector {
    x: i16,
    y: f32,
    z: TestValue,
}

#[chell_definition(id = 0)]
mod telemetry {
    #[chv(u32, result(f64, |v: &u32| *v as f64 / 2.))]
    struct FirstChellValue;
    #[chv(crate::TestValue)]
    struct SecondChellValue;
}

#[cfg(feature = "ground")]
extern crate alloc;

#[test]
fn run_parse_fn() {
    let first_value = 1234u32;
    let parsed = first_value.parser(telemetry::FirstChellValue).result();

    assert_eq!(parsed, first_value as f64 / 2.);
}
