pub mod memory;

use dnskit::protocol::message::{Question, ResourceRecord};

pub trait DnsCache {
    fn get(&mut self, question: &Question) -> Option<Vec<ResourceRecord>>;
    fn replace(&mut self, question: &Question, records: Vec<ResourceRecord>);
}
