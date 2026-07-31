#![feature(const_trait_impl)]
#![feature(const_cmp)]

use chell::*;

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
    /// Test doc
    #[chv(i64)]
    struct Timestamp;
    #[chv(u32)]
    struct FirstChellValue;
    #[chv(crate::TestValue)]
    struct SecondChellValue;
    #[chm(id = 100)]
    mod some_other_mod {
        #[chv(crate::TestVector)]
        struct ThirdChellValue;
    }
}

#[test]
fn test_match() {
    let def: &dyn ChellDefinition = &telemetry::Timestamp;
    match_def!(def, {
        telemetry::Timestamp => (),
        telemetry::FirstChellValue => panic!(),
    });
}

#[test]
fn test_match_default() {
    let def: &dyn ChellDefinition = &telemetry::Timestamp;
    match_def!(def, {
        telemetry::Timestamp => (),
        telemetry::FirstChellValue => panic!(),
        :default panic!()
    });
}

#[test]
fn test_match_deserialize() {
    let def: &dyn ChellDefinition = &telemetry::Timestamp;
    let v = 214i64;
    let bytes = telemetry::Timestamp.serialize(&v).unwrap();

    match_def!(def, bytes: bytes, {
        telemetry::Timestamp [deserialized_value] => assert_eq!(v, deserialized_value),
        telemetry::FirstChellValue => panic!(),
        :default panic!()
    });
}

#[test]
fn test_match_deserialization_error() {
    let def: &dyn ChellDefinition = &telemetry::Timestamp;
    let v = 214i64;
    let bytes = telemetry::Timestamp.serialize(&v).unwrap();

    match_def!(def, bytes: bytes, error: panic!(), {
        telemetry::Timestamp [deserialized_value] => assert_eq!(v, deserialized_value),
        telemetry::FirstChellValue => panic!(),
        :default panic!()
    });
}

#[test]
fn test_deserialize() {
    let v = 802267u32;
    let bytes = telemetry::FirstChellValue.serialize(&v).unwrap();
    assert_eq!(v, telemetry::FirstChellValue.deserialize(&bytes).unwrap())
}
