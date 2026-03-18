#![feature(const_trait_impl)]
#![feature(const_cmp)]
#![cfg(feature = "ground")]

use chell::*;
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

struct CborSerializer;
impl ground::Serializer for CborSerializer {
    type Error = serde_cbor::Error;
    fn serialize_value<T: serde::Serialize>(
        &self,
        value: &T,
    ) -> Result<std::vec::Vec<u8>, Self::Error> {
        serde_cbor::to_vec(value)
    }
}

#[test]
fn tm_serialize() {
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

    let serialized_pairs = beacon.serialize(&CborSerializer).unwrap();
    for (ser, address) in serialized_pairs.iter().zip(addresses) {
        assert_eq!(ser.0, address);
    }
}
