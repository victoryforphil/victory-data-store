use std::{
    hash::{BuildHasher, DefaultHasher, Hash, Hasher},
    sync::Arc,
};

use serde::{Deserialize, Serialize};
use tracing::instrument;

pub type TopicIDType = u64;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopicKeySection {
    pub id: TopicIDType,
    pub display_name: String,
}
pub type TopicKeySectionHandle = Arc<TopicKeySection>;
impl Hash for TopicKeySection {
    #[instrument(skip_all)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl TopicKeySection {
    #[instrument(skip_all)]
    pub fn new_existing(id: TopicIDType, display_name: String) -> TopicKeySection {
        TopicKeySection { id, display_name }
    }

    pub fn handle(&self) -> TopicKeySectionHandle {
        Arc::new(self.clone())
    }

    pub fn into_handle(self) -> TopicKeySectionHandle {
        Arc::new(self)
    }
    #[instrument(skip_all)]
    pub fn new_generate(display_name: &str) -> TopicKeySection {
        let mut hasher = DefaultHasher::new();
        display_name.hash(&mut hasher);
        let id = hasher.finish() as TopicIDType;
        TopicKeySection {
            id,
            display_name: display_name.to_string(),
        }
    }
}
#[derive(Clone, Eq, Serialize, Deserialize)]
pub struct TopicKey {
    pub sections: Vec<TopicKeySectionHandle>,
}

// Implement string formatting / printing (dispaly name)
impl std::fmt::Display for TopicKey {
    #[instrument(skip_all, name = "TopicKey::fmt")]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl std::fmt::Debug for TopicKey {
    #[instrument(skip_all, name = "TopicKey::fmt")]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl Hash for TopicKey {
    #[instrument(skip_all)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        for section in &self.sections {
            section.id.hash(state);
        }
    }
}

impl Ord for TopicKey {
    #[instrument(skip_all)]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.display_name().cmp(&other.display_name())
    }
}

impl PartialOrd for TopicKey {
    #[instrument(skip_all)]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for TopicKey {
    #[instrument(skip_all, name = "TopicKey::eq")]
    fn eq(&self, other: &Self) -> bool {
        self.sections.len() == other.sections.len()
            && self
                .sections
                .iter()
                .zip(other.sections.iter())
                .all(|(a, b)| a.id == b.id)
    }
}
pub type TopicKeyHandle = Arc<TopicKey>;

pub trait TopicKeyProvider {
    fn key(&self) -> &TopicKey;
    #[instrument(skip_all)]
    fn handle(&self) -> TopicKeyHandle {
        Arc::new(self.key().clone())
    }
}

impl TopicKeyProvider for TopicKeyHandle {
    fn key(&self) -> &TopicKey {
        self.as_ref()
    }
}

impl TopicKeyProvider for TopicKey {
    fn key(&self) -> &TopicKey {
        self
    }
}

impl TopicKey {
    #[instrument(skip_all)]
    pub fn from_str(display_name: &str) -> TopicKey {
        let sections: Vec<TopicKeySectionHandle> = display_name
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|s| TopicKeySection::new_generate(s).into_handle())
            .collect();
        TopicKey { sections }
    }

    #[instrument(skip_all)]
    pub fn from_existing(sections: Vec<TopicKeySectionHandle>) -> TopicKey {
        TopicKey { sections }
    }

    #[instrument(skip_all)]
    pub fn empty() -> TopicKey {
        TopicKey { sections: vec![] }
    }
    #[instrument(skip_all)]
    pub fn display_name(&self) -> String {
        self.sections
            .iter()
            .map(|s| s.display_name.to_string())
            .collect::<Vec<String>>()
            .join("/")
    }
    #[instrument(skip_all, name = "TopicKey::to_string")]
    pub fn to_string(&self) -> String {
        self.sections
            .iter()
            .map(|s| s.id.to_string())
            .collect::<Vec<String>>()
            .join("/")
    }
    #[instrument(skip_all)]
    pub fn add_prefix(&self, prefix: TopicKey) -> TopicKey {
        TopicKey {
            sections: prefix
                .sections
                .into_iter()
                .chain(self.sections.clone())
                .collect(),
        }
    }

    #[instrument(skip_all)]
    pub fn add_prefix_mut(&mut self, prefix: TopicKey) {
        for section in prefix.sections.into_iter().rev() {
            self.sections.insert(0, section);
        }
    }
    #[instrument(skip_all)]
    pub fn remove_prefix(&self, prefix: TopicKey) -> Option<TopicKey> {
        if !self.is_child_of(&prefix) {
            return None;
        }

        let sections = self.sections[prefix.sections.len()..].to_vec();
        Some(TopicKey::from_existing(sections))
    }

    pub fn add_suffix_owned(mut self, suffix: TopicKey) -> TopicKey {
        self.add_suffix_mut(&suffix);
        self
    }

    #[instrument(skip_all)]
    pub fn add_suffix(&self, suffix: &TopicKey) -> TopicKey {
        let mut new_key = self.clone();
        new_key.add_suffix_mut(&suffix);
        new_key
    }

    #[instrument(skip_all)]
    pub fn add_suffix_mut(&mut self, suffix: &TopicKey) {
        self.sections.extend(suffix.sections.clone());
    }

    #[instrument(skip_all)]
    pub fn remove_suffix(&self, suffix: &TopicKey) -> Option<TopicKey> {
        let sections = self.sections[..self.sections.len() - suffix.sections.len()].to_vec();
        Some(TopicKey::from_existing(sections))
    }

    #[instrument(skip_all)]
    pub fn is_child_of(&self, parent: &TopicKey) -> bool {
        if self.sections.len() <= parent.sections.len() {
            return false;
        }

        for i in 0..parent.sections.len() {
            if self.sections[i] != parent.sections[i] {
                return false;
            }
        }

        true
    }

    #[instrument(skip_all)]
    pub fn is_parent_of(&self, child: &TopicKey) -> bool {
        child.is_child_of(self)
    }

    #[instrument(skip_all, name = "TopicKey::id")]
    pub fn id(&self) -> TopicIDType {
        let mut hasher = DefaultHasher::new();
        for section in &self.sections {
            section.id.hash(&mut hasher);
        }
        hasher.finish() as TopicIDType
    }
}

impl From<&str> for TopicKey {
    fn from(value: &str) -> Self {
        TopicKey::from_str(value)
    }
}

impl From<String> for TopicKey {
    fn from(value: String) -> Self {
        TopicKey::from_str(&value)
    }
}

impl From<&String> for TopicKey {
    fn from(value: &String) -> Self {
        TopicKey::from_str(value)
    }
}

// Test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topic_key() {
        let key = TopicKey::from_str("test/test");
        assert_eq!(key.sections.len(), 2);
        assert_eq!(key.sections[0].display_name, "test");
        assert_eq!(key.sections[1].display_name, "test");
        assert_eq!(key.sections[0].id, key.sections[1].id);
    }
    #[test]
    fn test_topic_key_display() {
        let key = TopicKey::from_str("test/test");
        assert_eq!(key.display_name(), "test/test");
    }
    #[test]
    fn test_topic_prefix() {
        let key = TopicKey::from_str("test/test");
        let prefix = TopicKey::from_str("prefix/key");
        let prefixed = key.add_prefix(prefix);
        assert_eq!(prefixed.display_name(), "prefix/key/test/test");
    }
    #[test]
    fn test_topic_suffix() {
        let key = TopicKey::from_str("test/test");
        let suffix = TopicKey::from_str("suffix/key");
        let suffixed = key.add_suffix(&suffix);
        assert_eq!(suffixed.display_name(), "test/test/suffix/key");
    }
}
