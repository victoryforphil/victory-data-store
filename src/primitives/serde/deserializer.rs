use log::trace;
use serde::de::{self, Deserializer, IntoDeserializer, MapAccess, SeqAccess, Visitor};
use std::collections::{HashMap, HashSet};
use tracing::instrument;

use crate::{
    primitives::Primitives,
    topics::{TopicKey, TopicKeyHandle, TopicKeySection},
};
#[allow(unused_imports)]
#[allow(unused_variables)]
pub struct PrimitiveDeserializer<'de> {
    flat_map: &'de HashMap<TopicKeyHandle, Primitives>,
    path: TopicKey,
}

impl<'de> PrimitiveDeserializer<'de> {
    #[instrument(skip_all, name = "PrimitiveDeserializer::new")]
    pub fn new(flat_map: &'de HashMap<TopicKeyHandle, Primitives>) -> Self {
        PrimitiveDeserializer {
            flat_map,
            path: TopicKey::empty(),
        }
    }

    #[instrument(skip_all, name = "PrimitiveDeserializer::get_value")]
    fn get_value(&self) -> Option<&Primitives> {
        let value = self.flat_map.get(&self.path);
        value
    }
    #[instrument(skip_all, name = "PrimitiveDeserializer::enter")]
    fn enter(&mut self, key: &TopicKey) {
        self.path.sections.extend(key.sections.clone());
    }
    #[instrument(skip_all, name = "PrimitiveDeserializer::exit")]
    fn exit(&mut self) {
        trace!("[exit] path: {:?}", self.path);
        self.path.sections.pop();
    }
}

impl<'de, 'a> Deserializer<'de> for &'a mut PrimitiveDeserializer<'de> {
    type Error = de::value::Error;

    #[instrument(skip_all, name = "PrimitiveDeserializer::deserialize_any")]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(primitive) = self.get_value() {
            match primitive {
                Primitives::Boolean(b) => visitor.visit_bool(*b),
                Primitives::Integer(i) => visitor.visit_i64(*i),
                Primitives::Float(f) => visitor.visit_f64(*f),
                Primitives::Text(s) => visitor.visit_str(s),
                Primitives::Blob(blob) => visitor.visit_bytes(blob.data.as_slice()),
                Primitives::List(_) => self.deserialize_seq(visitor),
                Primitives::Unset => visitor.visit_unit(),
                _ => Err(de::Error::custom("Unsupported primitive type")),
            }
        } else {
            self.deserialize_map(visitor)
        }
    }

    #[instrument(skip_all, name = "PrimitiveDeserializer::deserialize_struct")]
    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // Add the struct name to the path
        trace!(
            "[deserialize_struct] path: {:?}, name: {:?}",
            self.path,
            name
        );

        let res = visitor.visit_map(StructAccess {
            de: self,
            fields: fields.to_vec(),
            field_index: 0,
        });

        res
    }
    #[instrument(skip_all, name = "PrimitiveDeserializer::deserialize_seq")]
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let indices = self.collect_sequence_indices(&self.path);
        let seq_access = SeqAccessImpl {
            de: self,
            indices,
            index: 0,
        };
        visitor.visit_seq(seq_access)
    }
    #[instrument(skip_all, name = "PrimitiveDeserializer::deserialize_map")]
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let keys = self.collect_map_keys(&self.path);
        let map_access = MapAccessImpl {
            de: self,
            keys,
            index: 0,
        };
        visitor.visit_map(map_access)
    }

    #[instrument(skip_all, name = "PrimitiveDeserializer::deserialize_bool")]
    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(Primitives::Boolean(b)) = self.get_value() {
            visitor.visit_bool(*b)
        } else {
            Err(de::Error::custom("Expected boolean"))
        }
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(Primitives::Integer(i)) = self.get_value() {
            visitor.visit_i64(*i)
        } else {
            Err(de::Error::custom("Expected integer"))
        }
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(Primitives::Float(f)) = self.get_value() {
            visitor.visit_f64(*f)
        } else {
            Err(de::Error::custom("Expected float"))
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(Primitives::Text(s)) = self.get_value() {
            visitor.visit_string(s.clone())
        } else {
            Err(de::Error::custom("Expected string"))
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(Primitives::Blob(blob)) = self.get_value() {
            visitor.visit_bytes(blob.data.as_slice())
        } else {
            Err(de::Error::custom("Expected bytes"))
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(Primitives::Integer(i)) = self.get_value() {
            visitor.visit_i8(*i as i8)
        } else {
            Err(de::Error::custom("Expected i8"))
        }
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(Primitives::Integer(i)) = self.get_value() {
            visitor.visit_i16(*i as i16)
        } else {
            Err(de::Error::custom("Expected i16"))
        }
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(Primitives::Integer(i)) = self.get_value() {
            visitor.visit_i32(*i as i32)
        } else {
            Err(de::Error::custom(format!(
                "Expected i32, got {:?}",
                self.get_value()
            )))
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(Primitives::Integer(i)) = self.get_value() {
            visitor.visit_u8(*i as u8)
        } else {
            Err(de::Error::custom("Expected u8"))
        }
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(Primitives::Integer(i)) = self.get_value() {
            visitor.visit_u16(*i as u16)
        } else {
            Err(de::Error::custom("Expected u16"))
        }
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(Primitives::Integer(i)) = self.get_value() {
            visitor.visit_u32(*i as u32)
        } else {
            Err(de::Error::custom("Expected u32"))
        }
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(Primitives::Integer(i)) = self.get_value() {
            visitor.visit_u64(*i as u64)
        } else {
            Err(de::Error::custom("Expected u64"))
        }
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(Primitives::Float(f)) = self.get_value() {
            visitor.visit_f32(*f as f32)
        } else {
            Err(de::Error::custom("Expected f32"))
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(Primitives::Text(s)) = self.get_value() {
            if let Some(c) = s.chars().next() {
                visitor.visit_char(c)
            } else {
                Err(de::Error::custom("Expected char"))
            }
        } else {
            Err(de::Error::custom("Expected char"))
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(Primitives::Text(s)) = self.get_value() {
            visitor.visit_str(s)
        } else {
            Err(de::Error::custom("Expected string"))
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(Primitives::Blob(blob)) = self.get_value() {
            visitor.visit_byte_buf(blob.data.clone())
        } else {
            Err(de::Error::custom("Expected byte buffer"))
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(_primitive) = self.get_value() {
            visitor.visit_some(self)
        } else {
            visitor.visit_none()
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(Primitives::Unset) = self.get_value() {
            visitor.visit_unit()
        } else {
            Err(de::Error::custom("Expected unit"))
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(Primitives::Unset) = self.get_value() {
            visitor.visit_unit()
        } else {
            Err(de::Error::custom("Expected unit"))
        }
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(SeqAccessImpl {
            de: self,
            indices: (0..len).collect(),
            index: 0,
        })
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(SeqAccessImpl {
            de: self,
            indices: (0.._len).collect(),
            index: 0,
        })
    }
    #[instrument(skip_all, name = "PrimitiveDeserializer::deserialize_enum")]
    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(EnumAccessImpl { de: self })
    }
    #[instrument(skip_all, name = "PrimitiveDeserializer::deserialize_identifier")]
    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str(&self.path.display_name())
    }

    #[instrument(skip_all, name = "PrimitiveDeserializer::deserialize_ignored_any")]
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}
struct EnumAccessImpl<'a, 'de> {
    de: &'a mut PrimitiveDeserializer<'de>,
}

impl<'de, 'a> de::EnumAccess<'de> for EnumAccessImpl<'a, 'de> {
    type Error = de::value::Error;
    type Variant = VariantAccessImpl<'a, 'de>;
    #[instrument(skip_all, name = "EnumAccessImpl::variant_seed")]
    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        // The enum variant name is expected to be at the current path
        if let Some(Primitives::Text(variant_name)) = self.de.get_value() {
            let val = seed.deserialize(variant_name.as_str().into_deserializer())?;
            Ok((val, VariantAccessImpl { de: self.de }))
        } else {
            Err(de::Error::custom("Expected enum variant as text"))
        }
    }
}

struct VariantAccessImpl<'a, 'de> {
    de: &'a mut PrimitiveDeserializer<'de>,
}

impl<'de, 'a> de::VariantAccess<'de> for VariantAccessImpl<'a, 'de> {
    type Error = de::value::Error;
    #[instrument(skip_all, name = "VariantAccessImpl::unit_variant")]
    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }
    #[instrument(skip_all, name = "VariantAccessImpl::newtype_variant_seed")]
    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        // Enter the path to the variant's associated data
        self.de.enter(&TopicKey::empty()); // Entering empty string to append '/' to path
        let value = seed.deserialize(&mut *self.de)?;
        self.de.exit();
        Ok(value)
    }
    #[instrument(skip_all, name = "VariantAccessImpl::tuple_variant")]
    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.de.enter(&TopicKey::empty()); // Entering empty string to append '/' to path
        let indices = self.de.collect_sequence_indices(&self.de.path);
        let value = visitor.visit_seq(SeqAccessImpl {
            de: self.de,
            indices,
            index: 0,
        })?;
        self.de.exit();
        Ok(value)
    }
    #[instrument(skip_all, name = "VariantAccessImpl::struct_variant")]
    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.de.enter(&TopicKey::empty()); // Entering empty string to append '/' to path
        let value = visitor.visit_map(StructAccess {
            de: self.de,
            fields: _fields.to_vec(),
            field_index: 0,
        })?;
        self.de.exit();
        Ok(value)
    }
}

impl<'de, 'a> PrimitiveDeserializer<'de> {
    #[instrument(skip_all, name = "PrimitiveDeserializer::collect_sequence_indices")]
    fn collect_sequence_indices(&self, prefix: &TopicKey) -> Vec<usize> {
        let mut indices = Vec::new();
        for key in self.flat_map.keys() {
            if key.is_child_of(prefix) {
                let key_idx = key.sections.last().unwrap();
                if let Ok(idx) = key_idx.display_name.parse::<usize>() {
                    trace!("idx: {:?}", idx);
                    indices.push(idx);
                }
            }
        }
        indices.sort_unstable();
        indices.dedup();
        indices
    }
    #[instrument(skip_all, name = "PrimitiveDeserializer::collect_map_keys")]
    fn collect_map_keys(&self, prefix: &TopicKey) -> HashSet<TopicKey> {
        let mut keys = HashSet::new();

        for key in self.flat_map.keys() {
            if key.is_child_of(prefix) {
                let remainder = key
                    .sections
                    .iter()
                    .skip(prefix.sections.len())
                    .take(1)
                    .cloned()
                    .collect();

                let remainder = TopicKey::from_existing(remainder);
                keys.insert(remainder);
            }
        }
        keys
    }
}

// Implement MapAccess for structs
struct StructAccess<'a, 'de> {
    de: &'a mut PrimitiveDeserializer<'de>,
    fields: Vec<&'static str>,
    field_index: usize,
}

impl<'de, 'a> MapAccess<'de> for StructAccess<'a, 'de> {
    type Error = de::value::Error;
    #[instrument(skip_all, name = "StructAccess::next_key_seed")]
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.field_index < self.fields.len() {
            let key = self.fields[self.field_index];
            self.field_index += 1;
            seed.deserialize(key.into_deserializer()).map(Some)
        } else {
            Ok(None)
        }
    }
    #[instrument(skip_all, name = "StructAccess::next_value_seed")]
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let key = self.fields[self.field_index - 1];

        self.de.path.add_suffix_mut(&TopicKey::from_str(key));
        let value = seed.deserialize(&mut *self.de)?;
        self.de.exit();
        Ok(value)
    }
}

// Implement SeqAccess for sequences
struct SeqAccessImpl<'a, 'de> {
    de: &'a mut PrimitiveDeserializer<'de>,
    indices: Vec<usize>,
    index: usize,
}

impl<'de, 'a> SeqAccess<'de> for SeqAccessImpl<'a, 'de> {
    type Error = de::value::Error;
    #[instrument(skip_all, name = "SeqAccessImpl::next_element_seed")]
    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.index < self.indices.len() {
            let idx = self.indices[self.index];
            self.index += 1;
            self.de.enter(&TopicKey::from_str(&idx.to_string()));
            let value = seed.deserialize(&mut *self.de)?;
            self.de.exit();
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
}

// Implement MapAccess for maps
struct MapAccessImpl<'a, 'de> {
    de: &'a mut PrimitiveDeserializer<'de>,
    keys: HashSet<TopicKey>,
    index: usize,
}
impl<'de, 'a> MapAccess<'de> for MapAccessImpl<'a, 'de> {
    type Error = de::value::Error;
    #[instrument(skip_all, name = "MapAccessImpl::next_key_seed")]
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        if let Some(key) = self.keys.iter().nth(self.index) {
            self.index += 1;
            seed.deserialize(key.display_name().into_deserializer())
                .map(Some)
        } else {
            Ok(None)
        }
    }
    #[instrument(skip_all, name = "MapAccessImpl::next_value_seed")]
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        /*
         let key = &self.keys[self.key_index - 1];
        self.de.enter(key);
        let value = seed.deserialize(&mut *self.de)?;
        self.de.exit();
        Ok(value) */
        let key = self.keys.iter().nth(self.index - 1).unwrap();
        self.de.enter(key);

        let value = seed.deserialize(&mut *self.de)?;

        self.de.exit();

        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use crate::topics::TopicKeyProvider;

    use super::*;
    use std::collections::HashMap;

    #[derive(Debug, PartialEq, Deserialize)]
    struct TestSimpleStruct {
        simple_int: i32,
        simple_string: String,
    }
    #[derive(Debug, PartialEq, Deserialize)]
    struct TestComplexStruct {
        complex_int: i32,
        complex_string: String,
        simple_struct: TestSimpleStruct,
    }

    #[test]
    fn test_deserialize_primitives() {
        sensible_env_logger::safe_init!();
        let mut flat_map = HashMap::new();
        flat_map.insert(
            TopicKey::from_str("/_type").handle(),
            Primitives::StructType("TestComplexStruct".to_string()),
        );
        flat_map.insert(
            TopicKey::from_str("/complex_int").handle(),
            Primitives::Integer(42),
        );
        flat_map.insert(
            TopicKey::from_str("/complex_string").handle(),
            Primitives::Text("Test String".to_string()),
        );
        flat_map.insert(
            TopicKey::from_str("/simple_struct/_type").handle(),
            Primitives::StructType("TestSimpleStruct".to_string()),
        );
        flat_map.insert(
            TopicKey::from_str("/simple_struct/simple_int").handle(),
            Primitives::Integer(42),
        );
        flat_map.insert(
            TopicKey::from_str("/simple_struct/simple_string").handle(),
            Primitives::Text("Test String".to_string()),
        );

        let mut deserializer = PrimitiveDeserializer::new(&flat_map);
        let result: TestComplexStruct = Deserialize::deserialize(&mut deserializer).unwrap();

        assert_eq!(
            result,
            TestComplexStruct {
                complex_int: 42,
                complex_string: "Test String".to_string(),
                simple_struct: TestSimpleStruct {
                    simple_int: 42,
                    simple_string: "Test String".to_string(),
                },
            }
        );
    }
}
