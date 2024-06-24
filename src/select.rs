use crate::{Databasable, Error, Record};

use serde::de::DeserializeOwned;
use serde::Serialize;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

pub struct Select<'a> {
    client: Surreal<Client>,
    table: &'a str,
    fields: String,
    condition: String,
    wrapper: Vec<(String, String)>,
    order_by: String,
}

impl<'a> Select<'a> {
    pub fn new(client: Surreal<Client>, table: &'a str, owners: Vec<&str>) -> Self {
        let select = Self {
            client,
            table,
            fields: String::new(),
            condition: String::new(),
            wrapper: Vec::new(),
            order_by: String::new(),
        };

        if owners.len() > 1 {
            select.condition(&format!(
                "owner in [{}]",
                owners
                    .iter()
                    .map(|owner| format!("\"{}\"", owner))
                    .collect::<Vec<String>>()
                    .join(",")
            ))
        } else if owners.len() > 0 {
            select.condition(&format!("owner == \"{}\"", owners[0]))
        } else {
            select
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

    pub fn id(self, id: &str) -> Self {
        let condition = format!("id = {}:{}", self.table, id);
        self.condition(&condition)
    }

    pub fn field(mut self, field: &str) -> Self {
        if self.fields.len() == 0 {
            self.fields = field.into();
        } else {
            self.fields = format!("{}, {}", self.fields, field);
        }
        self
    }

    pub fn wrapper(mut self, wrapper: (&str, &str)) -> Self {
        self.wrapper.push((wrapper.0.into(), wrapper.1.into()));
        self
    }

    pub fn wrapper_fn(self, function: &str) -> Self {
        self.wrapper((&(function.to_string() + "("), ")"))
    }

    pub fn wrapper_js(self, function: &str) -> Self {
        self.wrapper(("function(", &format!("){{{}}}", function)))
    }

    pub fn wrapper_js_map(self, function: &str) -> Self {
        self.wrapper_js(&format!("return arguments[0].map(element => {})", function))
    }

    pub fn order_by(mut self, field: &str) -> Self {
        self.order_by = field.into();
        self
    }

    pub fn query_str(&self) -> String {
        let fields = if self.fields.len() > 0 {
            &self.fields
        } else {
            "*"
        };

        let mut query = format!("SELECT {} FROM {}", fields, self.table);

        if self.condition.len() > 0 {
            query = format!("{} WHERE {}", query, self.condition);
        }

        if self.order_by.len() > 0 {
            query = format!("{} ORDER BY {}", query, self.order_by);
        }

        for wrapper in &self.wrapper {
            query = format!("{}{}{}", wrapper.0, query, wrapper.1);
        }

        println!("{}", query);
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

    pub async fn query_one<T: Serialize + DeserializeOwned + Databasable>(
        &self,
    ) -> Result<T, Error> {
        Ok(self.query().await?.remove(0))
    }

    pub async fn query_direct<T: Serialize + DeserializeOwned>(&self) -> Result<Vec<T>, Error> {
        Ok(self
            .client
            .query(self.query_str())
            .await?
            .take::<Vec<T>>(0)?)
    }

    pub async fn query_direct_one<T: Serialize + DeserializeOwned>(&self) -> Result<T, Error> {
        Ok(self.query_direct().await?.remove(0))
    }
}
