use crate::{Change, Databasable, Error, Record, Select};

use futures::future::join_all;
use serde::de::DeserializeOwned;
use serde::Serialize;
use surrealdb::engine::remote::ws;
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

#[derive(Debug, Clone)]
pub struct Client<'a> {
    client: Surreal<ws::Client>,
    table: Option<&'a str>,
    owners: Vec<String>,
}

impl<'a> Client<'a> {
    pub async fn new(
        host: &str,
        port: u16,
        username: &str,
        password: &str,
        namespace: &str,
        database: &str,
    ) -> Result<Self, Error> {
        let client = Surreal::new::<ws::Ws>(format!("{}:{}", host, port)).await?;
        client.signin(Root { username, password }).await?;
        client.use_ns(namespace).use_db(database).await?;
        Ok(Self {
            client,
            table: None,
            owners: vec![],
        })
    }

    pub fn table(&self, table: &'a str) -> Self {
        Self {
            client: self.client.clone(),
            table: Some(table),
            owners: self.owners.clone(),
        }
    }

    pub fn owner(&self, owner: &'a str) -> Self {
        Self {
            client: self.client.clone(),
            table: self.table.clone(),
            owners: vec![owner.to_string()],
        }
    }

    pub fn owners(&self, owners: Vec<String>) -> Self {
        Self {
            client: self.client.clone(),
            table: self.table.clone(),
            owners,
        }
    }

    pub fn select(self) -> Result<Select<'a>, Error> {
        Ok(Select::new(
            self.client.clone(),
            self.get_table()?,
            self.owners,
        ))
    }

    pub fn change(self) -> Result<Change<'a>, Error> {
        Ok(Change::new(
            self.client.clone(),
            self.get_table()?,
            self.owners,
        ))
    }

    fn get_table(&self) -> Result<&'a str, Error> {
        self.table.ok_or(Error::new("table is none"))
    }

    fn first_owner(&self) -> Option<String> {
        self.owners.first().map(|owner| owner.to_string())
    }

    async fn authorized(&self, id: &str) -> Result<(), Error> {
        if self.owners.len() == 0 {
            return Ok(());
        }
        if self.owners.contains(&"admin".to_string()) {
            return Ok(());
        }

        let table = self.get_table()?;
        let mut response = self
            .client
            .query(format!(
                "count(SELECT id FROM {} WHERE id == {}:{}) == 1;",
                &table, &table, id,
            ))
            .query(if self.owners.len() > 1 {
                format!(
                    "count(SELECT id FROM {} WHERE id =={}:{} AND owner in [{}]) == 1;",
                    &table,
                    &table,
                    id,
                    self.owners
                        .iter()
                        .map(|owner| format!("\"{}\"", owner))
                        .collect::<Vec<String>>()
                        .join(","),
                )
            } else {
                format!(
                    "count(SELECT id FROM {} WHERE id =={}:{} AND owner == \"{}\") == 1;",
                    &table, &table, id, self.owners[0],
                )
            })
            .await?;
        let exists: Vec<bool> = response.take(0)?;
        if !exists[0] {
            Ok(())
        } else {
            let authorized: Vec<bool> = response.take(1)?;
            if authorized[0] {
                Ok(())
            } else {
                Err(Error::new("not authorized"))
            }
        }
    }

    pub async fn drop_table<T: Databasable + Serialize + DeserializeOwned>(
        &self,
        table: &str,
    ) -> Result<Vec<T>, Error> {
        Ok(self
            .client
            .delete(table)
            .await?
            .into_iter()
            .map(|record: Record<T>| record.content())
            .collect())
    }

    pub async fn create_one<T: Databasable + Serialize + DeserializeOwned + 'static>(
        &self,
        content: T,
    ) -> Result<Vec<T>, Error> {
        let table = self.table.clone().ok_or(Error::new("no table given"))?;
        if let Some(id) = content.get_id() {
            self.client
                .create((table, id))
                .content(Record::new(content, table.to_string(), self.first_owner()))
                .await?
                .map(|record: Record<T>| vec![record.content()])
                .ok_or(Error::new("record is none"))
        } else {
            Ok(self
                .client
                .create(table)
                .content(Record::new(content, table.to_string(), self.first_owner()))
                .await?
                .into_iter()
                .map(|record: Record<T>| record.content())
                .collect())
        }
    }

    pub async fn create<T: Databasable + Serialize + DeserializeOwned + 'static>(
        &self,
        content: Vec<T>,
    ) -> Result<Vec<T>, Error> {
        join_all(content.into_iter().map(|content| self.create_one(content)))
            .await
            .into_iter()
            .try_fold(Vec::new(), |acc, result| {
                result.and_then(|inner_vec| {
                    let mut acc = acc;
                    acc.extend(inner_vec);
                    Ok(acc)
                })
            })
    }

    pub async fn update_one<T: Databasable + Serialize + DeserializeOwned + Clone + 'static>(
        &self,
        content: T,
    ) -> Result<Vec<T>, Error> {
        let table = self.table.clone().ok_or(Error::new("no table given"))?;
        let id = content.get_id().ok_or(Error::new("no id given"))?;
        self.authorized(&id).await?;
        Ok(self
            .client
            .update((table, id))
            .content(Record::new(
                content.clone(),
                table.to_string(),
                self.first_owner(),
            ))
            .await?
            .map(|record: Record<T>| vec![record.content()])
            .unwrap_or(self.create_one(content).await?))
    }

    pub async fn update<T: Databasable + Serialize + DeserializeOwned + Clone + 'static>(
        &self,
        content: Vec<T>,
    ) -> Result<Vec<T>, Error> {
        join_all(content.into_iter().map(|content| self.update_one(content)))
            .await
            .into_iter()
            .try_fold(Vec::new(), |acc, result| {
                result.and_then(|inner_vec| {
                    let mut acc = acc;
                    acc.extend(inner_vec);
                    Ok(acc)
                })
            })
    }

    pub async fn delete_one<T: Databasable + Serialize + DeserializeOwned + 'static>(
        &self,
        content: T,
    ) -> Result<Vec<T>, Error> {
        let table = self.table.clone().ok_or(Error::new("no table given"))?;
        let id = content.get_id().ok_or(Error::new("no id given"))?;
        self.authorized(&id).await?;
        self.client
            .delete((table, id))
            .await?
            .map(|record: Record<T>| vec![record.content()])
            .ok_or(Error::new("record is none"))
    }

    pub async fn delete<T: Databasable + Serialize + DeserializeOwned + 'static>(
        &self,
        content: Vec<T>,
    ) -> Result<Vec<T>, Error> {
        join_all(content.into_iter().map(|content| self.delete_one(content)))
            .await
            .into_iter()
            .try_fold(Vec::new(), |acc, result| {
                result.and_then(|inner_vec| {
                    let mut acc = acc;
                    acc.extend(inner_vec);
                    Ok(acc)
                })
            })
    }
}
