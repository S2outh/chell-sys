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
    let v: &dyn ChellDefinition = &telemetry::Timestamp;
    match_value!(v, {
        telemetry::Timestamp => (),
        telemetry::FirstChellValue => panic!(),
    });
}
