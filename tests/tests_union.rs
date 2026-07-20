#![feature(const_trait_impl)]
#![feature(const_cmp)]
use chell::{_internal::InternalChellDefinition, *};

#[cfg(feature = "ground")]
extern crate alloc;

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
    #[chv(u32)]
    struct FirstChellValue;
    #[chv(crate::TestVector)]
    struct SecondChellValue;
    #[chv(Option<i32>)]
    struct OptionTest;
    #[chv([i16; 2])]
    struct ArrayTest;
    #[chm(id = 100)]
    mod some_other_mod {
        #[chv(u64)]
        struct ThirdChellValue;
        #[chv(i32)]
        struct FourthChellValue;
        #[chv(crate::TestValue)]
        struct FifthChellValue;
    }
}

type ValueTestContainer = fd_compat_chell_union!(telemetry::OptionTest);
type PartialTestContainer = fd_compat_chell_union!(telemetry::some_other_mod);
type FullTestContainer = fd_compat_chell_union!(telemetry);

#[test]
fn value_container_creation() {
    assert_eq!(telemetry::OptionTest::MAX_BYTE_SIZE, 5);
    assert_eq!(ValueTestContainer::SIZE, 5);

    let container = ValueTestContainer::new(&telemetry::OptionTest, &Some(22)).unwrap();
    assert_eq!(container.id(), 2);

    assert_eq!(container.bytes().len(), 5);
    assert_eq!(container.bytes()[0], 1);
    assert_eq!(container.bytes()[1..5], 22i32.to_le_bytes());
}

#[test]
fn partial_container_creation() {
    assert_eq!(telemetry::some_other_mod::MAX_BYTE_SIZE, 8);
    assert_eq!(PartialTestContainer::SIZE, 8);

    let container =
        PartialTestContainer::new(&telemetry::some_other_mod::FourthChellValue, &42).unwrap();
    assert_eq!(container.id(), 101);

    assert_eq!(container.bytes().len(), 4);
    assert_eq!(container.bytes()[0..4], 42i32.to_le_bytes());
}

#[test]
fn full_container_creation() {
    assert_eq!(telemetry::MAX_BYTE_SIZE, 10);
    assert_eq!(FullTestContainer::SIZE, 12);

    let container = FullTestContainer::new(
        &telemetry::SecondChellValue,
        &TestVector {
            x: 12,
            y: 24.,
            z: TestValue { val: 36 },
        },
    )
    .unwrap();
    assert_eq!(container.id(), 1);

    assert_eq!(container.bytes().len(), 10);
    assert_eq!(container.bytes()[0..2], 12i16.to_le_bytes());
    assert_eq!(container.bytes()[2..6], 24f32.to_le_bytes());
    assert_eq!(container.bytes()[6..10], 36u32.to_le_bytes());
}
