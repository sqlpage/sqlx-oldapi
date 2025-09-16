use bitflags::bitflags;
use bytes::{Buf, Bytes};
use encoding_rs::Encoding;

use crate::encode::{Encode, IsNull};
use crate::error::Error;
use crate::mssql::Mssql;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    #[cfg_attr(feature = "offline", derive(serde::Serialize, serde::Deserialize))]
    pub(crate) struct CollationFlags: u8 {
        const IGNORE_CASE = (1 << 0);
        const IGNORE_ACCENT = (1 << 1);
        const IGNORE_WIDTH = (1 << 2);
        const IGNORE_KANA = (1 << 3);
        const BINARY = (1 << 4);
        const BINARY2 = (1 << 5);
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "offline", derive(serde::Serialize, serde::Deserialize))]
pub(crate) struct Collation {
    pub(crate) locale: u32,
    pub(crate) flags: CollationFlags,
    pub(crate) sort: u8,
    pub(crate) version: u8,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "offline", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub(crate) enum DataType {
    // fixed-length data types
    // https://docs.microsoft.com/en-us/openspecs/sql_server_protocols/ms-sstds/d33ef17b-7e53-4380-ad11-2ba42c8dda8d
    Null = 0x1f,
    TinyInt = 0x30,
    Bit = 0x32,
    SmallInt = 0x34,
    Int = 0x38,
    SmallDateTime = 0x3a,
    Real = 0x3b,
    Money = 0x3c,
    DateTime = 0x3d,
    Float = 0x3e,
    SmallMoney = 0x7a,
    BigInt = 0x7f,

    // variable-length data types
    // https://docs.microsoft.com/en-us/openspecs/windows_protocols/ms-tds/ce3183a6-9d89-47e8-a02f-de5a1a1303de

    // byte length
    Guid = 0x24,
    IntN = 0x26,
    Decimal = 0x37, // legacy
    Numeric = 0x3f, // legacy
    BitN = 0x68,
    DecimalN = 0x6a,
    NumericN = 0x6c,
    FloatN = 0x6d,
    MoneyN = 0x6e,
    DateTimeN = 0x6f,
    DateN = 0x28,
    TimeN = 0x29,
    DateTime2N = 0x2a,
    DateTimeOffsetN = 0x2b,
    Char = 0x2f,      // legacy
    VarChar = 0x27,   // legacy
    Binary = 0x2d,    // legacy
    VarBinary = 0x25, // legacy

    // short length
    BigVarBinary = 0xa5,
    BigVarChar = 0xa7,
    BigBinary = 0xad,
    BigChar = 0xaf,
    NVarChar = 0xe7,
    NChar = 0xef,
    Xml = 0xf1,
    UserDefined = 0xf0,

    // long length
    Text = 0x23,
    Image = 0x22,
    NText = 0x63,
    Variant = 0x62,
}

// http://msdn.microsoft.com/en-us/library/dd358284.aspx
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "offline", derive(serde::Serialize, serde::Deserialize))]
pub(crate) struct TypeInfo {
    pub(crate) ty: DataType,
    pub(crate) size: u32,
    pub(crate) scale: u8,
    pub(crate) precision: u8,
    pub(crate) collation: Option<Collation>,
}

impl TypeInfo {
    pub(crate) const fn new(ty: DataType, size: u32) -> Self {
        Self {
            ty,
            size,
            scale: 0,
            precision: 0,
            collation: None,
        }
    }

    pub(crate) fn encoding(&self) -> Result<&'static Encoding, Error> {
        match self.ty {
            DataType::NChar | DataType::NVarChar => Ok(encoding_rs::UTF_16LE),

            DataType::VarChar | DataType::Char | DataType::BigChar | DataType::BigVarChar => {
                // unwrap: impossible to unwrap here, collation will be set
                // The locale is a windows LCID (locale identifier), which maps to an encoding
                let lcid = self.collation.unwrap().locale;
                // see https://github.com/lovasoa/lcid-to-codepage
                Ok(match lcid {
                    // Arabic locales
                    0x0401 | 0x3801 | 0x3C01 | 0x1401 | 0x0C01 | 0x0801 | 0x2C01 | 0x3401
                    | 0x3001 | 0x1001 | 0x1801 | 0x2001 | 0x4001 | 0x2801 | 0x1C01 | 0x2401
                    | 0x0429 | 0x0492 | 0x0846 | 0x048C | 0x0859 | 0x0420 | 0x0820 | 0x045F
                    | 0x0480 => encoding_rs::WINDOWS_1256,

                    // Chinese locales
                    0x0804 | 0x50804 | 0x20804 | 0x1004 | 0x51004 | 0x21004 => encoding_rs::GBK,
                    0x0C04 | 0x40C04 | 0x1404 | 0x41404 | 0x21404 | 0x0404 | 0x30404 | 0x40404 => {
                        encoding_rs::BIG5
                    }

                    // Cyrillic locales
                    0x082C | 0x046D | 0x0423 | 0x0402 | 0x201A | 0x0440 | 0x042F | 0x0450
                    | 0x0419 | 0x0819 | 0x0485 | 0x0428 | 0x0444 | 0x0422 | 0x0843 | 0x281A
                    | 0x1C1A | 0x301A => encoding_rs::WINDOWS_1251,

                    // Central European locales
                    0x141A | 0x0405 | 0x041A | 0x101A | 0x040E | 0x1040E | 0x0415 | 0x0418
                    | 0x0818 | 0x041B | 0x0424 | 0x041C | 0x241A | 0x181A | 0x2C1A | 0x0442 => {
                        encoding_rs::WINDOWS_1250
                    }

                    // Baltic locales
                    0x0425 | 0x0427 | 0x0426 => encoding_rs::WINDOWS_1257,

                    // Greek
                    0x0408 => encoding_rs::WINDOWS_1253,

                    // Hebrew
                    0x040D => encoding_rs::WINDOWS_1255,

                    // Japanese
                    0x0411 | 0x40411 => encoding_rs::SHIFT_JIS,

                    // Korean
                    0x0412 => encoding_rs::EUC_KR,

                    // Thai
                    0x041E => encoding_rs::WINDOWS_874,

                    // Turkish
                    0x042C | 0x041F | 0x0443 => encoding_rs::WINDOWS_1254,

                    // Vietnamese
                    0x042A => encoding_rs::WINDOWS_1258,

                    // Western European/US locales - default for unhandled LCIDs
                    _ => encoding_rs::WINDOWS_1252,
                })
            }

            _ => {
                // default to UTF-8 for anything
                // else coming in here
                Ok(encoding_rs::UTF_8)
            }
        }
    }

    // reads a TYPE_INFO from the buffer
    pub(crate) fn get(buf: &mut Bytes) -> Result<Self, Error> {
        let ty = DataType::get(buf)?;

        Ok(match ty {
            DataType::Null => Self::new(ty, 0),

            DataType::TinyInt | DataType::Bit => Self::new(ty, 1),

            DataType::SmallInt => Self::new(ty, 2),

            DataType::Int | DataType::SmallDateTime | DataType::Real | DataType::SmallMoney => {
                Self::new(ty, 4)
            }

            DataType::BigInt | DataType::Money | DataType::DateTime | DataType::Float => {
                Self::new(ty, 8)
            }

            DataType::DateN => Self::new(ty, 3),

            DataType::TimeN | DataType::DateTime2N | DataType::DateTimeOffsetN => {
                let scale = buf.get_u8();

                let mut size = match scale {
                    0..=2 => 3,
                    3..=4 => 4,
                    5..=7 => 5,

                    scale => {
                        return Err(err_protocol!("invalid scale {} for type {:?}", scale, ty));
                    }
                };

                match ty {
                    DataType::DateTime2N => {
                        size += 3;
                    }

                    DataType::DateTimeOffsetN => {
                        size += 5;
                    }

                    _ => {}
                }

                Self {
                    scale,
                    size,
                    ty,
                    precision: 0,
                    collation: None,
                }
            }

            DataType::Guid
            | DataType::IntN
            | DataType::BitN
            | DataType::FloatN
            | DataType::MoneyN
            | DataType::DateTimeN
            | DataType::Char
            | DataType::VarChar
            | DataType::Binary
            | DataType::VarBinary => Self::new(ty, buf.get_u8() as u32),

            DataType::Decimal | DataType::Numeric | DataType::DecimalN | DataType::NumericN => {
                let size = buf.get_u8() as u32;
                let precision = buf.get_u8();
                let scale = buf.get_u8();

                Self {
                    size,
                    precision,
                    scale,
                    ty,
                    collation: None,
                }
            }

            DataType::BigVarBinary | DataType::BigBinary => Self::new(ty, buf.get_u16_le() as u32),

            DataType::BigVarChar | DataType::BigChar | DataType::NVarChar | DataType::NChar => {
                let size = buf.get_u16_le() as u32;
                let collation = Collation::get(buf);

                Self {
                    ty,
                    size,
                    collation: Some(collation),
                    scale: 0,
                    precision: 0,
                }
            }
            _ => {
                return Err(err_protocol!("unsupported data type {:?}", ty));
            }
        })
    }

    // writes a TYPE_INFO to the buffer
    pub(crate) fn put(&self, buf: &mut Vec<u8>) {
        buf.push(self.ty as u8);

        match self.ty {
            DataType::Null
            | DataType::TinyInt
            | DataType::Bit
            | DataType::SmallInt
            | DataType::Int
            | DataType::SmallDateTime
            | DataType::Real
            | DataType::SmallMoney
            | DataType::BigInt
            | DataType::Money
            | DataType::DateTime
            | DataType::Float => {
                // nothing to do
            }

            DataType::TimeN | DataType::DateTime2N | DataType::DateTimeOffsetN => {
                buf.push(self.scale);
            }

            DataType::Guid
            | DataType::IntN
            | DataType::BitN
            | DataType::FloatN
            | DataType::MoneyN
            | DataType::DateTimeN
            | DataType::DateN
            | DataType::Char
            | DataType::VarChar
            | DataType::Binary
            | DataType::VarBinary => {
                buf.push(u8::try_from(self.size).unwrap());
            }

            DataType::Decimal | DataType::Numeric | DataType::DecimalN | DataType::NumericN => {
                buf.push(u8::try_from(self.size).unwrap());
                buf.push(self.precision);
                buf.push(self.scale);
            }

            DataType::BigVarBinary | DataType::BigBinary => {
                buf.extend(&(u16::try_from(self.size).unwrap().to_le_bytes()));
            }

            DataType::BigVarChar | DataType::BigChar | DataType::NVarChar | DataType::NChar => {
                let short_size = u16::try_from(self.size).unwrap();
                buf.extend(&(short_size.to_le_bytes()));

                if let Some(collation) = &self.collation {
                    collation.put(buf);
                } else {
                    buf.extend(&0_u32.to_le_bytes());
                    buf.push(0);
                }
            }
            DataType::Xml
            | DataType::UserDefined
            | DataType::Text
            | DataType::Image
            | DataType::NText
            | DataType::Variant => {
                log::error!("Unsupported mssql data type argument writing {:?}", self.ty);
            }
        }
    }

    pub(crate) fn is_null(&self) -> bool {
        matches!(self.ty, DataType::Null)
    }

    pub(crate) fn get_value(&self, buf: &mut Bytes) -> Option<Bytes> {
        match self.ty {
            DataType::Null
            | DataType::TinyInt
            | DataType::Bit
            | DataType::SmallInt
            | DataType::Int
            | DataType::SmallDateTime
            | DataType::Real
            | DataType::Money
            | DataType::DateTime
            | DataType::Float
            | DataType::SmallMoney
            | DataType::BigInt => Some(buf.split_to(self.size as usize)),

            DataType::Guid
            | DataType::IntN
            | DataType::Decimal
            | DataType::Numeric
            | DataType::BitN
            | DataType::DecimalN
            | DataType::NumericN
            | DataType::FloatN
            | DataType::MoneyN
            | DataType::DateN
            | DataType::DateTimeN
            | DataType::TimeN
            | DataType::DateTime2N
            | DataType::DateTimeOffsetN => {
                let size = buf.get_u8();

                if size == 0 || size == 0xFF {
                    None
                } else {
                    Some(buf.split_to(size as usize))
                }
            }

            DataType::Char | DataType::VarChar | DataType::Binary | DataType::VarBinary => {
                let size = buf.get_u8();
                if size == 0xFF {
                    None
                } else {
                    Some(buf.split_to(size as usize))
                }
            }

            DataType::BigVarBinary
            | DataType::BigVarChar
            | DataType::BigBinary
            | DataType::BigChar
            | DataType::NVarChar
            | DataType::NChar
            | DataType::Xml
            | DataType::UserDefined => {
                if self.size == 0xffff {
                    self.get_big_blob(buf)
                } else {
                    let size = buf.get_u16_le();
                    if size == 0xFF_FF {
                        None
                    } else {
                        Some(buf.split_to(size as usize))
                    }
                }
            }

            DataType::Text | DataType::Image | DataType::NText | DataType::Variant => {
                let size = buf.get_u32_le();

                if size == 0xFFFF_FFFF {
                    None
                } else {
                    Some(buf.split_to(size as usize))
                }
            }
        }
    }

    pub(crate) fn get_big_blob(&self, buf: &mut Bytes) -> Option<Bytes> {
        // Unknown size, length-prefixed blobs
        let len = buf.get_u64_le();

        let mut data = match len {
            // NULL
            0xffffffffffffffff => return None,
            // Unknown size
            0xfffffffffffffffe => Vec::new(),
            // Known size
            _ => Vec::with_capacity(usize::try_from(len).unwrap()),
        };

        loop {
            let chunk_size = buf.get_u32_le() as usize;

            if chunk_size == 0 {
                break; // found a sentinel, we're done
            }
            let chunk = buf.split_to(chunk_size);
            data.extend_from_slice(&chunk);
        }

        Some(data.into())
    }

    pub(crate) fn put_value<'q, T: Encode<'q, Mssql>>(&self, buf: &mut Vec<u8>, value: T) {
        match self.ty {
            DataType::Null
            | DataType::TinyInt
            | DataType::Bit
            | DataType::SmallInt
            | DataType::Int
            | DataType::SmallDateTime
            | DataType::Real
            | DataType::Money
            | DataType::DateTime
            | DataType::DateN
            | DataType::Float
            | DataType::SmallMoney
            | DataType::BigInt => {
                self.put_fixed_value(buf, value);
            }

            DataType::Guid
            | DataType::IntN
            | DataType::Decimal
            | DataType::Numeric
            | DataType::BitN
            | DataType::DecimalN
            | DataType::NumericN
            | DataType::FloatN
            | DataType::MoneyN
            | DataType::DateTimeN
            | DataType::TimeN
            | DataType::DateTime2N
            | DataType::DateTimeOffsetN
            | DataType::Char
            | DataType::VarChar
            | DataType::Binary
            | DataType::VarBinary => {
                self.put_byte_len_value(buf, value);
            }

            DataType::BigVarBinary
            | DataType::BigVarChar
            | DataType::BigBinary
            | DataType::BigChar
            | DataType::NVarChar
            | DataType::NChar
            | DataType::Xml
            | DataType::UserDefined => {
                if self.size == 0xFF_FF {
                    self.put_big_blob(buf, value);
                } else {
                    self.put_short_len_value(buf, value);
                }
            }

            DataType::Text | DataType::Image | DataType::NText | DataType::Variant => {
                self.put_long_len_value(buf, value);
            }
        }
    }

    pub(crate) fn put_fixed_value<'q, T: Encode<'q, Mssql>>(&self, buf: &mut Vec<u8>, value: T) {
        let _ = value.encode(buf);
    }

    pub(crate) fn put_byte_len_value<'q, T: Encode<'q, Mssql>>(&self, buf: &mut Vec<u8>, value: T) {
        let offset = buf.len();
        buf.push(0);

        let size = if let IsNull::Yes = value.encode(buf) {
            0xFF
        } else {
            u8::try_from(buf.len() - offset - 1).unwrap()
        };

        buf[offset] = size;
    }

    pub(crate) fn put_short_len_value<'q, T: Encode<'q, Mssql>>(
        &self,
        buf: &mut Vec<u8>,
        value: T,
    ) {
        let offset = buf.len();
        buf.extend(&0_u16.to_le_bytes());

        let size = if let IsNull::Yes = value.encode(buf) {
            0xFFFF
        } else {
            u16::try_from(buf.len() - offset - 2).unwrap()
        };

        buf[offset..(offset + 2)].copy_from_slice(&size.to_le_bytes());
    }

    pub(crate) fn put_big_blob<'q, T: Encode<'q, Mssql>>(&self, buf: &mut Vec<u8>, value: T) {
        // Multiple chunks, are not supported yet
        let start_of_value = buf.len();
        buf.extend(&0_u64.to_le_bytes()); // total blob length
        let start_of_chunk = buf.len();
        buf.extend(&0_u32.to_le_bytes()); // chunk length
        let start_of_bytes = buf.len();

        let size = if let IsNull::Yes = value.encode(buf) {
            unreachable!("put_big_blob should never be called with NULL value");
        } else {
            u32::try_from(buf.len() - start_of_bytes).expect("blobs >4GB not supported")
        };

        buf[start_of_value..(start_of_value + 4)].copy_from_slice(&size.to_le_bytes());
        buf[start_of_chunk..(start_of_chunk + 4)].copy_from_slice(&size.to_le_bytes());
        buf.extend(&0_u32.to_le_bytes()); // end of chunks marker
    }

    pub(crate) fn put_long_len_value<'q, T: Encode<'q, Mssql>>(&self, buf: &mut Vec<u8>, value: T) {
        let offset = buf.len();
        buf.extend(&0_u32.to_le_bytes());

        let size = if let IsNull::Yes = value.encode(buf) {
            0xFFFF_FFFF
        } else {
            u32::try_from(buf.len() - offset - 4).unwrap()
        };

        buf[offset..(offset + 4)].copy_from_slice(&size.to_le_bytes());
    }

    pub(crate) fn name(&self) -> &'static str {
        match self.ty {
            DataType::Null => "NULL",
            DataType::TinyInt => "TINYINT",
            DataType::SmallInt => "SMALLINT",
            DataType::Int => "INT",
            DataType::BigInt => "BIGINT",
            DataType::Real => "REAL",
            DataType::Float => "FLOAT",

            DataType::IntN => match self.size {
                1 => "TINYINT",
                2 => "SMALLINT",
                4 => "INT",
                8 => "BIGINT",

                n => unreachable!("invalid size {} for int", n),
            },

            DataType::FloatN => match self.size {
                4 => "REAL",
                8 => "FLOAT",

                n => unreachable!("invalid size {} for float", n),
            },

            DataType::VarChar => "VARCHAR",
            DataType::NVarChar => "NVARCHAR",
            DataType::BigVarChar => "BIGVARCHAR",
            DataType::Char => "CHAR",
            DataType::BigChar => "BIGCHAR",
            DataType::NChar => "NCHAR",
            DataType::VarBinary => "VARBINARY",
            DataType::BigVarBinary => "BIGVARBINARY",
            DataType::Binary => "BINARY",
            DataType::BigBinary => "BIGBINARY",
            DataType::DateN => "DATE",
            DataType::DateTimeN => "DATETIME",
            DataType::DateTime2N => "DATETIME2",
            DataType::DateTimeOffsetN => "DATETIMEOFFSET",

            DataType::Bit => "BIT",
            DataType::SmallDateTime => "SMALLDATETIME",
            DataType::Money => "MONEY",
            DataType::DateTime => "DATETIME",
            DataType::SmallMoney => "SMALLMONEY",
            DataType::Guid => "UNIQUEIDENTIFIER",
            DataType::Decimal => "DECIMAL",
            DataType::Numeric => "NUMERIC",
            DataType::BitN => "BIT",
            DataType::DecimalN => "DECIMAL",
            DataType::NumericN => "NUMERIC",
            DataType::MoneyN => "MONEY",
            DataType::TimeN => "TIME",
            DataType::Xml => "XML",
            DataType::UserDefined => "USER_DEFINED_TYPE",
            DataType::Text => "TEXT",
            DataType::Image => "IMAGE",
            DataType::NText => "NTEXT",
            DataType::Variant => "SQL_VARIANT",
        }
    }

    pub(crate) fn fmt(&self, s: &mut String) {
        match self.ty {
            DataType::Null => s.push_str("nvarchar(1)"),
            DataType::TinyInt => s.push_str("tinyint"),
            DataType::SmallInt => s.push_str("smallint"),
            DataType::Int => s.push_str("int"),
            DataType::BigInt => s.push_str("bigint"),
            DataType::Real => s.push_str("real"),
            DataType::Float => s.push_str("float"),
            DataType::Bit => s.push_str("bit"),

            DataType::IntN => s.push_str(match self.size {
                1 => "tinyint",
                2 => "smallint",
                4 => "int",
                8 => "bigint",

                n => unreachable!("invalid size {} for int", n),
            }),

            DataType::FloatN => s.push_str(match self.size {
                4 => "real",
                8 => "float",

                n => unreachable!("invalid size {} for float", n),
            }),

            DataType::NVarChar | DataType::NChar => {
                // name
                s.push_str(match self.ty {
                    DataType::NVarChar => "nvarchar",
                    DataType::NChar => "nchar",
                    _ => unreachable!(),
                });

                if self.size == 0xFF_FF {
                    s.push_str("(max)");
                } else {
                    s.push('(');
                    let size_in_characters = self.size / 2;
                    s.push_str(itoa::Buffer::new().format(size_in_characters));
                    s.push(')');
                }
            }

            DataType::VarChar
            | DataType::BigVarChar
            | DataType::Char
            | DataType::BigChar
            | DataType::VarBinary
            | DataType::BigVarBinary
            | DataType::Binary
            | DataType::BigBinary => {
                // name
                s.push_str(match self.ty {
                    DataType::VarChar => "varchar",
                    DataType::BigVarChar => "bigvarchar",
                    DataType::Char => "char",
                    DataType::BigChar => "bigchar",
                    DataType::VarBinary => "varbinary",
                    DataType::BigVarBinary => "varbinary",
                    DataType::Binary => "binary",
                    DataType::BigBinary => "binary",
                    _ => unreachable!(),
                });

                if self.size == 0xFF_FF {
                    s.push_str("(max)");
                } else {
                    s.push('(');
                    s.push_str(itoa::Buffer::new().format(self.size));
                    s.push(')');
                }
            }

            DataType::BitN => {
                s.push_str("bit");
            }

            DataType::DateN => {
                s.push_str("date");
            }

            DataType::DateTime | DataType::DateTimeN => {
                s.push_str("datetime");
            }

            DataType::DateTime2N => {
                s.push_str("datetime2(");
                s.push_str(itoa::Buffer::new().format(self.scale));
                s.push(')');
            }

            DataType::DateTimeOffsetN => {
                s.push_str("datetimeoffset(");
                s.push_str(itoa::Buffer::new().format(self.scale));
                s.push(')');
            }

            DataType::TimeN => {
                s.push_str("time(");
                s.push_str(itoa::Buffer::new().format(self.scale));
                s.push(')');
            }
            DataType::SmallDateTime => s.push_str("smalldatetime"),
            DataType::Money => s.push_str("money"),
            DataType::SmallMoney => s.push_str("smallmoney"),
            DataType::Guid => s.push_str("uniqueidentifier"),
            DataType::Decimal => s.push_str("decimal"),
            DataType::Numeric => s.push_str("numeric"),
            DataType::DecimalN => {
                s.push_str("decimal(");
                s.push_str(itoa::Buffer::new().format(self.precision));
                s.push(',');
                s.push_str(itoa::Buffer::new().format(self.scale));
                s.push(')');
            }
            DataType::NumericN => {
                s.push_str("numeric(");
                s.push_str(itoa::Buffer::new().format(self.precision));
                s.push(',');
                s.push_str(itoa::Buffer::new().format(self.scale));
                s.push(')');
            }
            DataType::MoneyN => {
                s.push_str("money(");
                s.push_str(itoa::Buffer::new().format(self.scale));
                s.push(')');
            }
            DataType::Xml => s.push_str("xml"),
            DataType::UserDefined => s.push_str("user_defined_type"),
            DataType::Text => s.push_str("text"),
            DataType::Image => s.push_str("image"),
            DataType::NText => s.push_str("ntext"),
            DataType::Variant => s.push_str("sql_variant"),
        }
    }
}

impl DataType {
    pub(crate) fn get(buf: &mut Bytes) -> Result<Self, Error> {
        Ok(match buf.get_u8() {
            0x1f => DataType::Null,
            0x30 => DataType::TinyInt,
            0x32 => DataType::Bit,
            0x34 => DataType::SmallInt,
            0x38 => DataType::Int,
            0x3a => DataType::SmallDateTime,
            0x3b => DataType::Real,
            0x3c => DataType::Money,
            0x3d => DataType::DateTime,
            0x3e => DataType::Float,
            0x7a => DataType::SmallMoney,
            0x7f => DataType::BigInt,
            0x24 => DataType::Guid,
            0x26 => DataType::IntN,
            0x37 => DataType::Decimal,
            0x3f => DataType::Numeric,
            0x68 => DataType::BitN,
            0x6a => DataType::DecimalN,
            0x6c => DataType::NumericN,
            0x6d => DataType::FloatN,
            0x6e => DataType::MoneyN,
            0x6f => DataType::DateTimeN,
            0x28 => DataType::DateN,
            0x29 => DataType::TimeN,
            0x2a => DataType::DateTime2N,
            0x2b => DataType::DateTimeOffsetN,
            0x2f => DataType::Char,
            0x27 => DataType::VarChar,
            0x2d => DataType::Binary,
            0x25 => DataType::VarBinary,
            0xa5 => DataType::BigVarBinary,
            0xa7 => DataType::BigVarChar,
            0xad => DataType::BigBinary,
            0xaf => DataType::BigChar,
            0xe7 => DataType::NVarChar,
            0xef => DataType::NChar,
            0xf1 => DataType::Xml,
            0xf0 => DataType::UserDefined,
            0x23 => DataType::Text,
            0x22 => DataType::Image,
            0x63 => DataType::NText,
            0x62 => DataType::Variant,

            ty => {
                return Err(err_protocol!("unknown data type 0x{:02x}", ty));
            }
        })
    }
}

impl Collation {
    pub(crate) fn get(buf: &mut Bytes) -> Collation {
        let locale_sort_version = buf.get_u32_le();
        let locale = locale_sort_version & 0xfffff;
        let flags = CollationFlags::from_bits_truncate(((locale_sort_version >> 20) & 0xFF) as u8);
        let version = (locale_sort_version >> 28) as u8;
        let sort = buf.get_u8();

        Collation {
            locale,
            flags,
            sort,
            version,
        }
    }

    pub(crate) fn put(&self, buf: &mut Vec<u8>) {
        let locale_sort_version =
            self.locale | ((self.flags.bits() as u32) << 20) | ((self.version as u32) << 28);

        buf.extend(&locale_sort_version.to_le_bytes());
        buf.push(self.sort);
    }
}

#[test]
fn test_get() {
    #[rustfmt::skip]
    let mut buf = Bytes::from_static(&[
        0x26, 4, 4, 1, 0, 0, 0, 0xfe, 0, 0, 0xe0, 0, 0, 0, 0, 0, 0, 0, 0, 0
    ]);

    let type_info = TypeInfo::get(&mut buf).unwrap();
    assert_eq!(type_info, TypeInfo::new(DataType::IntN, 4));
}
