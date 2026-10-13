//! Audit logger for pipeline-core
//!
//! Emits audit events to OTLP endpoint (Loki) per M1.3 requirement.
//! All mutating routes must emit audit events.

use opentelemetry::{
    global,
    trace::{Span, SpanKind, TraceContextExt, Tracer, TracerProvider},
    Context, KeyValue,
};
use opentelemetry_otlp::{SpanExporter, WithExportConfig};
use opentelemetry_sdk::trace::{Sampler, TracerProvider as SdkTracerProvider};
use opentelemetry_sdk::runtime;
use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::{
    config::AuditConfig,
    error::{AppError, Result},
};

/// Audit logger for mutating operations
pub struct AuditLogger {
    tracer: opentelemetry_sdk::trace::Tracer,
}

impl AuditLogger {
    /// Create a new audit logger
    pub async fn new(config: &AuditConfig) -> Result<Self> {
        // Initialize OTLP trace exporter (opentelemetry-otlp 0.27 API)
        let exporter = SpanExporter::builder()
            .with_tonic()
            .with_endpoint(&config.otlp_endpoint)
            .build()
            .map_err(|e| AppError::Internal(format!("OTLP exporter init failed: {e}")))?;

        let provider = SdkTracerProvider::builder()
            .with_sampler(Sampler::AlwaysOn)
            .with_batch_exporter(exporter, runtime::Tokio)
            .build();

        let tracer = provider.tracer(config.service_name.clone());
        global::set_tracer_provider(provider);
        
        // Also initialize tracing subscriber for local logging
        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info"));
        
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt::layer().json())
            .init();
        
        info!("Audit logger initialized with OTLP endpoint: {}", config.otlp_endpoint);
        
        Ok(Self { tracer })
    }
    
    /// Log an audit event
    /// 
    /// Per docs/11-architecture.md M1.3: All mutations emit audit events
    pub fn log(&self, event_type: &str, payload: serde_json::Value) {
        let mut span = self.tracer.start_with_context(
            format!("audit.{}", event_type),
            &Context::current(),
        );
        
        span.set_attribute(KeyValue::new("audit.event_type", event_type.to_string()));
        span.set_attribute(KeyValue::new("audit.payload", payload.to_string()));
        span.set_attribute(KeyValue::new("audit.timestamp", chrono::Utc::now().to_rfc3339()));
        
        // Also log locally for immediate visibility
        info!(
            event_type = %event_type,
            payload = %payload,
            "Audit event"
        );
        
        span.end();
    }
    
    /// Log an audit event with user context
    pub fn log_with_user(&self, event_type: &str, payload: serde_json::Value, user_id: uuid::Uuid) {
        let mut enriched_payload = payload.clone();
        if let Some(obj) = enriched_payload.as_object_mut() {
            obj.insert("actor_id".to_string(), serde_json::Value::String(user_id.to_string()));
        }
        
        self.log(event_type, enriched_payload);
    }
}

/// Initialize tracing for the application
pub fn init_tracing(config: &crate::config::TracingConfig) -> Result<()> {
    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint(&config.otlp_endpoint)
        .build()
        .map_err(|e| AppError::Internal(format!("OTLP exporter init failed: {e}")))?;

    let tracer_provider = SdkTracerProvider::builder()
        .with_sampler(Sampler::AlwaysOn)
        .with_batch_exporter(exporter, runtime::Tokio)
        .build();

    global::set_tracer_provider(tracer_provider);
    
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.level));
    
    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().json())
        .init();
    
    info!("Tracing initialized with OTLP endpoint: {}", config.otlp_endpoint);
    Ok(())
}