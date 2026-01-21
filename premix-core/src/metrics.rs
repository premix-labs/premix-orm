use metrics_exporter_prometheus::PrometheusBuilder;
pub use metrics_exporter_prometheus::PrometheusHandle;

/// Install the Prometheus recorder and return the handle for scraping.
pub fn install_prometheus_recorder() -> Result<PrometheusHandle, Box<dyn std::error::Error>> {
    let handle = PrometheusBuilder::new().install_recorder()?;
    Ok(handle)
}

/// Record SQLx pool stats as gauges.
pub fn record_pool_stats<DB: sqlx::Database>(pool: &sqlx::Pool<DB>, db: &'static str) {
    metrics::gauge!("premix.pool.size", "db" => db).set(pool.size() as f64);
    metrics::gauge!("premix.pool.idle", "db" => db).set(pool.num_idle() as f64);
    metrics::gauge!("premix.pool.max_size", "db" => db)
        .set(pool.options().get_max_connections() as f64);
}
