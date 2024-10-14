use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::Arc,
};

use log::debug;

pub type TopicIDType = String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopicKeySection {
    pub id: TopicIDType,
    pub display_name: String,
}

impl Hash for TopicKeySection {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl TopicKeySection {
    pub fn new_existing(id: TopicIDType, display_name: String) -> TopicKeySection {
        TopicKeySection { id, display_name }
    }

    pub fn new_generate(display_name: String) -> TopicKeySection {
        let id = display_name.to_ascii_lowercase();
        debug!("Generated ID {} for display name {}", id, display_name);
        TopicKeySection { id, display_name }
    }
}
#[derive(Clone, Eq)]
pub struct TopicKey {
    sections: Vec<TopicKeySection>,
}

// Implement string formatting / printing (dispaly name)
impl std::fmt::Display for TopicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl std::fmt::Debug for TopicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl Hash for TopicKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for section in &self.sections {
            section.id.hash(state);
        }
    }
}

impl Ord for TopicKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id().cmp(&other.id())
    }
}

impl PartialOrd for TopicKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for TopicKey {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}
pub type TopicKeyHandle = Arc<TopicKey>;

pub trait TopicKeyProvider {
    fn key(&self) -> &TopicKey;
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
    pub fn from_str(display_name: &str) -> TopicKey {
        let sections = display_name
            .split("/")
            .map(|s| TopicKeySection::new_generate(s.to_string()))
            .collect();

        TopicKey { sections }
    }

    pub fn from_existing(sections: Vec<TopicKeySection>) -> TopicKey {
        TopicKey { sections }
    }

    pub fn display_name(&self) -> String {
        self.sections
            .iter()
            .map(|s| s.display_name.clone())
            .collect::<Vec<String>>()
            .join("/")
    }

    pub fn add_prefix(&self, prefix: TopicKey) -> TopicKey {
        let mut sections = prefix.sections.clone();
        sections.extend(self.sections.clone());
        TopicKey::from_existing(sections)
        
    }

    pub fn add_suffix(&self, suffix: TopicKey) -> TopicKey {
        let mut sections = self.sections.clone();
        sections.extend(suffix.sections.clone());
        TopicKey::from_existing(sections)
    }

    pub fn to_string(&self) -> String {
        self.sections
            .iter()
            .map(|s| s.id.clone())
            .collect::<Vec<String>>()
            .join("/")
    }

    pub fn id(&self) -> TopicIDType {
        let mut hasher = DefaultHasher::new();
        for section in &self.sections {
            section.id.hash(&mut hasher);
        }
        hasher.finish().to_string()
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
        let suffixed = key.add_suffix(suffix);
        assert_eq!(suffixed.display_name(), "test/test/suffix/key");
    }
}
