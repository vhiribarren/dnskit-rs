use std::{
    collections::HashMap,
    time::Instant,
};

use dnskit::protocol::message::{Question, ResourceRecord};

use crate::cache::DnsCache;

pub struct DnsCacheMemory {
    cache: HashMap<Question, Vec<ResourceRecord>>,
}

impl DnsCacheMemory {
    pub fn new() -> Self {
        DnsCacheMemory {
            cache: HashMap::new(),
        }
    }
}

impl DnsCache for DnsCacheMemory {
    fn get(&mut self, question: &Question) -> Option<Vec<ResourceRecord>> {
        let now = Instant::now();
        self.cache.get_mut(question).map(|entries| {
            entries.retain(|e| e.ttl > now);
            entries.clone()
        })
    }

    fn replace(&mut self, question: &Question, records: Vec<ResourceRecord>) {
        self.cache.insert((*question).clone(), records);
    }
}

#[cfg(test)]
mod tests {

    use std::error::Error;

    use dnskit::protocol::message::{Class, QClass, QType, Type};

    use super::*;

    #[test]
    fn test() -> Result<(), Box<dyn Error>> {
        let cache = &mut DnsCacheMemory::new();
        let question = &Question::new(
            "www.alea.net",
            QClass::Class(Class::Internet),
            QType::Type(Type::A),
        )?;

        assert!(matches!(cache.get(question), None));

        Ok(())
    }
}
