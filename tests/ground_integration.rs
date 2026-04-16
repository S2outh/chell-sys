#![feature(const_trait_impl)]
#![feature(const_cmp)]
#![cfg(feature = "ground")]

use chell::*;

use crate::telemetry::{FirstChellValue, SecondChellValue, some_other_mod::ThirdChellValue};
extern crate alloc;

#[derive(ChellValue, Default, Clone, Copy, serde::Serialize)]
pub struct TestValue {
    pub val: u32,
}

#[derive(ChellValue, Default, Clone, Copy, serde::Serialize)]
pub struct TestVector {
    x: i16,
    y: f32,
    z: TestValue,
}

fn transfer(value: &u32) -> f32 {
    (value * 3) as f32
}

#[chell_definition(id = 0)]
mod telemetry {
    #[chv(i64)]
    struct Timestamp;
    #[chv(u32, c = crate::transfer)]
    struct FirstChellValue;
    #[chv(crate::TestValue, other = |v: &crate::TestValue| v.val)]
    struct SecondChellValue;
    #[chm(id = 100)]
    mod some_other_mod {
        #[chv(crate::TestVector)]
        struct ThirdChellValue;
    }
}

beacon!(
    TestBeacon,
    crate::telemetry,
    crate::telemetry::Timestamp,
    id = 0,
    values(
        FirstChellValue,
        SecondChellValue,
        some_other_mod::ThirdChellValue
    )
);

fn serializer_func(
    value: &dyn erased_serde::Serialize,
) -> Result<alloc::vec::Vec<u8>, erased_serde::Error> {
    let mut buffer = alloc::vec::Vec::new();
    let mut serializer = serde_cbor::Serializer::new(&mut buffer);
    value.erased_serialize(&mut <dyn erased_serde::Serializer>::erase(&mut serializer))?;
    Ok(buffer)
}

#[test]
fn beacon_serialize() {
    let mut beacon = test_beacon::TestBeacon::new();

    let first_value = 1234u32;
    let second_value = TestValue { val: 3 };
    let third_value = TestVector {
        x: 3,
        y: 3.3,
        z: TestValue { val: 1 },
    };
    let addresses = vec![
        "telemetry.first_chell_value.c",
        "telemetry.first_chell_value",
        "telemetry.second_chell_value.other",
        "telemetry.second_chell_value",
        "telemetry.some_other_mod.third_chell_value",
    ];

    beacon.first_chell_value = Some(first_value);
    beacon.second_chell_value = Some(second_value);
    beacon.some_other_mod_third_chell_value = Some(third_value);

    let serialized_pairs = beacon.serialize(&serializer_func).unwrap();
    for (ser, address) in serialized_pairs.iter().zip(addresses) {
        assert_eq!(ser.0, address);
    }
}

macro_rules! to_bytes {
    ($type: ty, $chell_value:ident) => {{
        let mut bytes = [0u8; <$type>::MAX_BYTE_SIZE];
        $chell_value.write(&mut bytes).unwrap();
        bytes
    }};
}
#[test]
fn tm_serialize() {
    let first_value = 1234u32;
    let second_value = TestValue { val: 3 };
    let third_value = TestVector {
        x: 3,
        y: 3.3,
        z: TestValue { val: 1 },
    };
    let addresses = vec![
        "telemetry.first_chell_value.c",
        "telemetry.first_chell_value",
        "telemetry.second_chell_value.other",
        "telemetry.second_chell_value",
        "telemetry.some_other_mod.third_chell_value",
    ];

    let mut serialized_pairs = Vec::new();
    serialized_pairs.append(
        &mut FirstChellValue
            .reserialize(&to_bytes!(u32, first_value), &12, &serializer_func)
            .unwrap(),
    );
    serialized_pairs.append(
        &mut SecondChellValue
            .reserialize(&to_bytes!(TestValue, second_value), &12, &serializer_func)
            .unwrap(),
    );
    serialized_pairs.append(
        &mut ThirdChellValue
            .reserialize(&to_bytes!(TestVector, third_value), &12, &serializer_func)
            .unwrap(),
    );
    for (ser, address) in serialized_pairs.iter().zip(addresses) {
        assert_eq!(ser.0, address);
    }
}
