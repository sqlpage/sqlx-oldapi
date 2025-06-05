use crate::error::BoxDynError;

pub(crate) fn decode_money_bytes(bytes: &[u8]) -> Result<i64, BoxDynError> {
    if bytes.len() != 8 && bytes.len() != 4 {
        return Err(err_protocol!("expected 8/4 bytes for Money, got {}", bytes.len()).into());
    }
    let amount: i64 = if bytes.len() == 8 {
        let amount_h = i32::from_le_bytes(bytes[0..4].try_into()?) as i64;
        let amount_l = u32::from_le_bytes(bytes[4..8].try_into()?) as i64;
        (amount_h << 32) | amount_l
    } else {
        i32::from_le_bytes(bytes.try_into()?) as i64
    };
    Ok(amount)
}
pub(crate) fn decode_numeric_bytes(bytes: &[u8]) -> Result<(i8, u128), BoxDynError> {
    if bytes.is_empty() {
        return Err(err_protocol!("numeric bytes cannot be empty").into());
    }

    let sign = match bytes[0] {
        0 => -1,
        1 => 1,
        other => return Err(err_protocol!("invalid sign byte: 0x{:02x}", other).into()),
    };

    let rest = &bytes[1..];
    if rest.len() > 16 {
        return Err(err_protocol!("numeric value exceeds 16 bytes").into());
    }

    let mut fixed_bytes = [0u8; 16];
    fixed_bytes[..rest.len()].copy_from_slice(rest);
    let amount = u128::from_le_bytes(fixed_bytes);

    Ok((sign, amount))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    // ========== test decode_money_bytes  ==========

    #[test]
    fn test_decode_money_bytes_empty() {
        let bytes: &[u8] = &[];
        let result = decode_money_bytes(bytes);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            err_protocol!("expected 8/4 bytes for Money, got {}", bytes.len()).to_string()
        );
    }

    #[test]
    fn test_decode_money_bytes_invalid_length() {
        let bytes = [0x01, 0x02, 0x03];
        let result = decode_money_bytes(&bytes);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            err_protocol!("expected 8/4 bytes for Money, got {}", bytes.len()).to_string()
        );
    }

    #[test]
    fn test_decode_money_bytes_4bytes_positive() {
        let bytes = [0xd2, 0xe8, 0x95, 0x49]; // 1234561234
        let amount = decode_money_bytes(&bytes).unwrap();
        assert_eq!(amount, 1234561234);
    }

    #[test]
    fn test_decode_money_bytes_4bytes_negative() {
        let bytes = [0x2e, 0x17, 0x6a, 0xb6]; // -1234561234
        let amount = decode_money_bytes(&bytes).unwrap();
        assert_eq!(amount, -1234561234);
    }

    #[test]
    fn test_decode_money_bytes_8bytes_positive() {
        let bytes = [0x1f, 0x01, 0x00, 0x00, 0x22, 0x09, 0xfb, 0x71]; // 1234567891234
        let amount = decode_money_bytes(&bytes).unwrap();
        assert_eq!(amount, 1234567891234);
    }

    #[test]
    fn test_decode_money_bytes_8bytes_negative() {
        let bytes = [0xe0, 0xfe, 0xff, 0xff, 0xde, 0xf6, 0x04, 0x8e]; // -1234567891234
        let amount = decode_money_bytes(&bytes).unwrap();
        assert_eq!(amount, -1234567891234);
    }

    #[test]
    fn test_decode_money_bytes_max_i64() {
        let bytes = [0xff, 0xff, 0xff, 0x7f, 0xff, 0xff, 0xff, 0xff];
        let amount = decode_money_bytes(&bytes).unwrap();
        assert_eq!(amount, i64::MAX);
    }

    #[test]
    fn test_decode_money_bytes_min_i64() {
        let bytes = [0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00];
        let amount = decode_money_bytes(&bytes).unwrap();
        assert_eq!(amount, i64::MIN);
    }

    #[test]
    fn test_decode_money_bytes_all_zero() {
        let bytes = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let amount = decode_money_bytes(&bytes).unwrap();
        assert_eq!(amount, 0);
    }

    // ========== test decode_numeric_bytes  ==========

    #[test]
    fn test_decode_numeric_bytes_empty() {
        let bytes: &[u8] = &[];
        let result = decode_numeric_bytes(bytes);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            err_protocol!("numeric bytes cannot be empty").to_string()
        );
    }

    #[test]
    fn test_decode_numeric_bytes_invalid_sign() {
        let bytes = [0x02, 0x01, 0x02];
        let result = decode_numeric_bytes(&bytes);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            err_protocol!("invalid sign byte: 0x02").to_string()
        );
    }

    #[test]
    fn test_decode_numeric_bytes_overflow() {
        let bytes = vec![0x01; 18]; // 1 sign + 17 data
        let result = decode_numeric_bytes(&bytes);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            err_protocol!("numeric value exceeds 16 bytes").to_string()
        );
    }

    #[test]
    fn test_decode_numeric_bytes_positive() {
        let bytes = [0x01, 0xb9, 0x4a, 0x06, 0x00]; // +412345
        let (sign, amount) = decode_numeric_bytes(&bytes).unwrap();
        assert_eq!(sign, 1);
        assert_eq!(amount, 412345);
    }

    #[test]
    fn test_decode_numeric_bytes_negative() {
        let bytes = [0x00, 0x48, 0x91, 0x0f, 0x86, 0x48, 0x70, 0x00, 0x00]; // -123456789123400
        let (sign, amount) = decode_numeric_bytes(&bytes).unwrap();
        assert_eq!(sign, -1);
        assert_eq!(amount, 123456789123400);
    }

    #[test]
    fn test_decode_numeric_bytes_positive_zero() {
        let bytes = [0x01, 0x00, 0x00, 0x00, 0x00];
        let (sign, amount) = decode_numeric_bytes(&bytes).unwrap();
        assert_eq!(sign, 1);
        assert_eq!(amount, 0);
    }

    #[test]
    fn test_decode_numeric_bytes_negative_zero() {
        let bytes = [0x00, 0x00, 0x00, 0x00, 0x00];
        let (sign, amount) = decode_numeric_bytes(&bytes).unwrap();
        assert_eq!(sign, -1);
        assert_eq!(amount, 0);
    }

    #[test]
    fn test_decode_numeric_bytes_max_u128() {
        let bytes = [0x01] // sign
            .iter()
            .chain([0xff; 16].iter()) // 16 bytes of 0xff
            .cloned()
            .collect::<Vec<u8>>();

        let (sign, amount) = decode_numeric_bytes(&bytes).unwrap();
        assert_eq!(sign, 1);
        assert_eq!(amount, u128::MAX);
    }

    #[test]
    fn test_decode_numeric_bytes_16bytes_value() {
        let bytes = [
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x01,
        ];
        let (sign, amount) = decode_numeric_bytes(&bytes).unwrap();
        assert_eq!(sign, 1);
        assert_eq!(amount, 1 << 120);
    }
}
