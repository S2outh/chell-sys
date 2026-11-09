#![feature(const_trait_impl)]
#![feature(const_cmp)]
use std::collections::HashSet;

use chell::*;

#[cfg(feature = "ground")]
extern crate alloc;

#[derive(ChellValue, Default, Clone, Copy)]
#[cfg_attr(feature = "ground", derive(serde::Serialize))]
pub struct TestValue {
    pub val: u32,
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

    #[def(i64)]
    struct Timestamp;

    #[def(u32)]
    struct FirstChellValue;

    #[def(crate::TestValue)]
    struct SecondChellValue;

    mod some_other_mod {

        #[def(crate::TestVector)]
        struct ThirdChellValue;
    }
}

#[test]
fn test_ids_unequal() {
    let ids = [
        telemetry::Timestamp.id().get(0).unwrap(),
        telemetry::FirstChellValue.id().get(0).unwrap(),
        telemetry::SecondChellValue.id().get(0).unwrap(),
        telemetry::some_other_mod::ThirdChellValue
            .id()
            .get(0)
            .unwrap(),
    ];
    let mut seen = HashSet::new();
    assert!(!ids.iter().any(|x| !seen.insert(x)))
}

#[test]
fn test_addr_gen() {
    assert_eq!(telemetry::Timestamp.address(), "telemetry.timestamp");
    assert_eq!(
        telemetry::FirstChellValue.address(),
        "telemetry.first_chell_value"
    );
    assert_eq!(
        telemetry::some_other_mod::ThirdChellValue.address(),
        "telemetry.some_other_mod.third_chell_value"
    );
}

#[test]
fn test_id_recover() {
    let def = telemetry::some_other_mod::ThirdChellValue;
    let recovered = telemetry::from_id(def.id().get(0).unwrap()).unwrap();
    assert!(
        &recovered
            .as_any()
            .is::<telemetry::some_other_mod::ThirdChellValue>()
    );
}

#[test]
fn test_addr_recover() {
    let def = telemetry::FirstChellValue;
    let recovered = telemetry::from_address(def.address()).unwrap();
    assert!(&recovered.as_any().is::<telemetry::FirstChellValue>());
}
