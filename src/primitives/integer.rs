use super::Primitives;

impl From<u64> for Primitives {
    fn from(value: u64) -> Self {
        Primitives::Integer(value as i64)
    }
}

impl From<u32> for Primitives {
    fn from(value: u32) -> Self {
        Primitives::Integer(value as i64)
    }
}

impl From<i64> for Primitives {
    fn from(value: i64) -> Self {
        Primitives::Integer(value)
    }
}

impl From<i32> for Primitives {
    fn from(value: i32) -> Self {
        Primitives::Integer(value as i64)
    }
}

// I8
impl From<i8> for Primitives {
    fn from(value: i8) -> Self {
        Primitives::Integer(value as i64)
    }
}

impl From<Primitives> for i8 {
    fn from(value: Primitives) -> Self {
        match value {
            Primitives::Integer(v) => v as i8,
            _ => panic!("Cannot convert {:?} to i8", value),
        }
    }
}

// i16
impl From<i16> for Primitives {
    fn from(value: i16) -> Self {
        Primitives::Integer(value as i64)
    }
}

impl From<Primitives> for i16 {
    fn from(value: Primitives) -> Self {
        match value {
            Primitives::Integer(v) => v as i16,
            _ => panic!("Cannot convert {:?} to i16", value),
        }
    }
}

// u8
impl From<u8> for Primitives {
    fn from(value: u8) -> Self {
        Primitives::Integer(value as i64)
    }
}

impl From<Primitives> for u8 {
    fn from(value: Primitives) -> Self {
        match value {
            Primitives::Integer(v) => v as u8,
            _ => panic!("Cannot convert {:?} to u8", value),
        }
    }
}

// u16

impl From<u16> for Primitives {
    fn from(value: u16) -> Self {
        Primitives::Integer(value as i64)
    }
}

impl From<Primitives> for u16 {
    fn from(value: Primitives) -> Self {
        match value {
            Primitives::Integer(v) => v as u16,
            _ => panic!("Cannot convert {:?} to u16", value),
        }
    }
}

impl From<Primitives> for u64 {
    fn from(value: Primitives) -> Self {
        match value {
            Primitives::Integer(v) => v as u64,
            _ => panic!("Cannot convert {:?} to u64", value),
        }
    }
}

impl From<Primitives> for u32 {
    fn from(value: Primitives) -> Self {
        match value {
            Primitives::Integer(v) => v as u32,
            _ => panic!("Cannot convert {:?} to u32", value),
        }
    }
}

impl From<Primitives> for i64 {
    fn from(value: Primitives) -> Self {
        match value {
            Primitives::Integer(v) => v,
            _ => panic!("Cannot convert {:?} to i64", value),
        }
    }
}

impl From<Primitives> for i32 {
    fn from(value: Primitives) -> Self {
        match value {
            Primitives::Integer(v) => v as i32,
            _ => panic!("Cannot convert {:?} to i32", value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversion_unsigned() {
        let value = Primitives::Integer(42);
        let u64_value: u64 = value.clone().into();
        assert_eq!(u64_value, 42);

        let u32_value: u32 = value.clone().into();
        assert_eq!(u32_value, 42);
    }

    #[test]
    fn test_conversion_signed() {
        let value = Primitives::Integer(-42_i64);
        let i64_value: i64 = value.clone().into();
        assert_eq!(i64_value, -42);

        let i32_value: i32 = value.into();
        assert_eq!(i32_value, -42);
    }

    #[test]
    fn test_conversion_panic() {
        let value = Primitives::Float(42.0);
        let result = std::panic::catch_unwind(|| {
            let _: u64 = value.into();
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_conversion_from() {
        let value: u64 = 42;
        let primitive: Primitives = value.into();
        assert_eq!(primitive, Primitives::Integer(42));

        let value: i64 = -42;
        let primitive: Primitives = value.into();
        assert_eq!(primitive, Primitives::Integer(-42));
    }
}
