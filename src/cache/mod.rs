use std::collections::{HashSet, VecDeque};

#[derive(Debug)]
pub struct SignatureCache {
    max_entries: usize,
    entries: HashSet<String>,
    order: VecDeque<String>,
}

impl SignatureCache {
    pub fn new(max_entries: usize) -> Self {
        Self {
            max_entries,
            entries: HashSet::new(),
            order: VecDeque::new(),
        }
    }

    pub fn is_seen(&self, signature: &str) -> bool {
        self.entries.contains(signature)
    }

    pub fn mark_seen(&mut self, signature: String) {
        if self.entries.contains(&signature) {
            return;
        }

        self.entries.insert(signature.clone());
        self.order.push_back(signature);

        while self.entries.len() > self.max_entries {
            if let Some(oldest) = self.order.pop_front() {
                self.entries.remove(&oldest);
            }
        }
    }
}
