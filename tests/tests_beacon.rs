#![feature(const_trait_impl)]
#![feature(const_cmp)]
#![feature(const_default)]

use chell::{_internal::InternalChellDefinition, *};

#[derive(ChellValue, Default, Clone, Copy, PartialEq, Debug)]
#[cfg_attr(feature = "ground", derive(serde::Serialize))]
pub struct TestValue {
    val: u32,
}

#[derive(ChellValue, Default, Clone, Copy, PartialEq, Debug)]
#[cfg_attr(feature = "ground", derive(serde::Serialize))]
pub struct TestVector {
    x: i16,
    y: f32,
    z: TestValue,
}

#[chell_definition(id = 0)]
mod telemetry {
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

#[cfg(feature = "ground")]
extern crate alloc;

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

macro_rules! to_bytes {
    ($type: ty, $chell_value:ident) => {{
        let mut bytes = [0u8; <$type>::MAX_BYTE_SIZE];
        $chell_value.write(&mut bytes).unwrap();
        bytes
    }};
}

fn crc_ccitt(bytes: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for byte in bytes {
        crc ^= (*byte as u16) << 8;
        for _ in 0..8 {
            if (crc & 0x8000) != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

#[test]
fn beacon_creation() {
    let mut beacon = TestBeacon::new();

    let first_value = 1234u32;
    let second_value = TestValue { val: 3 };
    let third_value = TestVector {
        x: 3,
        y: 3.3,
        z: TestValue { val: 1 },
    };

    beacon.first_chell_value = Some(first_value);
    beacon.second_chell_value = Some(second_value);
    beacon.some_other_mod_third_chell_value = Some(third_value);

    let fix_header_bytes = 3; // id + crc
    let var_header_bytes = 1; // only 3 fields, 1 byte bitflag is enough
    let sizes = [
        fix_header_bytes,
        var_header_bytes,
        1, // Timestamp is 0
        2, // 1234 takes two varint bytes
        1, // 3 takes one varint byte
        1, // x is one varint byte
        4, // y is a float using 4 bytes
        1, // z takes 1 varint byte
    ];
    println!("{:?}", beacon.to_bytes(&mut crc_ccitt));
    assert_eq!(beacon.to_bytes(&mut crc_ccitt).len(), sizes.iter().sum());
}

#[test]
fn beacon_reserialization() {
    let mut beacon = TestBeacon::new();

    let first_value = 1234u32;
    let second_value = TestValue { val: 3 };
    let third_value = TestVector {
        x: 3,
        y: 3.3,
        z: TestValue { val: 1 },
    };

    beacon.first_chell_value = Some(first_value);
    beacon.second_chell_value = Some(second_value);
    beacon.some_other_mod_third_chell_value = Some(third_value);

    let bytes = beacon.to_bytes(&mut crc_ccitt);
    let crc = crc_ccitt(&bytes[3..]);

    // check crc and id
    assert_eq!(bytes[0], 0);
    assert_eq!(bytes[1..3], crc.to_le_bytes());

    // deserialize and test eq
    let mut beacon_copy = TestBeacon::new();
    beacon_copy.from_bytes(bytes, &mut crc_ccitt).unwrap();
    assert_eq!(beacon_copy.first_chell_value, Some(first_value));
    assert_eq!(beacon_copy.second_chell_value, Some(second_value));
    assert_eq!(
        beacon_copy.some_other_mod_third_chell_value,
        Some(third_value)
    );
}

#[test]
fn beacon_insertion_id() {
    let mut id_beacon = TestBeacon::new();
    let mut beacon = TestBeacon::new();

    let first_value = 1234u32;
    let second_value = TestValue { val: 3 };
    let third_value = TestVector {
        x: 3,
        y: 3.3,
        z: TestValue { val: 1 },
    };

    id_beacon
        .insert_slice(telemetry::from_id(1).unwrap(), &to_bytes!(u32, first_value))
        .unwrap();
    id_beacon
        .insert_slice(
            telemetry::from_id(2).unwrap(),
            &to_bytes!(TestValue, second_value),
        )
        .unwrap();
    id_beacon
        .insert_slice(
            telemetry::from_id(100).unwrap(),
            &to_bytes!(TestVector, third_value),
        )
        .unwrap();

    beacon.first_chell_value = Some(first_value);
    beacon.second_chell_value = Some(second_value);
    beacon.some_other_mod_third_chell_value = Some(third_value);

    assert_eq!(
        id_beacon.to_bytes(&mut crc_ccitt),
        beacon.to_bytes(&mut crc_ccitt)
    );
}

#[test]
fn beacon_insertion_address() {
    let mut address_beacon = TestBeacon::new();
    let mut beacon = TestBeacon::new();

    let first_value = 1234u32;
    let second_value = TestValue { val: 3 };
    let third_value = TestVector {
        x: 3,
        y: 3.3,
        z: TestValue { val: 1 },
    };

    address_beacon
        .insert_slice(
            telemetry::from_address("telemetry.first_chell_value").unwrap(),
            &to_bytes!(u32, first_value),
        )
        .unwrap();
    address_beacon
        .insert_slice(
            telemetry::from_address("telemetry.second_chell_value").unwrap(),
            &to_bytes!(TestValue, second_value),
        )
        .unwrap();
    address_beacon
        .insert_slice(
            telemetry::from_address("telemetry.some_other_mod.third_chell_value").unwrap(),
            &to_bytes!(TestVector, third_value),
        )
        .unwrap();

    beacon.first_chell_value = Some(first_value);
    beacon.second_chell_value = Some(second_value);
    beacon.some_other_mod_third_chell_value = Some(third_value);

    assert_eq!(
        address_beacon.to_bytes(&mut crc_ccitt),
        beacon.to_bytes(&mut crc_ccitt)
    );
}
