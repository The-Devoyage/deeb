use criterion::{criterion_group, criterion_main, Criterion};
use deeb::Query;
use serde_json::json;

async fn setup() -> deeb::Deeb {
    let user = deeb::Entity::from("user");
    let comment = deeb::Entity::from("comment");

    let db = deeb::Deeb::new();
    db.add_instance("test", "./user.json", vec![user.clone()])
        .await
        .unwrap();
    db.add_instance("test2", "./comment.json", vec![comment.clone()])
        .await
        .unwrap();

    db
}

fn insert_benchmark(c: &mut Criterion) {
    let user = deeb::Entity::from("user");
    let rt = tokio::runtime::Runtime::new().unwrap();
    let db = rt.block_on(setup());

    let json = json!({
        "name": "John Doe",
        "age": 30,
    });

    c.bench_function("insert", |b| {
        b.iter(|| {
            rt.block_on(async {
                db.insert(&user, json.clone(), None).await.unwrap();
            });
        });
    });
}

// fn insert_1000_benchmark(c: &mut Criterion) {
//     let user = deeb::Entity::from("user");
//     let rt = tokio::runtime::Runtime::new().unwrap();
//     let db = rt.block_on(setup());

//     let json = json!({
//         "name": "John Doe",
//         "age": 30,
//     });

//     c.bench_function("insert 1000", |b| {
//         b.iter(|| {
//             rt.block_on(async {
//                 for _ in 0..1000 {
//                     db.insert(&user, json.clone(), None).await.unwrap();
//                 }
//             });
//         });
//     });
// }

fn insert_1000_transaction_benchmark(c: &mut Criterion) {
    let user = deeb::Entity::from("user");
    let rt = tokio::runtime::Runtime::new().unwrap();
    let db = rt.block_on(setup());

    c.bench_function("insert 1000 transaction", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut transaction = db.begin_transaction().await;
                for i in 0..1000 {
                    let json = json!({
                        "id": i,
                        "name": format!("John Doe {}", i),
                        "age": 30,
                    });
                    db.insert(&user, json.clone(), Some(&mut transaction))
                        .await
                        .unwrap();
                }
            });
        });
    });
}

fn find_one_benchmark(c: &mut Criterion) {
    let user = deeb::Entity::from("user");
    let rt = tokio::runtime::Runtime::new().unwrap();
    let db = rt.block_on(setup());
    // let query = Query::eq("name", json!("John Doe"));
    let query = Query::eq("id", json!(12456));

    c.bench_function("find one", |b| {
        b.iter(|| {
            rt.block_on(async {
                db.find_one(&user, query.clone(), None).await.unwrap();
            });
        });
    });
}

fn find_many_benchmark(c: &mut Criterion) {
    let user = deeb::Entity::from("user");
    let rt = tokio::runtime::Runtime::new().unwrap();
    let db = rt.block_on(setup());
    let query = Query::eq("name", json!("John Doe"));

    c.bench_function("find many", |b| {
        b.iter(|| {
            rt.block_on(async {
                db.find_many(&user, query.clone(), None).await.unwrap();
            });
        });
    });
}

criterion_group!(
    benches,
    insert_benchmark,
    find_one_benchmark,
    find_many_benchmark,
    // insert_1000_benchmark
    insert_1000_transaction_benchmark
);
criterion_main!(benches);
