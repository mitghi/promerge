# promerge

Promerge provides minimalistic and easy to use API to parse and manipulate Prometheus metrics. A simple usecase could be
collecting metrics from several servers and combining and exposing them as single metrics endpoint, adding namespace and
custom (Key,Value) pairs to each Prometheus exposition line.

Example:

```rust
use promerge::promerge::Context;

fn main() {
    // example from: https://prometheus.io/docs/instrumenting/exposition_formats/
    let metrics = r#"# HELP http_requests_total The total number of HTTP requests.
# TYPE http_requests_total counter
http_requests_total{method="post",code="200"} 1027 1395066363000
http_requests_total{method="post",code="400"}    3 1395066363000

# Escaping in label values:
msdos_file_access_time_seconds{path="C:\\DIR\\FILE.TXT",error="Cannot find file:\n\"FILE.TXT\""} 1.458255915e9

# Minimalistic line:
metric_without_timestamp_and_labels 12.47

# A weird metric from before the epoch:
something_weird{problem="division by zero"} +Inf -3982045

# A histogram, which has a pretty complex representation in the text format:
# HELP http_request_duration_seconds A histogram of the request duration.
# TYPE http_request_duration_seconds histogram
http_request_duration_seconds_bucket{le="0.05"} 24054
http_request_duration_seconds_bucket{le="0.1"} 33444
http_request_duration_seconds_bucket{le="0.2"} 100392
http_request_duration_seconds_bucket{le="0.5"} 129389
http_request_duration_seconds_bucket{le="1"} 133988
http_request_duration_seconds_bucket{le="+Inf"} 144320
http_request_duration_seconds_sum 53423
http_request_duration_seconds_count 144320

# Finally a summary, which has a complex representation, too:
# HELP rpc_duration_seconds A summary of the RPC duration in seconds.
# TYPE rpc_duration_seconds summary
rpc_duration_seconds{quantile="0.01"} 3102
rpc_duration_seconds{quantile="0.05"} 3272
rpc_duration_seconds{quantile="0.5"} 4773
rpc_duration_seconds{quantile="0.9"} 9001
rpc_duration_seconds{quantile="0.99"} 76656
rpc_duration_seconds_sum 1.7560473e+07
rpc_duration_seconds_count 2693"#;

    // add new (k,v) pairs into Prometheus metric exposition
    let pairs = [("custom_key".into(), "custom_value".into())];
    // add a prefix to metrics
    let mut metrics = Context::with_prefix_and_pairs(&metrics, "some_prefix_", &pairs);
	
    // parse and evaluate
    {
	let ctx = &mut metrics;
	match ctx.run() {
	    Ok(output) => {
		println!("first run: {}", output);
	    },
	    Err(err) => {
		eprintln!("{}", err);
	    }
	};
    }
	
    // append new metrics with difference prefix
    {
        let block = r#"# New minimalistic line:
new_metric_without_timestamp_and_labels 24.81"#;	
	let ctx = &mut metrics;
	match ctx.combine_with_prefix(block, "second_prefix_") {
	    Ok(output) => {
		println!("second run: {}", output);
	    },
	    Err(err) => {
		eprintln!("{}", err);
	    }
	};
    }

    /* Final Output ->
    
    # HELP some_prefix_http_requests_total The total number of HTTP requests.
    # TYPE some_prefix_http_requests_total counter
    some_prefix_http_requests_total{method="post",code="200",custom_key="custom_value"} 1027 1395066363000
    some_prefix_http_requests_total{method="post",code="400",custom_key="custom_value"} 3 1395066363000

    # Escaping in label values:
    some_prefix_msdos_file_access_time_seconds{path="C:\\DIR\\FILE.TXT",error="Cannot find file:\n\"FILE.TXT\"",custom_key="custom_value"} 1.458255915e9

    # Minimalistic line:
    some_prefix_metric_without_timestamp_and_labels{custom_key="custom_value"} 12.47

    # A weird metric from before the epoch:
    some_prefix_something_weird{problem="division by zero",custom_key="custom_value"} +Inf -3982045

    # A histogram, which has a pretty complex representation in the text format:
    # HELP some_prefix_http_request_duration_seconds A histogram of the request duration.
    # TYPE some_prefix_http_request_duration_seconds histogram
    some_prefix_http_request_duration_seconds_bucket{le="0.05",custom_key="custom_value"} 24054
    some_prefix_http_request_duration_seconds_bucket{le="0.1",custom_key="custom_value"} 33444
    some_prefix_http_request_duration_seconds_bucket{le="0.2",custom_key="custom_value"} 100392
    some_prefix_http_request_duration_seconds_bucket{le="0.5",custom_key="custom_value"} 129389
    some_prefix_http_request_duration_seconds_bucket{le="1",custom_key="custom_value"} 133988
    some_prefix_http_request_duration_seconds_bucket{le="+Inf",custom_key="custom_value"} 144320
    some_prefix_http_request_duration_seconds_sum 53423
    some_prefix_http_request_duration_seconds_count 144320

    # Finally a summary, which has a complex representation, too:
    # HELP some_prefix_rpc_duration_seconds A summary of the RPC duration in seconds.
    # TYPE some_prefix_rpc_duration_seconds summary
    some_prefix_rpc_duration_seconds{quantile="0.01",custom_key="custom_value"} 3102
    some_prefix_rpc_duration_seconds{quantile="0.05",custom_key="custom_value"} 3272
    some_prefix_rpc_duration_seconds{quantile="0.5",custom_key="custom_value"} 4773
    some_prefix_rpc_duration_seconds{quantile="0.9",custom_key="custom_value"} 9001
    some_prefix_rpc_duration_seconds{quantile="0.99",custom_key="custom_value"} 76656
    some_prefix_rpc_duration_seconds_sum 1.7560473e+07
    some_prefix_rpc_duration_seconds_count 2693

    # New minimalistic line:
    second_prefix_new_metric_without_timestamp_and_labels 24.81
    
     */
}
```
