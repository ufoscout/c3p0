use c3p0_common::*;
use maybe_single::tokio::{Data, MaybeSingleAsync};
use mongodb::{
    bson::{doc, Document},
    Client,
};
use once_cell::sync::OnceCell;
use testcontainers::{
    mongo::Mongo,
    testcontainers::{clients::Cli, Container},
};

pub type MaybeType = (Client, Container<'static, Mongo>);

async fn init() -> MaybeType {
    static DOCKER: OnceCell<Cli> = OnceCell::new();
    let node = DOCKER.get_or_init(Cli::default).run(Mongo);

    let host_port = node.get_host_port_ipv4(27017);
    let url = format!("mongodb://127.0.0.1:{host_port}/");

    let client = Client::with_uri_str(&url).await.unwrap();

    (client, node)
}

pub async fn data(serial: bool) -> Data<'static, MaybeType> {
    static DATA: OnceCell<MaybeSingleAsync<MaybeType>> = OnceCell::new();
    DATA.get_or_init(|| MaybeSingleAsync::new(|| Box::pin(init())))
        .data(serial)
        .await
}

#[tokio::test]
async fn mongo_fetch_document() {
    let data = data(false).await;
    let client = &data.0;
    // let pool = MongodbC3p0Pool::new(client.clone(), "TEST_DB".to_owned());

    let my_obj = Model::<i64, String> {
        id: 100,
        version: 0,
        create_epoch_millis: 0,
        update_epoch_millis: 0,
        data: "hello world!".to_owned(),
    };

    let coll = client
        .database("some_db")
        .collection::<Model<i64, String>>("some-coll");

    let insert_one_result = coll.insert_one(&my_obj, None).await.unwrap();
    println!("inserted_id: {:?}", insert_one_result.inserted_id.as_i64());
    // assert!(!insert_one_result
    //     .inserted_id
    //     .as_object_id()
    //     .unwrap()
    //     .to_hex()
    //     .is_empty());

    let coll = client.database("some_db").collection("some-coll");

    let find_one_result: Document = coll
        .find_one(doc! { "_id": 100 }, None)
        .await
        .unwrap()
        .unwrap();
    println!("find_one_result: {:?}", find_one_result);
    // assert_eq!(42, find_one_result.get_i32("x").unwrap())

    assert!(client
        .database("some_db")
        .collection::<Model<i64, String>>("some-coll")
        .insert_one(&my_obj, None)
        .await
        .is_err());
}

// #[test]
// fn should_be_bson() {
//     let x = 42_i64;
//     doc! { "x": x };
//     test_bson(x);

//     let x = ObjectId::new();
//     doc! { "x": x };
//     test_bson(x);

//     let x = "hello world!";
//     doc! { "x": x };
//     test_bson(x);
// }

// // fn test_bson<T: serde::Serialize>(val: T) {
// //     doc! { "x": val };
// // }

// fn test_bson<T>(val: T)
// where T: Into<Bson> + TryFrom<Bson>
// {
//     doc! { "x": val.into() };
// }
