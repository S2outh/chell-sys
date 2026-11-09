#![feature(const_trait_impl)]
#![feature(const_cmp)]

use chell::*;

#[cfg(feature = "ground")]
extern crate alloc;

#[derive(ChellValue, Default, Clone, Copy)]
#[cfg_attr(feature = "ground", derive(serde::Serialize))]
pub struct TestValue {
    val: u32,
}

#[chell_definition(id = 0)]
mod telemetry {
    #[def(u32)]
    #[parser(result -> f64 |v: &u32| *v as f64 / 2.)]
    struct FirstChellValue;
    #[def(crate::TestValue)]
    #[parser(first -> u32 |v: &crate::TestValue| v.val)]
    struct SecondChellValue;
}

#[test]
fn run_parse_fn() {
    let value = 1234u32;
    let parsed = value.parser(telemetry::FirstChellValue).result();

    assert_eq!(parsed, value as f64 / 2.);
}
