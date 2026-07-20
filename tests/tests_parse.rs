#![feature(const_trait_impl)]
#![feature(const_cmp)]
#![feature(const_default)]

use chell::*;

#[derive(ChellValue, Default, Clone, Copy)]
#[cfg_attr(feature = "ground", derive(serde::Serialize))]
pub struct TestValue {
    val: u32,
}

#[chell_definition(id = 0)]
mod telemetry {
    #[chv(u32, result(f64, |v: &u32| *v as f64 / 2.))]
    struct FirstChellValue;
    #[chv(crate::TestValue, first(u32, |v: &crate::TestValue| v.val, ground))]
    struct SecondChellValue;
}

#[cfg(feature = "ground")]
extern crate alloc;

#[test]
fn run_parse_fn() {
    let value = 1234u32;
    let parsed = value.parser(telemetry::FirstChellValue).result();

    assert_eq!(parsed, value as f64 / 2.);
}

#[cfg(feature = "ground")]
#[test]
fn run_gnd_parse_fn() {
    let value = TestValue { val: 1234u32 };
    let parsed = value.parser(telemetry::SecondChellValue).first();

    assert_eq!(parsed, value.val);
}
