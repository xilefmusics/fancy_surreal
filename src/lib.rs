mod client;
mod error;
mod record;
mod select;
#[cfg(test)]
mod tests;
mod traits;

pub use client::Client;
pub use error::Error;
pub use record::Record;
pub use select::Select;
pub use surrealdb::opt::RecordId;
pub use surrealdb::sql::Id;
pub use traits::Databasable;
