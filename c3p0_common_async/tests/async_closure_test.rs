// #![feature(async_closure)]

// Use
// cargo +nightly test

use futures::Future;

#[tokio::test]
async fn someday_it_will_work() {
/*
    let async_c = async move |message: String| {
        println!("{}", message);
    };

    foo("hello".to_owned(), async_c).await;
*/
    let a_var = "an_external_ref".to_owned();
    let an_external_ref = &a_var;

    let result = call_async::<_, Box<dyn std::error::Error>,_,_>(|value| async move  {
        let value_ref = value;
        println!("{}", an_external_ref);
        Ok("ok")
    }).await;

    assert_eq!("ok", result.unwrap());
}

async fn foo<F, Fut>(message: String, cb: F)
    where
        F: Fn(String) -> Fut,
        Fut: Future<Output = ()>,
{
    cb(message).await;
}

async fn call_async<'a, T, E, F, Fut>(callback: F) -> Result<T, E>
    where
        T: Send + Sync,
        F: Fn(String) -> Fut,
        Fut: 'a + Future<Output = Result<T, E>>,
{
    let s = "a value".to_owned();
    callback(s).await
}