use super::*;
use crate::{ChangeOperator, Client};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Inner {
    a: String,
    b: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Outher {
    id: Option<String>,
    data: Vec<Inner>,
}

impl Databasable for Outher {
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

    db.drop_table::<Outher>("change_test").await.unwrap();

    db.table("change_test")
        .owner("test")
        .create(vec![
            Outher {
                id: Some("firstId".to_string()),
                data: vec![],
            },
            Outher {
                id: Some("secondId".to_string()),
                data: vec![],
            },
        ])
        .await
        .unwrap();

    assert_eq!(
        db.table("change_test")
            .owner("test")
            .change()
            .unwrap()
            .id("firstId")
            .update(
                "content.data",
                &ChangeOperator::Add,
                &Inner {
                    a: "a".to_string(),
                    b: "b".to_string(),
                }
            )
            .unwrap()
            .query::<Outher>()
            .await
            .unwrap()
            .len(),
        1
    );

    assert_eq!(
        db.table("change_test")
            .owner("test")
            .select()
            .unwrap()
            .query::<Outher>()
            .await
            .unwrap(),
        vec![
            Outher {
                id: Some("firstId".to_string()),
                data: vec![Inner {
                    a: "a".to_string(),
                    b: "b".to_string(),
                }],
            },
            Outher {
                id: Some("secondId".to_string()),
                data: vec![],
            },
        ]
    );
}
