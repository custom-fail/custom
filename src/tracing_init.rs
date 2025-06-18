use opentelemetry_otlp::WithExportConfig;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use std::time::Duration;
use opentelemetry::global::ObjectSafeTracer;
use opentelemetry::KeyValue;
use opentelemetry::logs::LoggerProvider;
use opentelemetry::trace::TracerProvider;
use opentelemetry_sdk::Resource;
use tonic::metadata::{MetadataMap, MetadataValue};
use tonic::service::LayerExt;
use tracing::{warn, Level};

pub fn init() {
    let endpoint = std::env::var("OTEL_ENDPOINT");

    let fmt_layer = tracing_subscriber::fmt::layer();
    let subscriber = tracing_subscriber::Registry::default()
        .with(
            tracing_subscriber::filter::Targets::new()
                .with_targets(vec![
                    ("tower", Level::WARN),
                    ("hyper_util", Level::WARN),
                    ("opentelemetry-otlp", Level::WARN),
                    ("opentelemetry_sdk", Level::WARN),
                    ("tonic", Level::WARN),
                    ("h2", Level::WARN),
                ])
                .with_default(Level::DEBUG)
        )
        .with(tracing_subscriber::filter::LevelFilter::from_level(
            Level::DEBUG,
        ))
        .with(fmt_layer);

    if let Ok(endpoint) = endpoint {
        let span_exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .with_endpoint(endpoint.to_owned())
            .with_timeout(Duration::from_secs(3))
            .build()
            .unwrap();
        let log_exporter = opentelemetry_otlp::LogExporter::builder()
            .with_tonic()
            .with_endpoint(endpoint)
            .with_timeout(Duration::from_secs(3))
            .build()
            .unwrap();

        let resource = Resource::builder()
            .with_attribute(KeyValue::new("service.name", "custom"))
            .build();

        let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
            .with_batch_exporter(span_exporter)
            .with_resource(resource.to_owned())
            .build();
        let logs_provider = opentelemetry_sdk::logs::SdkLoggerProvider::builder()
            .with_batch_exporter(log_exporter)
            .with_resource(resource)
            .build();

        let tracer = tracer_provider.tracer("custom");

        let otel_span_layer = tracing_opentelemetry::layer()
            .with_tracer(tracer);
        let otel_logs_layer = opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge::new(&logs_provider);

        let subscriber = subscriber
            .with(otel_span_layer)
            .with(otel_logs_layer);


        tracing::subscriber::set_global_default(subscriber).unwrap();
    } else {
        tracing::subscriber::set_global_default(subscriber).unwrap();
        warn!("There is no OTEL_ENDPOINT environment variable, running with OpenTelemetry disabled");
    };

}