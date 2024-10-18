use super::Primitives;

impl From<bool> for Primitives {
    fn from(value: bool) -> Self {
        Primitives::Boolean(value)
    }
}

impl From<Primitives> for bool {
    fn from(value: Primitives) -> Self {
        match value {
            Primitives::Boolean(v) => v,
            _ => panic!("Cannot convert {:?} to bool", value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bool() {
        let value = true;
        let primitive = Primitives::from(value);
        assert_eq!(primitive, Primitives::Boolean(true));
        let value = false;
        let primitive = Primitives::from(value);
        assert_eq!(primitive, Primitives::Boolean(false));
        let value = Primitives::Boolean(true);
        let bool_value: bool = value.into();
        assert!(bool_value);
        let value = Primitives::Boolean(false);
        let bool_value: bool = value.into();
        assert!(!bool_value);
    }
}
