//! Database module for digital-product-studio
//!
//! Provides entities, repositories, and compile-time checked queries
//! using sqlx with PostgreSQL.

pub mod entities;
pub mod queries;

pub use entities::*;
pub use queries::*;

use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;

/// Create a new PostgreSQL connection pool
pub async fn create_pool(database_url: &str, max_connections: u32) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect(database_url)
        .await
}

/// Run database migrations
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::entities::*;

    /// Test that all entities can be serialized and deserialized correctly
    #[test]
    fn test_niche_serde_roundtrip() {
        let niche = Niche {
            id: uuid::Uuid::new_v4(),
            name: "Gothic Wall Art".to_string(),
            vertical: NicheVertical::PrintableWallArt,
            demand_signals: serde_json::json!({"search_volume": 10000, "trend": "rising"}),
            competition_score: rust_decimal::Decimal::new(75, 2),
            created_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&niche).unwrap();
        let decoded: Niche = serde_json::from_str(&json).unwrap();
        
        assert_eq!(niche.id, decoded.id);
        assert_eq!(niche.name, decoded.name);
        assert_eq!(niche.vertical, decoded.vertical);
        assert_eq!(niche.competition_score, decoded.competition_score);
    }

    #[test]
    fn test_production_run_serde_roundtrip() {
        let run = ProductionRun {
            id: uuid::Uuid::new_v4(),
            niche_id: uuid::Uuid::new_v4(),
            status: ProductionRunStatus::Generating,
            target_count: 50,
            params: serde_json::json!({"style": "gothic", "count": 50}),
            created_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&run).unwrap();
        let decoded: ProductionRun = serde_json::from_str(&json).unwrap();
        
        assert_eq!(run.id, decoded.id);
        assert_eq!(run.status, decoded.status);
        assert_eq!(run.target_count, decoded.target_count);
    }

    #[test]
    fn test_job_serde_roundtrip() {
        let job = Job {
            id: uuid::Uuid::new_v4(),
            run_id: uuid::Uuid::new_v4(),
            tool: JobTool::CfImageGen,
            command_spec: serde_json::json!({"prompt": "gothic wall art", "dimensions": "2480x3508"}),
            status: JobStatus::Queued,
            attempts: 0,
            queued_at: chrono::Utc::now(),
            started_at: None,
            finished_at: None,
            last_error: None,
            next_attempt_at: None,
            lock_expires_at: None,
        };

        let json = serde_json::to_string(&job).unwrap();
        let decoded: Job = serde_json::from_str(&json).unwrap();
        
        assert_eq!(job.id, decoded.id);
        assert_eq!(job.tool, decoded.tool);
        assert_eq!(job.status, decoded.status);
    }

    #[test]
    fn test_design_variant_serde_roundtrip() {
        let variant = DesignVariant {
            id: uuid::Uuid::new_v4(),
            design_id: uuid::Uuid::new_v4(),
            kind: "upscaled".to_string(),
            s3_key: "raw/run-123/design-456/variant-789.png".to_string(),
            sha256: "a1b2c3d4e5f6".to_string(),
            created_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&variant).unwrap();
        let decoded: DesignVariant = serde_json::from_str(&json).unwrap();
        
        assert_eq!(variant.id, decoded.id);
        assert_eq!(variant.kind, decoded.kind);
        assert_eq!(variant.sha256, decoded.sha256);
    }

    #[test]
    fn test_curation_serde_roundtrip() {
        let curation = Curation {
            id: uuid::Uuid::new_v4(),
            variant_id: uuid::Uuid::new_v4(),
            decision: CurationDecision::Approved,
            reject_reason: None,
            reviewer_id: uuid::Uuid::new_v4(),
            decided_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&curation).unwrap();
        let decoded: Curation = serde_json::from_str(&json).unwrap();
        
        assert_eq!(curation.id, decoded.id);
        assert_eq!(curation.decision, decoded.decision);
        assert_eq!(curation.reject_reason, decoded.reject_reason);
    }

    #[test]
    fn test_product_serde_roundtrip() {
        let product = Product {
            id: uuid::Uuid::new_v4(),
            sku: "WALL-ART-GOTH-001".to_string(),
            title: "Gothic Wall Art Set".to_string(),
            license_tier: LicenseTier::Personal,
            price: rust_decimal::Decimal::new(1999, 2), // $19.99
            s3_prefix: "products/prod-123/v1/".to_string(),
            created_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&product).unwrap();
        let decoded: Product = serde_json::from_str(&json).unwrap();
        
        assert_eq!(product.id, decoded.id);
        assert_eq!(product.sku, decoded.sku);
        assert_eq!(product.license_tier, decoded.license_tier);
        assert_eq!(product.price, decoded.price);
    }

    #[test]
    fn test_platform_listing_serde_roundtrip() {
        let listing = PlatformListing {
            id: uuid::Uuid::new_v4(),
            product_id: uuid::Uuid::new_v4(),
            platform: Platform::Etsy,
            platform_listing_id: "etsy-123456".to_string(),
            state: ListingState::Live,
            price: rust_decimal::Decimal::new(2499, 2), // $24.99
            platform_raw: serde_json::json!({"shop_id": "shop-789"}),
            fee_table_id: Some(uuid::Uuid::new_v4()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&listing).unwrap();
        let decoded: PlatformListing = serde_json::from_str(&json).unwrap();
        
        assert_eq!(listing.id, decoded.id);
        assert_eq!(listing.platform, decoded.platform);
        assert_eq!(listing.state, decoded.state);
        assert_eq!(listing.platform_listing_id, decoded.platform_listing_id);
    }

    #[test]
    fn test_sales_event_serde_roundtrip() {
        let event = SalesEvent {
            id: uuid::Uuid::new_v4(),
            listing_id: uuid::Uuid::new_v4(),
            event_type: "order_paid".to_string(),
            gross: rust_decimal::Decimal::new(2999, 2), // $29.99
            fee: rust_decimal::Decimal::new(200, 2),   // $2.00
            net: rust_decimal::Decimal::new(2799, 2),  // $27.99
            occurred_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let decoded: SalesEvent = serde_json::from_str(&json).unwrap();
        
        assert_eq!(event.id, decoded.id);
        assert_eq!(event.event_type, decoded.event_type);
        assert_eq!(event.gross, decoded.gross);
        assert_eq!(event.fee, decoded.fee);
        assert_eq!(event.net, decoded.net);
    }

    #[test]
    fn test_fee_table_serde_roundtrip() {
        let fee_table = FeeTable {
            id: uuid::Uuid::new_v4(),
            platform: Platform::Etsy,
            rules: serde_json::json!({"listing_fee": 0.20, "transaction_fee_pct": 0.065, "payment_processing_fee_pct": 0.03}),
            effective_from: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&fee_table).unwrap();
        let decoded: FeeTable = serde_json::from_str(&json).unwrap();
        
        assert_eq!(fee_table.id, decoded.id);
        assert_eq!(fee_table.platform, decoded.platform);
        assert_eq!(fee_table.rules, decoded.rules);
    }

    #[test]
    fn test_listing_state_history_serde_roundtrip() {
        let history = ListingStateHistory {
            id: uuid::Uuid::new_v4(),
            listing_id: uuid::Uuid::new_v4(),
            from_state: Some(ListingState::Draft),
            to_state: ListingState::Publishing,
            transition_reason: Some("operator_publish".to_string()),
            actor_id: Some(uuid::Uuid::new_v4()),
            created_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&history).unwrap();
        let decoded: ListingStateHistory = serde_json::from_str(&json).unwrap();
        
        assert_eq!(history.id, decoded.id);
        assert_eq!(history.from_state, decoded.from_state);
        assert_eq!(history.to_state, decoded.to_state);
        assert_eq!(history.transition_reason, decoded.transition_reason);
    }

    #[test]
    fn test_processed_webhook_id_serde_roundtrip() {
        let webhook = ProcessedWebhookId {
            platform_event_id: "evt_123456789".to_string(),
            platform: Platform::Etsy,
            received_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&webhook).unwrap();
        let decoded: ProcessedWebhookId = serde_json::from_str(&json).unwrap();
        
        assert_eq!(webhook.platform_event_id, decoded.platform_event_id);
        assert_eq!(webhook.platform, decoded.platform);
    }

    #[test]
    fn test_all_enum_variants_serde() {
        // Test all enum variants can be serialized/deserialized
        let verticals = vec![
            NicheVertical::PrintableWallArt,
            NicheVertical::PlannerJournalBundles,
            NicheVertical::PartyWeddingPrintables,
        ];
        
        for v in verticals {
            let json = serde_json::to_string(&v).unwrap();
            let decoded: NicheVertical = serde_json::from_str(&json).unwrap();
            assert_eq!(v, decoded);
        }

        let statuses = vec![
            ProductionRunStatus::Pending,
            ProductionRunStatus::Generating,
            ProductionRunStatus::Curating,
            ProductionRunStatus::Assembling,
            ProductionRunStatus::Publishing,
            ProductionRunStatus::Completed,
            ProductionRunStatus::Failed,
            ProductionRunStatus::Cancelled,
        ];
        
        for s in statuses {
            let json = serde_json::to_string(&s).unwrap();
            let decoded: ProductionRunStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(s, decoded);
        }

        let job_tools = vec![
            JobTool::CfImageGen,
            JobTool::PhotocraftCli,
            JobTool::VectorcraftCli,
            JobTool::WebhookProcessor,
            JobTool::ProductAssembly,
            JobTool::PublishListing,
        ];
        
        for t in job_tools {
            let json = serde_json::to_string(&t).unwrap();
            let decoded: JobTool = serde_json::from_str(&json).unwrap();
            assert_eq!(t, decoded);
        }

        let job_statuses = vec![
            JobStatus::Queued,
            JobStatus::Claimed,
            JobStatus::Running,
            JobStatus::Done,
            JobStatus::Failed,
            JobStatus::Dead,
        ];
        
        for s in job_statuses {
            let json = serde_json::to_string(&s).unwrap();
            let decoded: JobStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(s, decoded);
        }

        let curation_decisions = vec![
            CurationDecision::Pending,
            CurationDecision::Approved,
            CurationDecision::Rejected,
        ];
        
        for d in curation_decisions {
            let json = serde_json::to_string(&d).unwrap();
            let decoded: CurationDecision = serde_json::from_str(&json).unwrap();
            assert_eq!(d, decoded);
        }

        let reject_reasons = vec![
            RejectReason::StyleMismatch,
            RejectReason::Artifact,
            RejectReason::License,
            RejectReason::OffNiche,
            RejectReason::Duplicate,
        ];
        
        for r in reject_reasons {
            let json = serde_json::to_string(&r).unwrap();
            let decoded: RejectReason = serde_json::from_str(&json).unwrap();
            assert_eq!(r, decoded);
        }

        let license_tiers = vec![
            LicenseTier::Personal,
            LicenseTier::SmallCommercial,
            LicenseTier::ExtendedCommercial,
        ];
        
        for lt in license_tiers {
            let json = serde_json::to_string(&lt).unwrap();
            let decoded: LicenseTier = serde_json::from_str(&json).unwrap();
            assert_eq!(lt, decoded);
        }

        let platforms = vec![
            Platform::Etsy,
            Platform::LemonSqueezy,
            Platform::Printify,
            Platform::Gelato,
        ];
        
        for p in platforms {
            let json = serde_json::to_string(&p).unwrap();
            let decoded: Platform = serde_json::from_str(&json).unwrap();
            assert_eq!(p, decoded);
        }

        let listing_states = vec![
            ListingState::Draft,
            ListingState::Publishing,
            ListingState::Live,
            ListingState::Updating,
            ListingState::Failed,
            ListingState::Delisting,
            ListingState::Delisted,
            ListingState::Suspended,
        ];
        
        for ls in listing_states {
            let json = serde_json::to_string(&ls).unwrap();
            let decoded: ListingState = serde_json::from_str(&json).unwrap();
            assert_eq!(ls, decoded);
        }
    }
}