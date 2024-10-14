use serde_json::{json, Value};

use super::Primitives;

impl Primitives{
    pub(crate) fn from_value(val: Value) -> Option<Primitives> {
        match val {
            Value::String(s) => Some(Primitives::Text(s)),
            Value::Number(n) => n.as_f64().map(|f| Primitives::Float(f as f64)),
            Value::Bool(b) => Some(Primitives::Boolean(b)),
            Value::Array(arr) => {
                // Here we check the first element to guess the array type
                if let Some(first) = arr.first() {
                    // Check if first can be a float

                    if first.is_string() {
                        let string_array: Vec<String> = arr
                            .into_iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect();
                        let primative_string: Vec<Primitives> = string_array.into_iter().map(Primitives::Text).collect();
                        Some(Primitives::List(primative_string))
                    } else if first.is_number() {
                        let number_array: Vec<f64> = arr.into_iter().filter_map(|v| v.as_f64()).collect();

                        let primative_number: Vec<Primitives> = number_array.into_iter().map(Primitives::Float).collect();
                        Some(Primitives::List(primative_number))
                       
                    } else if first.is_boolean() {
                        let bool_array: Vec<bool> = arr.into_iter().filter_map(|v| v.as_bool()).collect();
                        let primative_bool: Vec<Primitives> = bool_array.into_iter().map(Primitives::Boolean).collect();
                        Some(Primitives::List(primative_bool))
                    } else {
                        None
                    }
                } else {
                    None // Empty array case or mixed types case
                }
            }
            Value::Null => None,
            _ => None, // Covers other cases such as Object, which you might want to handle differently
        }
    }
    pub(crate) fn to_value(&self) -> Value {
        match self {
            Primitives::Float(n) => json!(*n),
            Primitives::Text(s) => json!(*s),
            Primitives::Instant(timepoint) => todo!(),
            Primitives::Duration(timespan) => todo!(),
            Primitives::Integer(i) => json!(*i),
            Primitives::Blob(vic_blob) => todo!(),
            Primitives::Boolean(bol) => json!(*bol),
            Primitives::List(vec) =>{
                Value::Array(vec.iter().map(|v| v.to_value()).collect())
            }
            Primitives::Reference(_) => todo!(),
        }
    }
}