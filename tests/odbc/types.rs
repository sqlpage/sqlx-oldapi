#![allow(clippy::approx_constant)]
use sqlx_oldapi::odbc::Odbc;
use sqlx_test::{test_decode_type, test_type};

// Basic null test
test_type!(null<Option<i32>>(Odbc,
    "NULL::int" == None::<i32>
));

// Boolean type
test_type!(bool(Odbc, "1" == true, "0" == false));

// Signed integer types
test_type!(i8(
    Odbc,
    "5" == 5_i8,
    "0" == 0_i8,
    "-1" == -1_i8,
    "127" == 127_i8,
    "-128" == -128_i8
));

test_type!(i16(
    Odbc,
    "21415" == 21415_i16,
    "-2144" == -2144_i16,
    "0" == 0_i16,
    "32767" == 32767_i16,
    "-32768" == -32768_i16
));

test_type!(i32(
    Odbc,
    "94101" == 94101_i32,
    "-5101" == -5101_i32,
    "0" == 0_i32,
    "2147483647" == 2147483647_i32,
    "-2147483648" == -2147483648_i32
));

test_type!(i64(
    Odbc,
    "9358295312" == 9358295312_i64,
    "-9223372036854775808" == -9223372036854775808_i64,
    "0" == 0_i64,
    "9223372036854775807" == 9223372036854775807_i64
));

// Unsigned integer types
test_type!(u8(Odbc, "255" == 255_u8, "0" == 0_u8, "127" == 127_u8));

test_type!(u16(
    Odbc,
    "65535" == 65535_u16,
    "0" == 0_u16,
    "32767" == 32767_u16
));

test_type!(u32(
    Odbc,
    "4294967295" == 4294967295_u32,
    "0" == 0_u32,
    "2147483647" == 2147483647_u32
));

test_type!(u64(
    Odbc,
    "9223372036854775807" == 9223372036854775807_u64,
    "0" == 0_u64,
    "4294967295" == 4294967295_u64
));

// Floating point types
test_type!(f32(
    Odbc,
    "3.125" == 3.125_f32, // Use power-of-2 fractions for exact representation
    "0.0" == 0.0_f32,
    "-2.5" == -2.5_f32
));

test_type!(f64(
    Odbc,
    "939399419.1225182" == 939399419.1225182_f64,
    "3.14159265358979" == 3.14159265358979_f64,
    "0.0" == 0.0_f64,
    "-1.23456789" == -1.23456789_f64
));

// String types
test_type!(string<String>(Odbc,
    "'hello world'" == "hello world",
    "''" == "",
    "'test'" == "test",
    "'Unicode: 🦀 Rust'" == "Unicode: 🦀 Rust"
));

// Note: Binary data testing requires special handling in ODBC and is tested separately

// Feature-gated types
#[cfg(feature = "uuid")]
test_type!(uuid<sqlx_oldapi::types::Uuid>(Odbc,
    "'550e8400-e29b-41d4-a716-446655440000'" == sqlx_oldapi::types::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
    "'00000000-0000-0000-0000-000000000000'" == sqlx_oldapi::types::Uuid::nil()
));

// Extra UUID decoding edge cases (ODBC may return padded strings)
#[cfg(feature = "uuid")]
test_type!(uuid_padded<sqlx_oldapi::types::Uuid>(Odbc,
    "'550e8400-e29b-41d4-a716-446655440000  '" == sqlx_oldapi::types::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
));

#[cfg(feature = "json")]
mod json_tests {
    use super::*;
    use serde_json::{json, Value as JsonValue};

    test_type!(json<JsonValue>(Odbc,
        "'{\"name\":\"test\",\"value\":42}'" == json!({"name": "test", "value": 42}),
        "'\"hello\"'" == json!("hello"),
        "'[1,2,3]'" == json!([1, 2, 3]),
        "'null'" == json!(null)
    ));
}

#[cfg(feature = "bigdecimal")]
test_type!(bigdecimal<sqlx_oldapi::types::BigDecimal>(Odbc,
    "'123.456789'" == "123.456789".parse::<sqlx_oldapi::types::BigDecimal>().unwrap(),
    "'0'" == "0".parse::<sqlx_oldapi::types::BigDecimal>().unwrap(),
    "'999999.999999'" == "999999.999999".parse::<sqlx_oldapi::types::BigDecimal>().unwrap(),
    "'-123.456'" == "-123.456".parse::<sqlx_oldapi::types::BigDecimal>().unwrap()
));

#[cfg(feature = "decimal")]
test_type!(decimal<sqlx_oldapi::types::Decimal>(Odbc,
    "'123.456789'" == "123.456789".parse::<sqlx_oldapi::types::Decimal>().unwrap(),
    "'0'" == "0".parse::<sqlx_oldapi::types::Decimal>().unwrap(),
    "'999.123'" == "999.123".parse::<sqlx_oldapi::types::Decimal>().unwrap(),
    "'-456.789'" == "-456.789".parse::<sqlx_oldapi::types::Decimal>().unwrap()
));

#[cfg(feature = "chrono")]
mod chrono_tests {
    use super::*;
    use sqlx_oldapi::types::chrono::{
        DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, Utc,
    };

    test_type!(chrono_date<NaiveDate>(Odbc,
        "'2023-12-25'" == NaiveDate::from_ymd_opt(2023, 12, 25).unwrap(),
        "'2001-01-05'" == NaiveDate::from_ymd_opt(2001, 1, 5).unwrap(),
        "'2050-11-23'" == NaiveDate::from_ymd_opt(2050, 11, 23).unwrap()
    ));

    test_type!(chrono_time<NaiveTime>(Odbc,
        "'14:30:00'" == NaiveTime::from_hms_opt(14, 30, 0).unwrap(),
        "'23:59:59'" == NaiveTime::from_hms_opt(23, 59, 59).unwrap(),
        "'00:00:00'" == NaiveTime::from_hms_opt(0, 0, 0).unwrap()
    ));

    test_type!(chrono_datetime<NaiveDateTime>(Odbc,
        "'2023-12-25 14:30:00'" == NaiveDate::from_ymd_opt(2023, 12, 25).unwrap().and_hms_opt(14, 30, 0).unwrap(),
        "'2019-01-02 05:10:20'" == NaiveDate::from_ymd_opt(2019, 1, 2).unwrap().and_hms_opt(5, 10, 20).unwrap()
    ));

    // Extra chrono decoding edge case (padded timestamp string)
    test_decode_type!(chrono_datetime_padded<NaiveDateTime>(Odbc,
        "'2023-12-25 14:30:00   '" == NaiveDate::from_ymd_opt(2023, 12, 25).unwrap().and_hms_opt(14, 30, 0).unwrap()
    ));

    test_type!(chrono_datetime_utc<DateTime<Utc>>(Odbc,
        "'2023-12-25 14:30:00'" == DateTime::<Utc>::from_naive_utc_and_offset(
            NaiveDate::from_ymd_opt(2023, 12, 25).unwrap().and_hms_opt(14, 30, 0).unwrap(),
            Utc,
        ),
        "'2019-01-02 05:10:20'" == DateTime::<Utc>::from_naive_utc_and_offset(
            NaiveDate::from_ymd_opt(2019, 1, 2).unwrap().and_hms_opt(5, 10, 20).unwrap(),
            Utc,
        )
    ));

    test_type!(chrono_datetime_fixed<DateTime<FixedOffset>>(Odbc,
        "'2023-12-25 14:30:00'" == DateTime::<Utc>::from_naive_utc_and_offset(
            NaiveDate::from_ymd_opt(2023, 12, 25).unwrap().and_hms_opt(14, 30, 0).unwrap(),
            Utc,
        ).fixed_offset()
    ));
}

// Cross-type compatibility tests
test_type!(cross_type_integer_compatibility<i64>(Odbc,
    "127" == 127_i64,
    "32767" == 32767_i64,
    "2147483647" == 2147483647_i64
));

test_type!(cross_type_unsigned_compatibility<u32>(Odbc,
    "255" == 255_u32,
    "65535" == 65535_u32
));

test_type!(cross_type_float_compatibility<f64>(Odbc,
    "3.14159" == 3.14159_f64,
    "123.456789" == 123.456789_f64
));

// Type coercion from strings
test_type!(string_to_integer<i32>(Odbc,
    "'42'" == 42_i32,
    "'-123'" == -123_i32
));

test_type!(string_to_float<f64>(Odbc,
    "'3.14159'" == 3.14159_f64,
    "'-2.718'" == -2.718_f64
));

test_type!(string_to_bool<bool>(Odbc,
    "'1'" == true,
    "'0'" == false
));
