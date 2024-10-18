pub mod deserializer;
pub mod error;
pub mod serialize;

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use deserializer::PrimitiveDeserializer;
    use log::trace;
    use serde::{Deserialize, Serialize};
    use serialize::to_map;

    use super::*;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestSimpleStruct {
        simple_int: i32,
        simple_str: String,
    }

    impl Default for TestSimpleStruct {
        fn default() -> Self {
            TestSimpleStruct {
                simple_int: 7,
                simple_str: String::from("test_string"),
            }
        }
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]

    struct ComplexStruct {
        a: i32,
        b: TestSimpleStruct,
        c: Vec<i32>,
        d: BTreeMap<String, TestSimpleStruct>,
    }

    #[test]
    fn test_simple_struct() {
        let simple = TestSimpleStruct {
            simple_int: 7,
            simple_str: String::from("test_string"),
        };

        let serialized = to_map(&simple);

        assert!(serialized.is_ok());

        let serialized = serialized.unwrap();
        let mut deserializer = PrimitiveDeserializer::new(&serialized);
        let deserialized: TestSimpleStruct = Deserialize::deserialize(&mut deserializer).unwrap();

        assert_eq!(simple, deserialized);
    }

    #[test]
    fn test_complex_struct() {
        sensible_env_logger::safe_init!();
        let complex = ComplexStruct {
            a: 42,
            b: TestSimpleStruct {
                simple_int: 7,
                simple_str: String::from("test_string"),
            },
            c: vec![1, 2, 3],
            d: {
                let mut map = BTreeMap::new();
                map.insert(
                    String::from("test"),
                    TestSimpleStruct {
                        simple_int: 7,
                        simple_str: String::from("test_string"),
                    },
                );
                map
            },
        };

        let serialized = to_map(&complex);

        assert!(serialized.is_ok());
        let serialized = serialized.unwrap();
        trace!("Test serialized {:#?}", serialized);
        let mut deserializer = PrimitiveDeserializer::new(&serialized);
        let deserialized: ComplexStruct = Deserialize::deserialize(&mut deserializer).unwrap();

        assert_eq!(complex, deserialized);
    }
}
