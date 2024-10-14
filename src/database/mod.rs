use crate::{
    buckets::{Bucket, BucketHandle},
    datapoints::Datapoint,
    primitives::Primitives,
    topics::{TopicKey, TopicKeyHandle, TopicKeyProvider},
};
use std::collections::{BTreeMap, BTreeSet};
use json_unflattening::flattening::flatten;
use thiserror::Error;
use victory_time_rs::Timepoint;
#[derive(Debug, Clone)]
pub struct Datastore {
    buckets: BTreeMap<TopicKeyHandle, BucketHandle>,
}
#[derive(Error, Debug)]
pub enum DatastoreError {
    #[error("Generic Datastore Error: {0}")]
    Generic(String),
    #[error("Bucket not found for topic {0}")]
    BucketNotFound(TopicKey),
}

impl Datastore {
    pub fn new() -> Datastore {
        Datastore {
            buckets: BTreeMap::new(),
        }
    }

    pub fn create_bucket<T: TopicKeyProvider>(&mut self, topic: &T) {
        if !self.buckets.contains_key(&topic.handle()) {
            let bucket = Bucket::new(topic);
            self.buckets.insert(topic.handle().clone(), bucket);
        }
    }

    pub fn get_bucket<T: TopicKeyProvider>(
        &self,
        topic: &T,
    ) -> Result<BucketHandle, DatastoreError> {
        self.buckets
            .get(&topic.handle())
            .cloned()
            .ok_or_else(|| DatastoreError::BucketNotFound(topic.key().clone()))
    }

    pub fn add_primitive<T: TopicKeyProvider>(
        &mut self,
        topic: &T,
        time: Timepoint,
        value: Primitives,
    ) {
        let topic = topic.handle();
        let time = time.clone();
        if !self.buckets.contains_key(&topic) {
            let bucket = Bucket::new(&topic);
            self.buckets.insert(topic.clone(), bucket);
        }

        let bucket = self.buckets.get_mut(&topic).unwrap();

        bucket.write().unwrap().add_primitive(time, value);
    }

    pub fn add_json<T: TopicKeyProvider>(
        &mut self,
        topic: &T,
        time: Timepoint,
        value: serde_json::Value,
    ) {

        let out = flatten(&value).unwrap();
        for (key, val) in out.iter() {
            if let Some(supported_type) = Primitives::from_value(val.clone()) {
                let suffix = TopicKey::from_str(key);
                let new_key = topic.key().add_suffix(suffix);
                self.add_primitive(&new_key, time.clone(), supported_type);       
            }
        }
    }

    pub fn get_latest_primitive<T: TopicKeyProvider>(&self, topic: &T) -> Option<Primitives> {
        let topic = topic.handle();
        self.buckets
            .get(&topic)
            .and_then(|b| b.read().unwrap().get_latest_value().cloned())
    }

    pub fn get_latest_primitives<T: TopicKeyProvider>(
        &self,
        topics: BTreeSet<T>,
    ) -> BTreeMap<TopicKey, Primitives> {
        topics
            .iter()
            .filter_map(|t| self.get_latest_primitive(t).map(|p| (t.key().clone(), p)))
            .collect()
    }

    pub fn get_datapoints<T: TopicKeyProvider>(
        &self,
        topic: &T,
    ) -> Result<Vec<Datapoint>, DatastoreError> {
        let topic = topic.handle();
        let bucket = self.get_bucket(&topic)?;
        let bucket = bucket.read().unwrap();
        Ok(bucket.get_datapoints())
    }

    pub fn get_updated_keys<T: TopicKeyProvider>(
        &self,
        topic: &T,
        time: &Timepoint,
    ) -> Result<Vec<TopicKeyHandle>, DatastoreError> {
        let topic = topic.handle();
        let bucket = self.get_bucket(&topic)?;
        let bucket = bucket.read().unwrap();
        let new_values = bucket.get_data_points_after(&time);
        Ok(new_values.iter().map(|v| v.topic.handle()).collect())
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    use crate::database::*;

    #[test]
    pub fn test_datastore_creation() {
        let datastore = Datastore::new();
        assert_eq!(datastore.buckets.len(), 0);
    }

    #[test]
    pub fn test_datastore_create_bucket() {
        let mut datastore = Datastore::new();
        let topic = TopicKey::from_str("test/topic");
        datastore.create_bucket(&topic);
        assert_eq!(datastore.buckets.len(), 1);
        assert!(datastore.buckets.contains_key(&topic.handle()));
    }

    #[test]
    pub fn test_datastore_get_bucket() {
        let mut datastore = Datastore::new();
        let topic = TopicKey::from_str("test/topic");

        let bucket_failed = datastore.get_bucket(&topic);
        assert_eq!(bucket_failed.is_err(), true);

        datastore.create_bucket(&topic);

        let bucket = datastore.get_bucket(&topic);
        assert_eq!(bucket.is_ok(), true);
        assert_eq!(bucket.unwrap().read().unwrap().topic, topic.handle());
    }

    #[test]
    pub fn test_datastore_add_primitive() {
        let mut datastore = Datastore::new();
        let topic: TopicKey = "test/topic".into();
        let time = Timepoint::now();
        datastore.add_primitive(&topic, time.clone(), 42.into());

        let bucket = datastore.get_bucket(&topic).unwrap();
        let bucket = bucket.read().unwrap();
        let datapoints = bucket.get_datapoints();
        assert_eq!(datapoints.len(), 1);
        assert_eq!(datapoints[0].time, time.clone().into());
        assert_eq!(datapoints[0].value, 42.into());
    }
    #[derive(Serialize, Deserialize)]
    struct TestInnerStruct{
        a: i32,
        b: f64,
        c: String,
        d: bool,
    }
    #[derive(Serialize, Deserialize)]
    struct TestStruct{
        a: i32,
        b: f64,
        c: String,
        d: bool,
        e: TestInnerStruct,
        list: Vec<i32>,
        list_obj: Vec<TestInnerStruct>,
    }

    impl Default for TestStruct{
        fn default() -> Self{
            TestStruct{
                a: 42,
                b: 42.0,
                c: "42".to_string(),
                d: true,
                e: TestInnerStruct{
                    a: 42,
                    b: 42.0,
                    c: "42".to_string(),
                    d: true,
                },
                list: vec![42, 42, 42],
                list_obj: vec![TestInnerStruct{a: 42, b: 42.0, c: "42".to_string(), d: true}],
            }
        }
    }

    #[test]
    pub fn test_datastore_add_json() {
        let mut datastore = Datastore::new();

        let data = TestStruct::default();

        let topic: TopicKey = "test/topic".into();
        let time = Timepoint::now();
        datastore.add_json(&topic, time.clone(), json!(data));

    
        println!("{:?}", datastore.buckets.keys());
        assert!(false)
    }
}
