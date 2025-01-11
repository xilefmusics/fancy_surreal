use crate::{Databasable, Error, Record};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

pub enum Operator {
    Set,
    Add,
    Remove,
}

impl Operator {
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Set => "=",
            Self::Add => "+=",
            Self::Remove => "-=",
        }
    }
}

pub struct Change<'a> {
    client: Surreal<Client>,
    table: &'a str,
    condition: String,
    update: String,
}

impl<'a> Change<'a> {
    pub fn new(client: Surreal<Client>, table: &'a str, owners: Vec<String>) -> Self {
        let change = Self {
            client,
            table,
            condition: String::new(),
            update: String::new(),
        };

        if owners.contains(&"admin".to_string()) {
            change
        } else if owners.len() > 1 {
            change.condition(&format!(
                "owner in [{}]",
                owners
                    .iter()
                    .map(|owner| format!("\"{}\"", owner))
                    .collect::<Vec<String>>()
                    .join(",")
            ))
        } else if owners.len() > 0 {
            change.condition(&format!("owner == \"{}\"", owners[0]))
        } else {
            change
        }
    }

    pub fn condition(mut self, condition: &str) -> Self {
        if self.condition.len() == 0 {
            self.condition = condition.into();
        } else {
            self.condition = format!("{} AND {}", self.condition, condition);
        }
        self
    }

    pub fn update<T: Serialize>(
        mut self,
        key: &str,
        operator: &Operator,
        value: &T,
    ) -> Result<Self, serde_json::Error> {
        let separator = if self.update.len() > 0 { ", " } else { "" };
        self.update = format!(
            "{}{}{} {} {}",
            self.update,
            separator,
            key,
            operator.to_str(),
            serde_json::to_string(value)?
        );
        Ok(self)
    }

    pub fn id(self, id: &str) -> Self {
        let condition = format!("id = {}:{}", self.table, id);
        self.condition(&condition)
    }

    pub fn query_str(&self) -> String {
        let mut query = format!("UPDATE {} SET {}", self.table, self.update);

        if self.condition.len() > 0 {
            query = format!("{} WHERE {}", query, self.condition);
        }

        query + ";"
    }

    pub async fn query<T: Serialize + DeserializeOwned + Databasable>(
        &self,
    ) -> Result<Vec<T>, Error> {
        Ok(self
            .client
            .query(self.query_str())
            .await?
            .take::<Vec<Record<T>>>(0)?
            .into_iter()
            .map(|record: Record<T>| record.content())
            .collect())
    }
}
