use crate::Databasable;

use serde::{Deserialize, Serialize};
use surrealdb::opt::RecordId;
use surrealdb::sql::Id;

#[derive(Debug, Serialize, Deserialize)]
pub struct Record<T: Databasable + Serialize> {
    id: Option<RecordId>,
    owner: Option<String>,
    content: T,
}

impl<'de, T: Databasable + Serialize + Deserialize<'de>> Record<T> {
    pub fn new(mut content: T, table: String, owner: Option<String>) -> Self {
        Record {
            id: content.get_id().map(|id| {
                content.set_id(None);
                RecordId {
                    tb: table,
                    id: Id::String(id.to_string()),
                }
            }),
            owner,
            content,
        }
    }

    pub fn content(self) -> T {
        let mut content = self.content;
        content.set_id(self.id.map(|id| id.id.to_string()));
        content
    }
}
