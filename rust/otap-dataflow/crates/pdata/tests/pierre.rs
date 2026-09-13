//! isolated binary to limit impact of allocator customization

use otel_arrow_dfe_pdata::otlp::OtlpProtoBytes;
use bytes::Bytes;
use otel_arrow_dfe_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::InstrumentationScope;
use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::exponential_histogram_data_point::Buckets;
use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::number_data_point::Value;
use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::summary_data_point::ValueAtQuantile;
use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{
    AggregationTemporality, ExponentialHistogram, ExponentialHistogramDataPoint, Gauge,
    Histogram, HistogramDataPoint, Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics,
    Sum, Summary, SummaryDataPoint, metric::Data,
};
use otel_arrow_dfe_pdata::proto::opentelemetry::resource::v1::Resource;
use prost::Message;
use otel_arrow_dfe_pdata::OtapPayloadHelpers;

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[test]
fn test_pierre_memory() {
    let metrics = ExportMetricsServiceRequest {
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
                                aggregation_temporality: AggregationTemporality::Cumulative
                                    .into(),
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
                                aggregation_temporality: AggregationTemporality::Cumulative
                                    .into(),
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
                                aggregation_temporality: AggregationTemporality::Cumulative
                                    .into(),
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
    };

    let mut buf = Vec::new();
    metrics.encode(&mut buf).unwrap();

    let otlp_bytes = OtlpProtoBytes::ExportMetricsRequest(Bytes::from(buf));

    let _profiler = dhat::Profiler::builder().testing().build();

    let number_of_items = otlp_bytes.num_items();

    let stats = dhat::HeapStats::get();
    assert_eq!(number_of_items, 11);
    dhat::assert!(stats.total_blocks == 0);
    dhat::assert!(stats.max_bytes == 0);

    // From the agent
    // .testing() enables dhat::assert!
    // drop .testing() and use Profiler::new_heap() to get dhat-heap.json to load into viewer
    // dhat::assert!(stats.total_blocks < 500);
    // dhat::assert!(stats.max_bytes < 4 * 1024 * 1024);
    // Looks like failing assertions generate dhat-heap.json, and also print stats
}
