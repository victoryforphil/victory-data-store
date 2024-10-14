use blob::VicBlob;
use victory_time_rs::{Timepoint, Timespan};

use crate::topics::TopicIDType;

pub mod blob;
pub mod integer;
pub mod string;
pub mod json;

#[derive(Debug, Clone, PartialEq)]
pub enum Primitives {
    Instant(Timepoint),
    Duration(Timespan),
    Integer(i64),
    Float(f64),
    Text(String),
    Blob(VicBlob),
    Boolean(bool),
    List(Vec<Primitives>),
    Reference(TopicIDType),
}


