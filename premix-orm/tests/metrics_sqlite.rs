#![cfg(feature = "metrics")]

use premix_orm::metrics::install_prometheus_recorder;
use premix_orm::prelude::*;
use sqlx::Sqlite;

#[derive(Model, Debug, Clone)]
struct MetricUser {
    id: i32,
    name: String,
}

#[tokio::test]
async fn metrics_are_recorded_for_queries() {
    let handle = install_prometheus_recorder().expect("recorder");
    let pool = Premix::smart_sqlite_pool("sqlite::memory:")
        .await
        .expect("pool");
    Premix::sync::<Sqlite, MetricUser>(&pool)
        .await
        .expect("sync");

    let mut user = MetricUser {
        id: 0,
        name: "Metric".to_string(),
    };
    user.save(&pool).await.expect("save");

    let _ = MetricUser::find_in_pool(&pool)
        .filter_eq("name", "Metric")
        .all()
        .await
        .expect("all");

    metrics::counter!("premix.test.counter").increment(1);
    metrics::gauge!("premix.test.gauge").set(1.0);

    let rendered = handle.render();
    assert!(rendered.contains("premix_test_counter"));
    assert!(rendered.contains("premix_test_gauge"));
}
