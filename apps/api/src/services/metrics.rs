use metrics::{counter, histogram};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use std::sync::OnceLock;

static HANDLE: OnceLock<PrometheusHandle> = OnceLock::new();

pub fn init_metrics() -> PrometheusHandle {
    HANDLE
        .get_or_init(|| {
            PrometheusBuilder::new()
                .install_recorder()
                .expect("failed to install prometheus recorder")
        })
        .clone()
}

pub fn metrics_handle() -> PrometheusHandle {
    init_metrics()
}

pub fn record_http_request(method: &str, path: &str, status: u16, latency_secs: f64) {
    counter!(
        "tempoforge_http_requests_total",
        "method" => method.to_string(),
        "path" => path.to_string(),
        "status" => status.to_string()
    )
    .increment(1);

    histogram!(
        "tempoforge_http_request_duration_seconds",
        "method" => method.to_string(),
        "path" => path.to_string()
    )
    .record(latency_secs);
}
