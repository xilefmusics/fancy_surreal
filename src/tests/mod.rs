use crate::*;
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod basic;
#[cfg(test)]
mod multi_owners;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct SimpleDatabasable {
    id: Option<String>,
}

impl Databasable for SimpleDatabasable {
    fn get_id(&self) -> Option<String> {
        self.id.clone()
    }

    fn set_id(&mut self, id: Option<String>) {
        self.id = id;
    }
}
