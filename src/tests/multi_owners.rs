use super::SimpleDatabasable;
use crate::{Client, Error};

#[tokio::test]
async fn multi_owners() -> Result<(), Error> {
    let db = Client::new("localhost", 8000, "root", "root", "test", "test").await?;
    db.drop_table::<SimpleDatabasable>("multi_owners").await?;

    db.table("multi_owners")
        .owner("owner_a")
        .create_one(SimpleDatabasable {
            id: Some("a".into()),
        })
        .await?;

    db.table("multi_owners")
        .owner("owner_b")
        .create_one(SimpleDatabasable {
            id: Some("b".into()),
        })
        .await?;

    db.table("multi_owners")
        .owner("owner_c")
        .create_one(SimpleDatabasable {
            id: Some("c".into()),
        })
        .await?;

    assert_eq!(
        db.table("multi_owners")
            .owners(vec!["owner_a".to_string(), "owner_b".to_string()])
            .select()?
            .query::<SimpleDatabasable>()
            .await?,
        vec![
            SimpleDatabasable {
                id: Some("a".into()),
            },
            SimpleDatabasable {
                id: Some("b".into()),
            },
        ]
    );

    Ok(())
}
