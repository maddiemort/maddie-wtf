use std::sync::LazyLock;

use metrics::Unit;

pub static REQUESTS_RECEIVED: LazyLock<&'static str> = LazyLock::new(|| {
    let key = "maddie_wtf.requests_received_count";
    metrics::describe_counter!(key, Unit::Count, "Number of HTTP requests received");
    key
});
