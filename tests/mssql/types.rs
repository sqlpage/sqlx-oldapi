use core::f32;

use sqlx_oldapi::mssql::Mssql;
use sqlx_test::test_type;

test_type!(str<String>(Mssql,
    "'this is foo'" == "this is foo",
    "''" == "",
    "CAST('foo' AS VARCHAR(3))" == "foo",
    "CAST('foo' AS VARCHAR(max))" == "foo",
    "REPLICATE('a', 910)" == "a".repeat(910),
));

test_type!(str_unicode<String>(Mssql, "N'￮'" == "￮"));

test_type!(long_str<String>(Mssql,
    "REPLICATE(CAST('a' AS VARCHAR), 8000)" == "a".repeat(8000),
    "REPLICATE(CAST('a' AS VARCHAR(max)), 8192)" == "a".repeat(8192),
    "REPLICATE(CAST('a' AS NVARCHAR(max)), 8192)" == "a".repeat(8192),
    "REPLICATE(CAST('a' AS VARCHAR(max)), 100000)" == "a".repeat(100_000),
));

test_type!(null<Option<i32>>(Mssql,
    "CAST(NULL as INT)" == None::<i32>
));

test_type!(u8(
    Mssql,
    "CAST(5 AS TINYINT)" == 5_u8,
    "CAST(0 AS TINYINT)" == 0_u8,
    "CAST(255 AS TINYINT)" == 255_u8,
));

test_type!(i8(
    Mssql,
    "CAST(5 AS TINYINT)" == 5_i8,
    "CAST(0 AS TINYINT)" == 0_i8
));

test_type!(u8_edge_cases<u8>(
    Mssql,
    "CAST(0 AS TINYINT)" == 0_u8,
    "CAST(127 AS TINYINT)" == 127_u8,
    "CAST(128 AS TINYINT)" == 128_u8,
    "CAST(255 AS TINYINT)" == 255_u8,
));

test_type!(i16(Mssql, "CAST(21415 AS SMALLINT)" == 21415_i16));

test_type!(i16_edge_cases<i16>(
    Mssql,
    "CAST(-32768 AS SMALLINT)" == -32768_i16,
    "CAST(-1 AS SMALLINT)" == -1_i16,
    "CAST(0 AS SMALLINT)" == 0_i16,
    "CAST(32767 AS SMALLINT)" == 32767_i16,
));

test_type!(i32(Mssql, "CAST(2141512 AS INT)" == 2141512_i32));

test_type!(i32_edge_cases<i32>(
    Mssql,
    "CAST(-2147483648 AS INT)" == -2147483648_i32,
    "CAST(-1 AS INT)" == -1_i32,
    "CAST(0 AS INT)" == 0_i32,
    "CAST(2147483647 AS INT)" == 2147483647_i32,
));

test_type!(i64(Mssql, "CAST(32324324432 AS BIGINT)" == 32324324432_i64));

test_type!(i64_edge_cases<i64>(
    Mssql,
    "CAST(-9223372036854775808 AS BIGINT)" == -9223372036854775808_i64,
    "CAST(-1 AS BIGINT)" == -1_i64,
    "CAST(0 AS BIGINT)" == 0_i64,
    "CAST(9223372036854775807 AS BIGINT)" == 9223372036854775807_i64,
));

test_type!(f32(
    Mssql,
    "CAST(3.14159265358979323846264338327950288 AS REAL)" == f32::consts::PI,
    "CAST(0.5 AS DECIMAL(3,2))" == 0.5_f32,
));

test_type!(f64(
    Mssql,
    "CAST(939399419.1225182 AS FLOAT)" == 939399419.1225182_f64,
    "CAST(9.75 AS REAL)" == 9.75_f64,
));

test_type!(numeric<f64>(Mssql,
    "CAST(12 AS NUMERIC)" == 12_f64,
    "CAST(0.7 AS NUMERIC(2,1))" == 0.7_f64,
    "CAST(0.3 AS NUMERIC(2,1))" == 0.3_f64,
    "CAST(0.47 AS DECIMAL(3,2))" == 0.47_f64,
    "CAST(0.29 AS DECIMAL(3,2))" == 0.29_f64,
    "CAST(0.5678 AS NUMERIC(10,4))" == 0.5678_f64,
    "CAST(0.0003 AS NUMERIC(10,4))" == 0.0003_f64,
    "CAST(0.00003 AS NUMERIC(10,5))" == 0.00003_f64,
    "CAST(0.0000000000000001 AS NUMERIC(38,16))" == 0.0000000000000001_f64,
    "CAST(939399419.1225182 AS NUMERIC(15,2))" == 939399419.12_f64,
    "CAST(939399419.1225182 AS DECIMAL(15,2))" == 939399419.12_f64,
    "CAST(123456789.0123456789 AS NUMERIC(38,10))" == 123_456_789.012_345_678_9_f64,
    "CAST(123456789.0123456789012 AS NUMERIC(38,13))" == 123_456_789.012_345_678_901_2_f64,
    "CAST(123456789.012345678901234 AS NUMERIC(38,15))" == 123_456_789.012_345_678_901_234_f64,
    "CAST(1.0000000000000001 AS NUMERIC(18,16))" == 1.0000000000000001_f64,
    "CAST(0.99999999999999 AS NUMERIC(18,14))" == 0.99999999999999_f64,
    "CAST(2.00000000000001 AS NUMERIC(18,14))" == 2.00000000000001_f64,
    "CAST(333.33333333333333 AS NUMERIC(18,14))" == 333.333_333_333_333_3_f64,
    "CAST(0.14285714285714 AS NUMERIC(18,14))" == 0.14285714285714_f64,
    "CAST(9999999.99999999 AS NUMERIC(16,8))" == 9999999.99999999_f64, // Close to the precision limit
    "CAST(9007199254740992 AS NUMERIC(16,0))" == 9007199254740992_f64,    // 2^53, largest integer that can be exactly represented as a f64
    "CAST(0.000123456 AS NUMERIC(38,38))" == 0.000123456_f64,
    "CAST(1e-38 AS NUMERIC(38,38))" == 1e-38_f64,
));

test_type!(str_nvarchar<String>(Mssql,
    "CAST('this is foo' as NVARCHAR)" == "this is foo",
));

test_type!(bool(
    Mssql,
    "CAST(1 as BIT)" == true,
    "CAST(0 as BIT)" == false
));

test_type!(bytes<Vec<u8>>(Mssql,
    "0xDEADBEEF" == vec![0xDE_u8, 0xAD, 0xBE, 0xEF],
    "CAST(' ' AS VARBINARY)" == vec![0x20_u8],
    "CAST(REPLICATE(' ', 31) AS VARBINARY(max))" == vec![0x20_u8; 31],
));

test_type!(long_byte_buffer<Vec<u8>>(Mssql,
    "CAST(REPLICATE(CAST(' ' AS VARCHAR(max)), 100000) AS VARBINARY(max))" == vec![0x20_u8; 100000],
));

test_type!(empty_varbinary<Vec<u8>>(Mssql,
    "CAST('' AS VARBINARY)" == Vec::<u8>::new(),
));

test_type!(null_varbinary<Option<Vec<u8>>>(Mssql,
    "CAST(NULL AS VARBINARY)" == None::<Vec<u8>>,
    "CAST(NULL AS VARBINARY(max))" == None::<Vec<u8>>,
));

#[cfg(feature = "chrono")]
mod chrono {
    use super::*;
    use sqlx_core::types::chrono::{FixedOffset, NaiveTime, Utc};
    use sqlx_oldapi::types::chrono::{DateTime, NaiveDate, NaiveDateTime};

    test_type!(smalldatetime_type<DateTime<Utc>>(
        Mssql,
        "CAST('2023-07-31 23:59' as SmallDateTime)"
            == NaiveDateTime::parse_from_str("2023-07-31 23:59", "%Y-%m-%d %H:%M")
                .unwrap()
                .and_utc()
                .fixed_offset()
    ));

    test_type!(old_datetime_type<DateTime<Utc>>(
        Mssql,
        "CAST('1901-05-08 23:58:59' as DateTime)"
            == NaiveDateTime::parse_from_str("1901-05-08 23:58:59", "%Y-%m-%d %H:%M:%S")
                .unwrap()
                .and_utc()
                .fixed_offset()
    ));

    test_type!(old_datetime_type_as_naive<NaiveDateTime>(
        Mssql,
        "CAST('1901-05-08 23:58:59' as DateTime)"
            == NaiveDateTime::parse_from_str("1901-05-08 23:58:59", "%Y-%m-%d %H:%M:%S")
                .unwrap()
    ));

    test_type!(datetime2<NaiveDateTime>(
        Mssql,
        "CAST('2016-10-23 12:45:37.1234567' as DateTime2)"
            == NaiveDateTime::parse_from_str("2016-10-23 12:45:37.1234567", "%Y-%m-%d %H:%M:%S%.f")
                .unwrap()
    ));

    test_type!(datetimeoffset<DateTime<FixedOffset>>(
        Mssql,
        "CAST('2016-10-23 12:45:37.1234567 +02:00' as datetimeoffset(7))" == DateTime::parse_from_rfc3339("2016-10-23T12:45:37.1234567+02:00").unwrap()
    ));

    test_type!(NaiveDate(
        Mssql,
        "CAST('1789-07-14' AS DATE)"
            == NaiveDate::parse_from_str("1789-07-14", "%Y-%m-%d").unwrap()
    ));

    test_type!(NaiveTime(
        Mssql,
        "CAST('23:59:59.9999' AS TIME)"
            == NaiveTime::parse_from_str("23:59:59.9999", "%H:%M:%S%.f").unwrap(),
        "CAST('00:00' AS TIME)" == NaiveTime::default(),
    ));
}

#[cfg(feature = "decimal")]
mod decimal {
    use super::*;
    use sqlx_oldapi::types::Decimal;

    test_type!(Decimal(
        Mssql,
        "CAST('123456789.987654321' AS DECIMAL(18,9))"
            == Decimal::from_str_exact("123456789.987654321").unwrap(),
        "CAST('0' AS DECIMAL(1,0))" == Decimal::from_str_exact("0").unwrap(),
        "CAST('1' AS DECIMAL(1,0))" == Decimal::from_str_exact("1").unwrap(),
        "CAST('-1' AS DECIMAL(1,0))" == Decimal::from_str_exact("-1").unwrap(),
        "CAST('0.01234567890123456789' AS DECIMAL(38,20))"
            == Decimal::from_str_exact("0.01234567890123456789").unwrap(),
        "CAST('-12345678901234' AS DECIMAL(28,5))"
            == Decimal::from_str_exact("-12345678901234").unwrap(),
        "CAST('-1234567890.1234' AS MONEY)" == Decimal::from_str_exact("-1234567890.1234").unwrap(),
        "CAST('-123456.1234' AS SMALLMONEY)" == Decimal::from_str_exact("-123456.1234").unwrap(),
    ));
}

#[cfg(feature = "bigdecimal")]
mod bigdecimal {
    use super::*;
    use sqlx_oldapi::types::BigDecimal;
    use std::str::FromStr;

    test_type!(BigDecimal(
        Mssql,
        "CAST('0' AS DECIMAL(1,0))" == BigDecimal::from_str("0").unwrap(),
        "CAST('1' AS DECIMAL(1,0))" == BigDecimal::from_str("1").unwrap(),
        "CAST('-1' AS DECIMAL(1,0))" == BigDecimal::from_str("-1").unwrap(),
        "CAST('-12345678901234' AS DECIMAL(28,5))"
            == BigDecimal::from_str("-12345678901234").unwrap(),
        "CAST('-12345678901234567890' AS DECIMAL(38,5))"
            == BigDecimal::from_str("-12345678901234567890").unwrap(),
        "CAST('-12345678901234567890.012345678901234' AS DECIMAL(38,15))"
            == BigDecimal::from_str("-12345678901234567890.012345678901234").unwrap(),
        "CAST('-1234567890.1234' AS MONEY)" == BigDecimal::from_str("-1234567890.1234").unwrap(),
        "CAST('-123456.1234' AS SMALLMONEY)" == BigDecimal::from_str("-123456.1234").unwrap(),
    ));
}

#[cfg(feature = "json")]
mod json {
    use super::*;
    use serde_json::Value;
    use sqlx_core::types::Json;

    test_type!(json_value<Json<Value>>(Mssql,
        r#"'123'"# == Json(Value::Number(123.into()))
    ));
}

test_type!(cross_type_tinyint_to_all_signed<i8>(
    Mssql,
    "CAST(0 AS TINYINT)" == 0_i8,
    "CAST(127 AS TINYINT)" == 127_i8,
));

test_type!(cross_type_tinyint_to_i16<i16>(
    Mssql,
    "CAST(0 AS TINYINT)" == 0_i16,
    "CAST(127 AS TINYINT)" == 127_i16,
    "CAST(255 AS TINYINT)" == 255_i16,
));

test_type!(cross_type_tinyint_to_i64<i64>(
    Mssql,
    "CAST(0 AS TINYINT)" == 0_i64,
    "CAST(127 AS TINYINT)" == 127_i64,
    "CAST(255 AS TINYINT)" == 255_i64,
));

test_type!(cross_type_tinyint_to_u16<u16>(
    Mssql,
    "CAST(0 AS TINYINT)" == 0_u16,
    "CAST(127 AS TINYINT)" == 127_u16,
    "CAST(255 AS TINYINT)" == 255_u16,
));

test_type!(cross_type_tinyint_to_u64<u64>(
    Mssql,
    "CAST(0 AS TINYINT)" == 0_u64,
    "CAST(127 AS TINYINT)" == 127_u64,
    "CAST(255 AS TINYINT)" == 255_u64,
));

test_type!(cross_type_smallint_to_i64<i64>(
    Mssql,
    "CAST(-32768 AS SMALLINT)" == -32768_i64,
    "CAST(0 AS SMALLINT)" == 0_i64,
    "CAST(32767 AS SMALLINT)" == 32767_i64,
));

test_type!(cross_type_smallint_to_u16<u16>(
    Mssql,
    "CAST(0 AS SMALLINT)" == 0_u16,
    "CAST(32767 AS SMALLINT)" == 32767_u16,
));

test_type!(cross_type_smallint_to_u64<u64>(
    Mssql,
    "CAST(0 AS SMALLINT)" == 0_u64,
    "CAST(32767 AS SMALLINT)" == 32767_u64,
));

test_type!(cross_type_int_to_i64<i64>(
    Mssql,
    "CAST(-2147483648 AS INT)" == -2147483648_i64,
    "CAST(0 AS INT)" == 0_i64,
    "CAST(2147483647 AS INT)" == 2147483647_i64,
));

test_type!(cross_type_int_to_u32<u32>(
    Mssql,
    "CAST(0 AS INT)" == 0_u32,
    "CAST(2147483647 AS INT)" == 2147483647_u32,
));

test_type!(cross_type_int_to_u64<u64>(
    Mssql,
    "CAST(0 AS INT)" == 0_u64,
    "CAST(2147483647 AS INT)" == 2147483647_u64,
));

test_type!(cross_type_bigint_to_u64<u64>(
    Mssql,
    "CAST(0 AS BIGINT)" == 0_u64,
    "CAST(9223372036854775807 AS BIGINT)" == 9223372036854775807_u64,
));

test_type!(cross_type_decimal_to_integers<i64>(
    Mssql,
    "CAST(123456789 AS DECIMAL(15,0))" == 123456789_i64,
    "CAST(-123456789 AS DECIMAL(15,0))" == -123456789_i64,
    "CAST(0 AS DECIMAL(15,0))" == 0_i64,
));
