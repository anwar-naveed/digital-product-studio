//! Database entities for digital-product-studio
//!
//! These structs map 1:1 to the tables defined in migrations/0001_initial_schema.sql
//! and are designed for compile-time checked queries with sqlx::query!

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Niche verticals for product generation (R1: exactly 3 verticals)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Niche {
    pub id: Uuid,
    pub name: String,
    pub vertical: NicheVertical,
    pub demand_signals: serde_json::Value,
    pub competition_score: Decimal,
    pub created_at: DateTime<Utc>,
}

/// Niche vertical enum - matches CHECK constraint in niches table
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "snake_case")]
pub enum NicheVertical {
    PrintableWallArt,
    PlannerJournalBundles,
    PartyWeddingPrintables,
}

impl std::fmt::Display for NicheVertical {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NicheVertical::PrintableWallArt => write!(f, "printable_wall_art"),
            NicheVertical::PlannerJournalBundles => write!(f, "planner_journal_bundles"),
            NicheVertical::PartyWeddingPrintables => write!(f, "party_wedding_printables"),
        }
    }
}

/// Production run status enum - matches CHECK constraint
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "snake_case")]
pub enum ProductionRunStatus {
    Pending,
    Generating,
    Curating,
    Assembling,
    Publishing,
    Completed,
    Failed,
    Cancelled,
}

/// Production run spawned from niche selection
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductionRun {
    pub id: Uuid,
    pub niche_id: Uuid,
    pub status: ProductionRunStatus,
    pub target_count: i32,
    pub params: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Job tool enum - matches CHECK constraint
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "kebab-case")]
#[sqlx(rename_all = "kebab-case", type_name = "text")]
pub enum JobTool {
    #[sqlx(rename = "cf-image-gen")]
    CfImageGen,
    #[sqlx(rename = "photocraft-cli")]
    PhotocraftCli,
    #[sqlx(rename = "vectorcraft-cli")]
    VectorcraftCli,
    #[sqlx(rename = "webhook-processor")]
    WebhookProcessor,
    #[sqlx(rename = "product-assembly")]
    ProductAssembly,
    #[sqlx(rename = "publish-listing")]
    PublishListing,
}

/// Job status enum - matches CHECK constraint and state machine.
/// The DB column is TEXT + CHECK (not a PG enum), so declare TEXT
/// compatibility explicitly — sqlx otherwise tries to look up a PG
/// enum type named "job_status" at runtime (TypeNotFound).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(rename_all = "snake_case", type_name = "text")]
pub enum JobStatus {
    Queued,
    Claimed,
    Running,
    Done,
    Failed,
    Dead,
}

/// Individual job units in the worker queue (Postgres SKIP LOCKED)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Job {
    pub id: Uuid,
    pub run_id: Uuid,
    pub tool: JobTool,
    pub command_spec: serde_json::Value,
    pub status: JobStatus,
    pub attempts: i32,
    pub queued_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub next_attempt_at: Option<DateTime<Utc>>,
    pub lock_expires_at: Option<DateTime<Utc>>,
}

/// Execution receipts from workers (M2.3 evidence)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct JobReceipt {
    pub id: Uuid,
    pub job_id: Uuid,
    pub command_hash: String,
    pub args_hash: String,
    pub exit_code: i32,
    pub stdout_sha256: String,
    pub stderr_sha256: String,
    pub output_files: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Base designs generated from cf-image-gen
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Design {
    pub id: Uuid,
    pub run_id: Uuid,
    pub base_prompt: String,
    pub s3_prefix_raw: String,
    pub created_at: DateTime<Utc>,
}

/// Variants of designs (upscaled, colorways, formats)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DesignVariant {
    pub id: Uuid,
    pub design_id: Uuid,
    pub kind: String,
    pub s3_key: String,
    pub sha256: String,
    pub created_at: DateTime<Utc>,
}

/// Curation decision enum - matches CHECK constraint
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(rename_all = "snake_case")]
pub enum CurationDecision {
    Pending,
    Approved,
    Rejected,
}

/// Reject reason enum - matches CHECK constraint
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(rename_all = "snake_case")]
pub enum RejectReason {
    StyleMismatch,
    Artifact,
    License,
    OffNiche,
    Duplicate,
}

/// Human curation decisions (approve/reject with taxonomy)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Curation {
    pub id: Uuid,
    pub variant_id: Uuid,
    pub decision: CurationDecision,
    pub reject_reason: Option<RejectReason>,
    pub reviewer_id: Uuid,
    pub decided_at: DateTime<Utc>,
}

/// License tier enum - matches CHECK constraint and R15 pricing ladder
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(rename_all = "snake_case", type_name = "text")]
pub enum LicenseTier {
    Personal,
    SmallCommercial,
    ExtendedCommercial,
}

/// Assembled product bundles with pricing tiers (R15)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Product {
    pub id: Uuid,
    pub sku: String,
    pub title: String,
    pub license_tier: LicenseTier,
    pub price: Decimal,
    pub s3_prefix: String,
    pub created_at: DateTime<Utc>,
}

/// Junction linking product bundles to design variants
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductItem {
    pub id: Uuid,
    pub product_id: Uuid,
    pub variant_id: Uuid,
    pub position: i32,
    pub created_at: DateTime<Utc>,
}

/// Platform enum - matches CHECK constraint
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(rename_all = "snake_case")]
pub enum Platform {
    Etsy,
    LemonSqueezy,
    Printify,
    Gelato,
}

/// Listing state enum - matches CHECK constraint and state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(rename_all = "snake_case")]
pub enum ListingState {
    Draft,
    Publishing,
    Live,
    Updating,
    Failed,
    Delisting,
    Delisted,
    Suspended,
}

/// Published listings on marketplace platforms
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PlatformListing {
    pub id: Uuid,
    pub product_id: Uuid,
    pub platform: Platform,
    pub platform_listing_id: String,
    pub state: ListingState,
    pub price: Decimal,
    pub platform_raw: serde_json::Value,
    pub fee_table_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Revenue tracking per listing with fee calculations
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SalesEvent {
    pub id: Uuid,
    pub listing_id: Uuid,
    pub event_type: String,
    pub gross: Decimal,
    pub fee: Decimal,
    pub net: Decimal,
    pub occurred_at: DateTime<Utc>,
}

/// Platform fee rules for margin calculation (R9)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FeeTable {
    pub id: Uuid,
    pub platform: Platform,
    pub rules: serde_json::Value,
    pub effective_from: DateTime<Utc>,
}

/// Audit trail for listing state machine transitions
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ListingStateHistory {
    pub id: Uuid,
    pub listing_id: Uuid,
    pub from_state: Option<ListingState>,
    pub to_state: ListingState,
    pub transition_reason: Option<String>,
    pub actor_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Idempotency cache for webhook processing (M6.2, TTL 7 days)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProcessedWebhookId {
    pub platform_event_id: String,
    pub platform: Platform,
    pub received_at: DateTime<Utc>,
}
// Display impls for API-facing enums (used in responses and job payloads)

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Platform::Etsy => "etsy",
            Platform::LemonSqueezy => "lemon-squeezy",
            Platform::Printify => "printify",
            Platform::Gelato => "gelato",
        };
        write!(f, "{}", s)
    }
}

impl std::fmt::Display for ListingState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ListingState::Draft => "draft",
            ListingState::Publishing => "publishing",
            ListingState::Live => "live",
            ListingState::Updating => "updating",
            ListingState::Failed => "failed",
            ListingState::Delisting => "delisting",
            ListingState::Delisted => "delisted",
            ListingState::Suspended => "suspended",
        };
        write!(f, "{}", s)
    }
}

impl std::fmt::Display for LicenseTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            LicenseTier::Personal => "personal",
            LicenseTier::SmallCommercial => "small_commercial",
            LicenseTier::ExtendedCommercial => "extended_commercial",
        };
        write!(f, "{}", s)
    }
}

impl std::fmt::Display for CurationDecision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            CurationDecision::Pending => "pending",
            CurationDecision::Approved => "approved",
            CurationDecision::Rejected => "rejected",
        };
        write!(f, "{}", s)
    }
}
