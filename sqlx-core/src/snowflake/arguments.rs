use crate::arguments::Arguments;
use crate::encode::{Encode, IsNull};
use crate::snowflake::Snowflake;
use crate::types::Type;

/// Implementation of [`Arguments`] for Snowflake.
#[derive(Debug, Default, Clone)]
pub struct SnowflakeArguments {
    // Store arguments as strings since Snowflake SQL API uses text protocol
    pub(crate) bindings: Vec<String>,
}

/// Implementation of [`ArgumentBuffer`] for Snowflake.
#[derive(Debug, Default)]
pub struct SnowflakeArgumentBuffer {
    pub(crate) buffer: Vec<u8>,
}

impl SnowflakeArguments {
    pub fn new() -> Self {
        Self {
            bindings: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    pub(crate) fn get(&self, index: usize) -> Option<&String> {
        self.bindings.get(index)
    }
}

impl<'q> Arguments<'q> for SnowflakeArguments {
    type Database = Snowflake;

    fn reserve(&mut self, additional: usize, _size: usize) {
        self.bindings.reserve(additional);
    }

    fn add<T>(&mut self, value: T)
    where
        T: 'q + Encode<'q, Self::Database> + Type<Self::Database>,
    {
        let mut buffer = SnowflakeArgumentBuffer::new();
        let is_null = value.encode_by_ref(&mut buffer);

        match is_null {
            IsNull::No => {
                // Convert the encoded bytes to a string
                let binding = String::from_utf8_lossy(&buffer.buffer).into_owned();
                self.bindings.push(binding);
            }
            IsNull::Yes => {
                self.bindings.push("NULL".to_string());
            }
        }
    }
}

impl SnowflakeArgumentBuffer {
    pub fn new() -> Self {
        Self::default()
    }
}
