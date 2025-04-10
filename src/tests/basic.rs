use super::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MyData {
    id: Option<String>,
    data: String,
}

impl Databasable for MyData {
    fn get_id(&self) -> Option<String> {
        self.id.clone()
    }

    fn set_id(&mut self, id: Option<String>) {
        self.id = id;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MyNewData {
    id: Option<String>,
    inner: HashMap<String, usize>,
}

impl Databasable for MyNewData {
    fn get_id(&self) -> Option<String> {
        self.id.clone()
    }

    fn set_id(&mut self, id: Option<String>) {
        self.id = id;
    }
}

#[tokio::test]
async fn basic() {
    let db = Client::new("localhost", 8000, "root", "root", "test", "test")
        .await
        .unwrap();

    db.drop_table::<MyData>("test_table").await.unwrap();

    // create
    db.table("test_table")
        .owner("test_user")
        .create(vec![
            MyData {
                id: Some("firstId".to_string()),
                data: "firstData".to_string(),
            },
            MyData {
                id: None,
                data: "secondData".to_string(),
            },
        ])
        .await
        .unwrap();

    // update
    db.table("test_table")
        .owner("test_user")
        .update(vec![MyData {
            id: Some("firstId".to_string()),
            data: "firstDataUpdated".to_string(),
        }])
        .await
        .unwrap();

    // select
    db.table("test_table")
        .owner("test_user")
        .select()
        .unwrap()
        .query::<MyData>()
        .await
        .unwrap();

    db.table("test_table")
        .owner("test_user")
        .select()
        .unwrap()
        .id("firstId")
        .query_one::<MyData>()
        .await
        .unwrap();

    // delete
    db.table("test_table")
        .owner("test_user")
        .delete(vec![MyData {
            id: Some("firstId".to_string()),
            data: "firstDataUpdated".to_string(),
        }])
        .await
        .unwrap();

    // test wrapper functions
    db.drop_table::<MyNewData>("new_table").await.unwrap();
    db.table("new_table")
        .owner("new_user")
        .create(vec![
            MyNewData {
                id: None,
                inner: HashMap::from([("A".into(), 1), ("B".into(), 2), ("C".into(), 3)]),
            },
            MyNewData {
                id: None,
                inner: HashMap::from([("B".into(), 4), ("C".into(), 5), ("D".into(), 6)]),
            },
        ])
        .await
        .unwrap();
    db.table("new_table")
        .owner("new_user2")
        .create(vec![MyNewData {
            id: None,
            inner: HashMap::from([("E".into(), 7), ("F".into(), 8), ("G".into(), 9)]),
        }])
        .await
        .unwrap();

    assert_eq!(
        db.table("new_table")
            .owner("new_user")
            .select()
            .unwrap()
            .field("content.inner as item")
            .wrapper_js_map("Object.keys(element.item)")
            .wrapper_fn("array::group")
            .wrapper_fn("array::sort")
            .query_direct::<String>()
            .await
            .unwrap(),
        vec!["A", "B", "C", "D"]
    );
}
