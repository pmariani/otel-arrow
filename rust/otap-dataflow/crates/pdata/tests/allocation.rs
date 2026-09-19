// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This file implements allocation tests which require a customized allocator. The
//! allocator is set for the whole binary. By defining this file as standalone in the
//! tests/ directory, it makes it an independent binary.

use bytes::Bytes;
use otel_arrow_dfe_pdata::OtapPayloadHelpers;
use otel_arrow_dfe_pdata::otlp::OtlpProtoBytes;
use otel_arrow_dfe_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
use otel_arrow_dfe_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
use otel_arrow_dfe_pdata::proto::opentelemetry::collector::trace::v1::ExportTraceServiceRequest;
use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::InstrumentationScope;
use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::{LogRecord, ResourceLogs, ScopeLogs};
use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::exponential_histogram_data_point::Buckets;
use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::number_data_point::Value;
use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::summary_data_point::ValueAtQuantile;
use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{
    AggregationTemporality, ExponentialHistogram, ExponentialHistogramDataPoint, Gauge, Histogram,
    HistogramDataPoint, Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics, Sum, Summary,
    SummaryDataPoint, metric::Data,
};
use otel_arrow_dfe_pdata::proto::opentelemetry::resource::v1::Resource;
use otel_arrow_dfe_pdata::proto::opentelemetry::trace::v1::{
    ResourceSpans, ScopeSpans, Span, Status,
};

use prost::Message;

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn get_test_metrics() -> ExportMetricsServiceRequest {
    ExportMetricsServiceRequest {
        resource_metrics: vec![
            ResourceMetrics {
                resource: Some(Resource::default()),
                scope_metrics: vec![ScopeMetrics {
                    scope: Some(InstrumentationScope::default()),
                    metrics: vec![
                        Metric {
                            name: "gauge_metric".into(),
                            data: Some(Data::Gauge(Gauge {
                                data_points: vec![
                                    NumberDataPoint {
                                        value: Some(Value::AsDouble(1.0)),
                                        ..Default::default()
                                    },
                                    NumberDataPoint {
                                        value: Some(Value::AsDouble(2.0)),
                                        ..Default::default()
                                    },
                                ],
                            })),
                            ..Default::default()
                        },
                        Metric {
                            name: "sum_metric".into(),
                            data: Some(Data::Sum(Sum {
                                data_points: vec![
                                    NumberDataPoint {
                                        value: Some(Value::AsInt(100)),
                                        ..Default::default()
                                    },
                                    NumberDataPoint {
                                        value: Some(Value::AsInt(200)),
                                        ..Default::default()
                                    },
                                    NumberDataPoint {
                                        value: Some(Value::AsInt(300)),
                                        ..Default::default()
                                    },
                                ],
                                aggregation_temporality: AggregationTemporality::Cumulative.into(),
                                is_monotonic: true,
                            })),
                            ..Default::default()
                        },
                        Metric {
                            name: "histogram_metric".into(),
                            data: Some(Data::Histogram(Histogram {
                                data_points: vec![
                                    HistogramDataPoint {
                                        count: 10,
                                        sum: Some(100.0),
                                        bucket_counts: vec![2, 5, 3],
                                        explicit_bounds: vec![10.0, 50.0],
                                        ..Default::default()
                                    },
                                    HistogramDataPoint {
                                        count: 20,
                                        sum: Some(200.0),
                                        bucket_counts: vec![5, 10, 5],
                                        explicit_bounds: vec![10.0, 50.0],
                                        ..Default::default()
                                    },
                                ],
                                aggregation_temporality: AggregationTemporality::Cumulative.into(),
                            })),
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                }],
                ..Default::default()
            },
            ResourceMetrics {
                resource: Some(Resource::default()),
                scope_metrics: vec![ScopeMetrics {
                    scope: Some(InstrumentationScope::default()),
                    metrics: vec![
                        Metric {
                            name: "exp_histogram_metric".into(),
                            data: Some(Data::ExponentialHistogram(ExponentialHistogram {
                                data_points: vec![
                                    ExponentialHistogramDataPoint {
                                        count: 15,
                                        sum: Some(150.0),
                                        scale: 1,
                                        zero_count: 1,
                                        positive: Some(Buckets {
                                            offset: 0,
                                            bucket_counts: vec![3, 5, 7],
                                        }),
                                        negative: Some(Buckets {
                                            offset: 0,
                                            bucket_counts: vec![1, 2],
                                        }),
                                        ..Default::default()
                                    },
                                    ExponentialHistogramDataPoint {
                                        count: 25,
                                        sum: Some(250.0),
                                        scale: 1,
                                        zero_count: 2,
                                        positive: Some(Buckets {
                                            offset: 0,
                                            bucket_counts: vec![5, 10, 8],
                                        }),
                                        ..Default::default()
                                    },
                                ],
                                aggregation_temporality: AggregationTemporality::Cumulative.into(),
                            })),
                            ..Default::default()
                        },
                        Metric {
                            name: "summary_metric".into(),
                            data: Some(Data::Summary(Summary {
                                data_points: vec![
                                    SummaryDataPoint {
                                        count: 100,
                                        sum: 1000.0,
                                        quantile_values: vec![
                                            ValueAtQuantile {
                                                quantile: 0.5,
                                                value: 10.0,
                                            },
                                            ValueAtQuantile {
                                                quantile: 0.95,
                                                value: 50.0,
                                            },
                                        ],
                                        ..Default::default()
                                    },
                                    SummaryDataPoint {
                                        count: 200,
                                        sum: 2000.0,
                                        quantile_values: vec![ValueAtQuantile {
                                            quantile: 0.5,
                                            value: 20.0,
                                        }],
                                        ..Default::default()
                                    },
                                ],
                            })),
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                }],
                ..Default::default()
            },
        ],
    }
}

fn get_test_logs() -> ExportLogsServiceRequest {
    ExportLogsServiceRequest {
        resource_logs: vec![
            ResourceLogs {
                resource: Some(Resource::default()),
                scope_logs: vec![ScopeLogs {
                    scope: Some(InstrumentationScope::default()),
                    log_records: vec![LogRecord::default(), LogRecord::default()],
                    ..Default::default()
                }],
                ..Default::default()
            },
            ResourceLogs {
                resource: Some(Resource::default()),
                scope_logs: vec![
                    ScopeLogs {
                        scope: Some(InstrumentationScope::default()),
                        log_records: vec![LogRecord::default()],
                        ..Default::default()
                    },
                    ScopeLogs {
                        scope: Some(InstrumentationScope::default()),
                        log_records: vec![LogRecord::default(), LogRecord::default()],
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
        ],
    }
}

fn get_test_traces() -> ExportTraceServiceRequest {
    ExportTraceServiceRequest {
        resource_spans: vec![ResourceSpans {
            resource: Some(Resource::default()),
            scope_spans: vec![ScopeSpans {
                scope: Some(InstrumentationScope::default()),
                spans: vec![
                    Span {
                        trace_id: [12].into(),
                        span_id: [12].into(),
                        trace_state: "test".into(),
                        parent_span_id: [12].into(),
                        flags: 12,
                        name: "test".into(),
                        kind: 12,
                        start_time_unix_nano: 12,
                        end_time_unix_nano: 12,
                        attributes: vec![],
                        dropped_attributes_count: 0,
                        events: vec![],
                        dropped_events_count: 0,
                        links: vec![],
                        dropped_links_count: 0,
                        status: Some(Status::default()),
                    },
                    Span {
                        trace_id: [13].into(),
                        span_id: [13].into(),
                        trace_state: "test2".into(),
                        parent_span_id: [13].into(),
                        flags: 13,
                        name: "test2".into(),
                        kind: 13,
                        start_time_unix_nano: 13,
                        end_time_unix_nano: 13,
                        attributes: vec![],
                        dropped_attributes_count: 0,
                        events: vec![],
                        dropped_events_count: 0,
                        links: vec![],
                        dropped_links_count: 0,
                        status: Some(Status::default()),
                    },
                ],
                ..Default::default()
            }],
            ..Default::default()
        }],
    }
}

#[cfg(test)]
mod test_allocation {
    use super::*;

    #[test]
    #[ignore]
    fn test_metrics_num_items_should_not_allocate() {
        let metrics = get_test_metrics();

        let mut buf = Vec::new();
        metrics.encode(&mut buf).unwrap();

        let otlp_bytes = OtlpProtoBytes::ExportMetricsRequest(Bytes::from(buf));

        let _profiler = dhat::Profiler::builder().testing().build();

        let number_of_items = otlp_bytes.num_items();

        let stats = dhat::HeapStats::get();
        dhat::assert!(stats.total_blocks == 0);
        dhat::assert!(stats.max_bytes == 0);
        assert_eq!(number_of_items, 11);
    }

    #[test]
    #[ignore]
    fn test_logs_num_items_should_not_allocate() {
        let logs = get_test_logs();

        let mut buf = Vec::new();
        logs.encode(&mut buf).unwrap();

        let otlp_bytes = OtlpProtoBytes::ExportLogsRequest(Bytes::from(buf));

        let _profiler = dhat::Profiler::builder().testing().build();

        let number_of_items = otlp_bytes.num_items();

        let stats = dhat::HeapStats::get();
        dhat::assert!(stats.total_blocks == 0);
        dhat::assert!(stats.max_bytes == 0);
        assert_eq!(number_of_items, 5);
    }

    #[test]
    // #[ignore]
    fn test_traces_num_items_should_not_allocate() {
        let traces = get_test_traces();

        let mut buf = Vec::new();
        traces.encode(&mut buf).unwrap();

        let otlp_bytes = OtlpProtoBytes::ExportTracesRequest(Bytes::from(buf));

        let _profiler = dhat::Profiler::builder().testing().build();

        let number_of_items = otlp_bytes.num_items();

        let stats = dhat::HeapStats::get();
        dhat::assert!(stats.total_blocks == 0);
        dhat::assert!(stats.max_bytes == 0);
        assert_eq!(number_of_items, 2);
    }
}

