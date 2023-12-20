use crate::Databasable;
use crate::Error;
use crate::Record;
use crate::Select;

use futures::future::join_all;
use serde::de::DeserializeOwned;
use serde::Serialize;
use surrealdb::engine::remote::ws;
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

#[derive(Debug, Clone)]
pub struct Client {
    client: Surreal<ws::Client>,
    table: Option<String>,
    owner: Option<String>,
}

impl Client {
    pub async fn new(
        host: &str,
        port: u16,
        username: &str,
        password: &str,
        namespace: &str,
        database: &str,
    ) -> Result<Self, Error> {
        let client = Surreal::new::<ws::Ws>(format!("{}:{}", host, port)).await?;
        client.signin(Root { username, password }).await.unwrap();
        client.use_ns(namespace).use_db(database).await.unwrap();
        Ok(Self {
            client,
            table: None,
            owner: None,
        })
    }

    pub fn table(&self, table: &str) -> Self {
        Self {
            client: self.client.clone(),
            table: Some(table.to_string()),
            owner: self.owner.clone(),
        }
    }

    pub fn owner(&self, owner: &str) -> Self {
        Self {
            client: self.client.clone(),
            table: self.table.clone(),
            owner: Some(owner.to_string()),
        }
    }

    pub fn select(self) -> Result<Select, Error> {
        Ok(Select::new(
            self.client.clone(),
            self.get_table()?,
            self.owner,
        ))
    }

    fn get_table(&self) -> Result<String, Error> {
        self.table.clone().ok_or(Error::new("table is none"))
    }

    async fn authorized(&self, id: &str) -> Result<(), Error> {
        if let Some(owner) = self.owner.clone() {
            let table = self.get_table()?;
            let mut response = self
                .client
                .query(format!(
                    "count(SELECT id FROM {} WHERE id == {}:{}) == 1;",
                    &table, &table, id,
                ))
                .query(format!(
                    "count(SELECT id FROM {} WHERE id =={}:{} AND owner == \"{}\") == 1;",
                    &table, &table, id, owner,
                ))
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
        } else {
            Ok(())
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

    pub async fn create_one<T: Databasable + Serialize + DeserializeOwned>(
        &self,
        content: T,
    ) -> Result<Vec<T>, Error> {
        let table = self.table.clone().ok_or(Error::new("no table given"))?;
        if let Some(id) = content.get_id() {
            self.client
                .create((&table, id))
                .content(Record::new(content, table.clone(), self.owner.clone()))
                .await?
                .map(|record: Record<T>| vec![record.content()])
                .ok_or(Error::new("record is none"))
        } else {
            Ok(self
                .client
                .create(&table)
                .content(Record::new(content, table, self.owner.clone()))
                .await?
                .into_iter()
                .map(|record: Record<T>| record.content())
                .collect())
        }
    }

    pub async fn create<T: Databasable + Serialize + DeserializeOwned>(
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

    pub async fn update_one<T: Databasable + Serialize + DeserializeOwned>(
        &self,
        content: T,
    ) -> Result<Vec<T>, Error> {
        let table = self.table.clone().ok_or(Error::new("no table given"))?;
        let id = content.get_id().ok_or(Error::new("no id given"))?;
        self.authorized(&id).await?;
        self.client
            .update((&table, id))
            .content(Record::new(content, table, self.owner.clone()))
            .await?
            .map(|record: Record<T>| vec![record.content()])
            .ok_or(Error::new("record is none"))
    }

    pub async fn update<T: Databasable + Serialize + DeserializeOwned>(
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

    pub async fn delete_one<T: Databasable + Serialize + DeserializeOwned>(
        &self,
        content: T,
    ) -> Result<Vec<T>, Error> {
        let table = self.table.clone().ok_or(Error::new("no table given"))?;
        let id = content.get_id().ok_or(Error::new("no id given"))?;
        self.authorized(&id).await?;
        self.client
            .delete((&table, id))
            .await?
            .map(|record: Record<T>| vec![record.content()])
            .ok_or(Error::new("record is none"))
    }

    pub async fn delete<T: Databasable + Serialize + DeserializeOwned>(
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
