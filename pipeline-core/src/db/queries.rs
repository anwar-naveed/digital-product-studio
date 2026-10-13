//! Compile-time checked database queries using sqlx::query!
//!
//! All queries use sqlx::query! or sqlx::query_as! for compile-time validation
//! against the PostgreSQL schema. Requires DATABASE_URL at compile time.

use crate::db::entities::*;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::{PgPool, FromRow, Row};
use uuid::Uuid;

/// Repository for niches table
pub struct NicheRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> NicheRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Create a new niche
    pub async fn create(
        &self,
        name: &str,
        vertical: NicheVertical,
        demand_signals: serde_json::Value,
        competition_score: Decimal,
    ) -> Result<Niche, sqlx::Error> {
        let niche = sqlx::query_as!(
            Niche,
            r#"
            INSERT INTO niches (name, vertical, demand_signals, competition_score)
            VALUES ($1, $2, $3, $4)
            RETURNING id, name, vertical as "vertical: NicheVertical", demand_signals, competition_score, created_at
            "#,
            name,
            vertical as NicheVertical,
            demand_signals,
            competition_score
        )
        .fetch_one(self.pool)
        .await?;
        Ok(niche)
    }

    /// Get niche by ID
    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<Niche>, sqlx::Error> {
        let niche = sqlx::query_as!(
            Niche,
            r#"
            SELECT id, name, vertical as "vertical: NicheVertical", demand_signals, competition_score, created_at
            FROM niches
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(self.pool)
        .await?;
        Ok(niche)
    }

    /// List all niches with optional vertical filter
    pub async fn list(&self, vertical: Option<NicheVertical>, limit: i64, offset: i64) -> Result<Vec<Niche>, sqlx::Error> {
        let niches = if let Some(v) = vertical {
            sqlx::query_as!(
                Niche,
                r#"
                SELECT id, name, vertical as "vertical: NicheVertical", demand_signals, competition_score, created_at
                FROM niches
                WHERE vertical = $1
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3
                "#,
                v as NicheVertical,
                limit,
                offset
            )
            .fetch_all(self.pool)
            .await?
        } else {
            sqlx::query_as!(
                Niche,
                r#"
                SELECT id, name, vertical as "vertical: NicheVertical", demand_signals, competition_score, created_at
                FROM niches
                ORDER BY created_at DESC
                LIMIT $1 OFFSET $2
                "#,
                limit,
                offset
            )
            .fetch_all(self.pool)
            .await?
        };
        Ok(niches)
    }

    /// Count niches
    pub async fn count(&self, vertical: Option<NicheVertical>) -> Result<i64, sqlx::Error> {
        let count = if let Some(v) = vertical {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM niches WHERE vertical = $1",
                v as NicheVertical
            )
            .fetch_one(self.pool)
            .await?
        } else {
            sqlx::query_scalar!("SELECT COUNT(*) FROM niches")
            .fetch_one(self.pool)
            .await?
        };
        Ok(count.unwrap_or(0))
    }
}

/// Repository for production_runs table
pub struct ProductionRunRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> ProductionRunRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Create a new production run
    pub async fn create(
        &self,
        niche_id: Uuid,
        target_count: i32,
        params: serde_json::Value,
    ) -> Result<ProductionRun, sqlx::Error> {
        let run = sqlx::query_as!(
            ProductionRun,
            r#"
            INSERT INTO production_runs (niche_id, status, target_count, params)
            VALUES ($1, 'pending', $2, $3)
            RETURNING id, niche_id, status as "status: ProductionRunStatus", target_count, params, created_at
            "#,
            niche_id,
            target_count,
            params
        )
        .fetch_one(self.pool)
        .await?;
        Ok(run)
    }

    /// Get production run by ID
    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<ProductionRun>, sqlx::Error> {
        let run = sqlx::query_as!(
            ProductionRun,
            r#"
            SELECT id, niche_id, status as "status: ProductionRunStatus", target_count, params, created_at
            FROM production_runs
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(self.pool)
        .await?;
        Ok(run)
    }

    /// List production runs with optional status filter
    pub async fn list(&self, status: Option<ProductionRunStatus>, limit: i64, offset: i64) -> Result<Vec<ProductionRun>, sqlx::Error> {
        let runs = if let Some(s) = status {
            sqlx::query_as!(
                ProductionRun,
                r#"
                SELECT id, niche_id, status as "status: ProductionRunStatus", target_count, params, created_at
                FROM production_runs
                WHERE status = $1
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3
                "#,
                s as ProductionRunStatus,
                limit,
                offset
            )
            .fetch_all(self.pool)
            .await?
        } else {
            sqlx::query_as!(
                ProductionRun,
                r#"
                SELECT id, niche_id, status as "status: ProductionRunStatus", target_count, params, created_at
                FROM production_runs
                ORDER BY created_at DESC
                LIMIT $1 OFFSET $2
                "#,
                limit,
                offset
            )
            .fetch_all(self.pool)
            .await?
        };
        Ok(runs)
    }

    /// Update production run status
    pub async fn update_status(&self, id: Uuid, status: ProductionRunStatus) -> Result<ProductionRun, sqlx::Error> {
        let run = sqlx::query_as!(
            ProductionRun,
            r#"
            UPDATE production_runs
            SET status = $2
            WHERE id = $1
            RETURNING id, niche_id, status as "status: ProductionRunStatus", target_count, params, created_at
            "#,
            id,
            status as ProductionRunStatus
        )
        .fetch_one(self.pool)
        .await?;
        Ok(run)
    }
}

/// Repository for jobs table
pub struct JobRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> JobRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Create a new job
    pub async fn create(
        &self,
        run_id: Uuid,
        tool: JobTool,
        command_spec: serde_json::Value,
    ) -> Result<Job, sqlx::Error> {
        let job = sqlx::query_as!(
            Job,
            r#"
            INSERT INTO jobs (run_id, tool, command_spec, status)
            VALUES ($1, $2, $3, 'queued')
            RETURNING id, run_id, tool as "tool: JobTool", command_spec, status as "status: JobStatus", attempts, queued_at, started_at, finished_at, last_error, next_attempt_at, lock_expires_at
            "#,
            run_id,
            tool as JobTool,
            command_spec
        )
        .fetch_one(self.pool)
        .await?;
        Ok(job)
    }

    /// Claim a job using SKIP LOCKED (worker queue pattern)
    pub async fn claim_one(
        &self,
        lease_seconds: i64,
    ) -> Result<Option<Job>, sqlx::Error> {
        let now = Utc::now();
        let lock_expires_at = now + chrono::Duration::seconds(lease_seconds);
        
        let job = sqlx::query_as!(
            Job,
            r#"
            UPDATE jobs
            SET status = 'claimed',
                started_at = $1,
                lock_expires_at = $2,
                attempts = attempts + 1
            WHERE id = (
                SELECT id FROM jobs
                WHERE status = 'queued'
                  AND (next_attempt_at IS NULL OR next_attempt_at <= $1)
                ORDER BY queued_at
                FOR UPDATE SKIP LOCKED
                LIMIT 1
            )
            RETURNING id, run_id, tool as "tool: JobTool", command_spec, status as "status: JobStatus", attempts, queued_at, started_at, finished_at, last_error, next_attempt_at, lock_expires_at
            "#,
            now,
            lock_expires_at
        )
        .fetch_optional(self.pool)
        .await?;
        Ok(job)
    }

    /// Mark job as running
    pub async fn mark_running(&self, id: Uuid) -> Result<Job, sqlx::Error> {
        let job = sqlx::query_as!(
            Job,
            r#"
            UPDATE jobs
            SET status = 'running'
            WHERE id = $1
            RETURNING id, run_id, tool as "tool: JobTool", command_spec, status as "status: JobStatus", attempts, queued_at, started_at, finished_at, last_error, next_attempt_at, lock_expires_at
            "#,
            id
        )
        .fetch_one(self.pool)
        .await?;
        Ok(job)
    }

    /// Mark job as done
    pub async fn mark_done(&self, id: Uuid) -> Result<Job, sqlx::Error> {
        let job = sqlx::query_as!(
            Job,
            r#"
            UPDATE jobs
            SET status = 'done',
                finished_at = $2,
                lock_expires_at = NULL
            WHERE id = $1
            RETURNING id, run_id, tool as "tool: JobTool", command_spec, status as "status: JobStatus", attempts, queued_at, started_at, finished_at, last_error, next_attempt_at, lock_expires_at
            "#,
            id,
            Utc::now()
        )
        .fetch_one(self.pool)
        .await?;
        Ok(job)
    }

    /// Mark job as failed with error
    pub async fn mark_failed(&self, id: Uuid, error: &str, max_retries: i32) -> Result<Job, sqlx::Error> {
        let job = sqlx::query_as!(
            Job,
            r#"
            UPDATE jobs
            SET status = CASE 
                WHEN attempts >= $3 THEN 'dead'
                ELSE 'failed'
            END,
                last_error = $2,
                finished_at = CASE WHEN attempts >= $3 THEN $4::timestamptz ELSE NULL END,
                next_attempt_at = CASE 
                    WHEN attempts < $3 THEN $4::timestamptz + (interval '30 seconds' * (2 ^ attempts))
                    ELSE NULL
                END,
                lock_expires_at = NULL
            WHERE id = $1
            RETURNING id, run_id, tool as "tool: JobTool", command_spec, status as "status: JobStatus", attempts, queued_at, started_at, finished_at, last_error, next_attempt_at, lock_expires_at
            "#,
            id,
            error,
            max_retries,
            Utc::now()
        )
        .fetch_one(self.pool)
        .await?;
        Ok(job)
    }

    /// Get job by ID
    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<Job>, sqlx::Error> {
        let job = sqlx::query_as!(
            Job,
            r#"
            SELECT id, run_id, tool as "tool: JobTool", command_spec, status as "status: JobStatus", attempts, queued_at, started_at, finished_at, last_error, next_attempt_at, lock_expires_at
            FROM jobs
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(self.pool)
        .await?;
        Ok(job)
    }

    /// List jobs for a run
    pub async fn list_by_run(&self, run_id: Uuid) -> Result<Vec<Job>, sqlx::Error> {
        let jobs = sqlx::query_as!(
            Job,
            r#"
            SELECT id, run_id, tool as "tool: JobTool", command_spec, status as "status: JobStatus", attempts, queued_at, started_at, finished_at, last_error, next_attempt_at, lock_expires_at
            FROM jobs
            WHERE run_id = $1
            ORDER BY queued_at
            "#,
            run_id
        )
        .fetch_all(self.pool)
        .await?;
        Ok(jobs)
    }
}

/// Repository for job_receipts table
pub struct JobReceiptRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> JobReceiptRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Create a job receipt
    pub async fn create(
        &self,
        job_id: Uuid,
        command_hash: &str,
        args_hash: &str,
        exit_code: i32,
        stdout_sha256: &str,
        stderr_sha256: &str,
        output_files: serde_json::Value,
    ) -> Result<JobReceipt, sqlx::Error> {
        let receipt = sqlx::query_as!(
            JobReceipt,
            r#"
            INSERT INTO job_receipts (job_id, command_hash, args_hash, exit_code, stdout_sha256, stderr_sha256, output_files)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, job_id, command_hash, args_hash, exit_code, stdout_sha256, stderr_sha256, output_files, created_at
            "#,
            job_id,
            command_hash,
            args_hash,
            exit_code,
            stdout_sha256,
            stderr_sha256,
            output_files
        )
        .fetch_one(self.pool)
        .await?;
        Ok(receipt)
    }

    /// Get receipt by job ID
    pub async fn get_by_job_id(&self, job_id: Uuid) -> Result<Option<JobReceipt>, sqlx::Error> {
        let receipt = sqlx::query_as!(
            JobReceipt,
            r#"
            SELECT id, job_id, command_hash, args_hash, exit_code, stdout_sha256, stderr_sha256, output_files, created_at
            FROM job_receipts
            WHERE job_id = $1
            "#,
            job_id
        )
        .fetch_optional(self.pool)
        .await?;
        Ok(receipt)
    }
}

/// Repository for designs table
pub struct DesignRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> DesignRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Create a new design
    pub async fn create(
        &self,
        run_id: Uuid,
        base_prompt: &str,
        s3_prefix_raw: &str,
    ) -> Result<Design, sqlx::Error> {
        let design = sqlx::query_as!(
            Design,
            r#"
            INSERT INTO designs (run_id, base_prompt, s3_prefix_raw)
            VALUES ($1, $2, $3)
            RETURNING id, run_id, base_prompt, s3_prefix_raw, created_at
            "#,
            run_id,
            base_prompt,
            s3_prefix_raw
        )
        .fetch_one(self.pool)
        .await?;
        Ok(design)
    }

    /// Get design by ID
    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<Design>, sqlx::Error> {
        let design = sqlx::query_as!(
            Design,
            r#"
            SELECT id, run_id, base_prompt, s3_prefix_raw, created_at
            FROM designs
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(self.pool)
        .await?;
        Ok(design)
    }

    /// List designs for a run
    pub async fn list_by_run(&self, run_id: Uuid) -> Result<Vec<Design>, sqlx::Error> {
        let designs = sqlx::query_as!(
            Design,
            r#"
            SELECT id, run_id, base_prompt, s3_prefix_raw, created_at
            FROM designs
            WHERE run_id = $1
            ORDER BY created_at
            "#,
            run_id
        )
        .fetch_all(self.pool)
        .await?;
        Ok(designs)
    }
}

/// Repository for design_variants table
pub struct DesignVariantRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> DesignVariantRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Create a new design variant
    pub async fn create(
        &self,
        design_id: Uuid,
        kind: &str,
        s3_key: &str,
        sha256: &str,
    ) -> Result<DesignVariant, sqlx::Error> {
        let variant = sqlx::query_as!(
            DesignVariant,
            r#"
            INSERT INTO design_variants (design_id, kind, s3_key, sha256)
            VALUES ($1, $2, $3, $4)
            RETURNING id, design_id, kind, s3_key, sha256, created_at
            "#,
            design_id,
            kind,
            s3_key,
            sha256
        )
        .fetch_one(self.pool)
        .await?;
        Ok(variant)
    }

    /// Get variant by ID
    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<DesignVariant>, sqlx::Error> {
        let variant = sqlx::query_as!(
            DesignVariant,
            r#"
            SELECT id, design_id, kind, s3_key, sha256, created_at
            FROM design_variants
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(self.pool)
        .await?;
        Ok(variant)
    }

    /// List variants for a design
    pub async fn list_by_design(&self, design_id: Uuid) -> Result<Vec<DesignVariant>, sqlx::Error> {
        let variants = sqlx::query_as!(
            DesignVariant,
            r#"
            SELECT id, design_id, kind, s3_key, sha256, created_at
            FROM design_variants
            WHERE design_id = $1
            ORDER BY created_at
            "#,
            design_id
        )
        .fetch_all(self.pool)
        .await?;
        Ok(variants)
    }

    /// Find variant by SHA256 (deduplication)
    pub async fn find_by_sha256(&self, sha256: &str) -> Result<Option<DesignVariant>, sqlx::Error> {
        let variant = sqlx::query_as!(
            DesignVariant,
            r#"
            SELECT id, design_id, kind, s3_key, sha256, created_at
            FROM design_variants
            WHERE sha256 = $1
            "#,
            sha256
        )
        .fetch_optional(self.pool)
        .await?;
        Ok(variant)
    }
}

/// Repository for curations table
pub struct CurationRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> CurationRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Create a new curation (pending)
    pub async fn create_pending(&self, variant_id: Uuid, reviewer_id: Uuid) -> Result<Curation, sqlx::Error> {
        let curation = sqlx::query_as!(
            Curation,
            r#"
            INSERT INTO curations (variant_id, decision, reviewer_id)
            VALUES ($1, 'pending', $2)
            RETURNING id, variant_id, decision as "decision: CurationDecision", reject_reason as "reject_reason: RejectReason", reviewer_id, decided_at
            "#,
            variant_id,
            reviewer_id
        )
        .fetch_one(self.pool)
        .await?;
        Ok(curation)
    }

    /// Approve a variant
    pub async fn approve(&self, variant_id: Uuid, reviewer_id: Uuid) -> Result<Curation, sqlx::Error> {
        let _reviewer_id = reviewer_id;
        let curation = sqlx::query_as!(
            Curation,
            r#"
            UPDATE curations
            SET decision = 'approved', decided_at = $2
            WHERE variant_id = $1
            RETURNING id, variant_id, decision as "decision: CurationDecision", reject_reason as "reject_reason: RejectReason", reviewer_id, decided_at
            "#,
            variant_id,
            Utc::now()
        )
        .fetch_one(self.pool)
        .await?;
        Ok(curation)
    }

    /// Reject a variant
    pub async fn reject(&self, variant_id: Uuid, reviewer_id: Uuid, reason: RejectReason) -> Result<Curation, sqlx::Error> {
        let _reviewer_id = reviewer_id;
        let curation = sqlx::query_as!(
            Curation,
            r#"
            UPDATE curations
            SET decision = 'rejected', reject_reason = $2, decided_at = $3
            WHERE variant_id = $1
            RETURNING id, variant_id, decision as "decision: CurationDecision", reject_reason as "reject_reason: RejectReason", reviewer_id, decided_at
            "#,
            variant_id,
            reason as RejectReason,
            Utc::now()
        )
        .fetch_one(self.pool)
        .await?;
        Ok(curation)
    }

    /// Get pending curations for queue
    pub async fn get_pending_queue(&self, limit: i64, offset: i64) -> Result<Vec<Curation>, sqlx::Error> {
        let curations = sqlx::query_as!(
            Curation,
            r#"
            SELECT id, variant_id, decision as "decision: CurationDecision", reject_reason as "reject_reason: RejectReason", reviewer_id, decided_at
            FROM curations
            WHERE decision = 'pending'
            ORDER BY decided_at
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(self.pool)
        .await?;
        Ok(curations)
    }
}

/// Repository for products table
pub struct ProductRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> ProductRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Create a new product
    pub async fn create(
        &self,
        sku: &str,
        title: &str,
        license_tier: LicenseTier,
        price: Decimal,
        s3_prefix: &str,
    ) -> Result<Product, sqlx::Error> {
        let product = sqlx::query_as!(
            Product,
            r#"
            INSERT INTO products (sku, title, license_tier, price, s3_prefix)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, sku, title, license_tier as "license_tier: LicenseTier", price, s3_prefix, created_at
            "#,
            sku,
            title,
            license_tier as LicenseTier,
            price,
            s3_prefix
        )
        .fetch_one(self.pool)
        .await?;
        Ok(product)
    }

    /// Get product by ID
    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<Product>, sqlx::Error> {
        let product = sqlx::query_as!(
            Product,
            r#"
            SELECT id, sku, title, license_tier as "license_tier: LicenseTier", price, s3_prefix, created_at
            FROM products
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(self.pool)
        .await?;
        Ok(product)
    }

    /// List products with optional license tier filter
    pub async fn list(&self, license_tier: Option<LicenseTier>, limit: i64, offset: i64) -> Result<Vec<Product>, sqlx::Error> {
        let products = if let Some(lt) = license_tier {
            sqlx::query_as!(
                Product,
                r#"
                SELECT id, sku, title, license_tier as "license_tier: LicenseTier", price, s3_prefix, created_at
                FROM products
                WHERE license_tier = $1
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3
                "#,
                lt as LicenseTier,
                limit,
                offset
            )
            .fetch_all(self.pool)
            .await?
        } else {
            sqlx::query_as!(
                Product,
                r#"
                SELECT id, sku, title, license_tier as "license_tier: LicenseTier", price, s3_prefix, created_at
                FROM products
                ORDER BY created_at DESC
                LIMIT $1 OFFSET $2
                "#,
                limit,
                offset
            )
            .fetch_all(self.pool)
            .await?
        };
        Ok(products)
    }
}

/// Repository for product_items table
pub struct ProductItemRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> ProductItemRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Add variant to product
    pub async fn add_variant(&self, product_id: Uuid, variant_id: Uuid, position: i32) -> Result<ProductItem, sqlx::Error> {
        let item = sqlx::query_as!(
            ProductItem,
            r#"
            INSERT INTO product_items (product_id, variant_id, position)
            VALUES ($1, $2, $3)
            RETURNING id, product_id, variant_id, position, created_at
            "#,
            product_id,
            variant_id,
            position
        )
        .fetch_one(self.pool)
        .await?;
        Ok(item)
    }

    /// Get items for a product
    pub async fn list_by_product(&self, product_id: Uuid) -> Result<Vec<ProductItem>, sqlx::Error> {
        let items = sqlx::query_as!(
            ProductItem,
            r#"
            SELECT id, product_id, variant_id, position, created_at
            FROM product_items
            WHERE product_id = $1
            ORDER BY position
            "#,
            product_id
        )
        .fetch_all(self.pool)
        .await?;
        Ok(items)
    }
}

/// Repository for platform_listings table
pub struct PlatformListingRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> PlatformListingRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Create a new platform listing (draft)
    pub async fn create_draft(
        &self,
        product_id: Uuid,
        platform: Platform,
        price: Decimal,
        fee_table_id: Option<Uuid>,
    ) -> Result<PlatformListing, sqlx::Error> {
        let listing = sqlx::query_as!(
            PlatformListing,
            r#"
            INSERT INTO platform_listings (product_id, platform, platform_listing_id, state, price, fee_table_id)
            VALUES ($1, $2, '', 'draft', $3, $4)
            RETURNING id, product_id, platform as "platform: Platform", platform_listing_id, state as "state: ListingState", price, platform_raw, fee_table_id, created_at, updated_at
            "#,
            product_id,
            platform as Platform,
            price,
            fee_table_id
        )
        .fetch_one(self.pool)
        .await?;
        Ok(listing)
    }

    /// Update listing with platform ID after publishing
    pub async fn update_platform_id(&self, id: Uuid, platform_listing_id: &str) -> Result<PlatformListing, sqlx::Error> {
        let listing = sqlx::query_as!(
            PlatformListing,
            r#"
            UPDATE platform_listings
            SET platform_listing_id = $2, updated_at = $3
            WHERE id = $1
            RETURNING id, product_id, platform as "platform: Platform", platform_listing_id, state as "state: ListingState", price, platform_raw, fee_table_id, created_at, updated_at
            "#,
            id,
            platform_listing_id,
            Utc::now()
        )
        .fetch_one(self.pool)
        .await?;
        Ok(listing)
    }

    /// Update listing state
    pub async fn update_state(&self, id: Uuid, state: ListingState) -> Result<PlatformListing, sqlx::Error> {
        let listing = sqlx::query_as!(
            PlatformListing,
            r#"
            UPDATE platform_listings
            SET state = $2, updated_at = $3
            WHERE id = $1
            RETURNING id, product_id, platform as "platform: Platform", platform_listing_id, state as "state: ListingState", price, platform_raw, fee_table_id, created_at, updated_at
            "#,
            id,
            state as ListingState,
            Utc::now()
        )
        .fetch_one(self.pool)
        .await?;
        Ok(listing)
    }

    /// Get listing by ID
    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<PlatformListing>, sqlx::Error> {
        let listing = sqlx::query_as!(
            PlatformListing,
            r#"
            SELECT id, product_id, platform as "platform: Platform", platform_listing_id, state as "state: ListingState", price, platform_raw, fee_table_id, created_at, updated_at
            FROM platform_listings
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(self.pool)
        .await?;
        Ok(listing)
    }

    /// List listings for a product
    pub async fn list_by_product(&self, product_id: Uuid) -> Result<Vec<PlatformListing>, sqlx::Error> {
        let listings = sqlx::query_as!(
            PlatformListing,
            r#"
            SELECT id, product_id, platform as "platform: Platform", platform_listing_id, state as "state: ListingState", price, platform_raw, fee_table_id, created_at, updated_at
            FROM platform_listings
            WHERE product_id = $1
            ORDER BY created_at
            "#,
            product_id
        )
        .fetch_all(self.pool)
        .await?;
        Ok(listings)
    }
}

/// Repository for sales_events table
pub struct SalesEventRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> SalesEventRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Record a sales event
    pub async fn create(
        &self,
        listing_id: Uuid,
        event_type: &str,
        gross: Decimal,
        fee: Decimal,
        net: Decimal,
    ) -> Result<SalesEvent, sqlx::Error> {
        let event = sqlx::query_as!(
            SalesEvent,
            r#"
            INSERT INTO sales_events (listing_id, event_type, gross, fee, net)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, listing_id, event_type, gross, fee, net, occurred_at
            "#,
            listing_id,
            event_type,
            gross,
            fee,
            net
        )
        .fetch_one(self.pool)
        .await?;
        Ok(event)
    }

    /// Get sales events for a listing
    pub async fn list_by_listing(&self, listing_id: Uuid) -> Result<Vec<SalesEvent>, sqlx::Error> {
        let events = sqlx::query_as!(
            SalesEvent,
            r#"
            SELECT id, listing_id, event_type, gross, fee, net, occurred_at
            FROM sales_events
            WHERE listing_id = $1
            ORDER BY occurred_at DESC
            "#,
            listing_id
        )
        .fetch_all(self.pool)
        .await?;
        Ok(events)
    }
}

/// Repository for fee_tables table
pub struct FeeTableRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> FeeTableRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Create or update fee table for platform
    pub async fn upsert(
        &self,
        platform: Platform,
        rules: serde_json::Value,
    ) -> Result<FeeTable, sqlx::Error> {
        let fee_table = sqlx::query_as!(
            FeeTable,
            r#"
            INSERT INTO fee_tables (platform, rules, effective_from)
            VALUES ($1, $2, $3)
            ON CONFLICT (platform) DO UPDATE SET
                rules = EXCLUDED.rules,
                effective_from = EXCLUDED.effective_from
            RETURNING id, platform as "platform: Platform", rules, effective_from
            "#,
            platform as Platform,
            rules,
            Utc::now()
        )
        .fetch_one(self.pool)
        .await?;
        Ok(fee_table)
    }

    /// Get fee table for platform
    pub async fn get_by_platform(&self, platform: Platform) -> Result<Option<FeeTable>, sqlx::Error> {
        let fee_table = sqlx::query_as!(
            FeeTable,
            r#"
            SELECT id, platform as "platform: Platform", rules, effective_from
            FROM fee_tables
            WHERE platform = $1
            ORDER BY effective_from DESC
            LIMIT 1
            "#,
            platform as Platform
        )
        .fetch_optional(self.pool)
        .await?;
        Ok(fee_table)
    }
}

/// Repository for listing_states table
pub struct ListingStateRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> ListingStateRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Record a state transition
    pub async fn record_transition(
        &self,
        listing_id: Uuid,
        from_state: Option<ListingState>,
        to_state: ListingState,
        transition_reason: Option<&str>,
        actor_id: Option<Uuid>,
    ) -> Result<ListingStateHistory, sqlx::Error> {
        let history = sqlx::query_as!(
            ListingStateHistory,
            r#"
            INSERT INTO listing_states (listing_id, from_state, to_state, transition_reason, actor_id)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, listing_id, from_state as "from_state: ListingState", to_state as "to_state: ListingState", transition_reason, actor_id, created_at
            "#,
            listing_id,
            from_state as Option<ListingState>,
            to_state as ListingState,
            transition_reason,
            actor_id
        )
        .fetch_one(self.pool)
        .await?;
        Ok(history)
    }

    /// Get state history for a listing
    pub async fn list_by_listing(&self, listing_id: Uuid) -> Result<Vec<ListingStateHistory>, sqlx::Error> {
        let history = sqlx::query_as!(
            ListingStateHistory,
            r#"
            SELECT id, listing_id, from_state as "from_state: ListingState", to_state as "to_state: ListingState", transition_reason, actor_id, created_at
            FROM listing_states
            WHERE listing_id = $1
            ORDER BY created_at
            "#,
            listing_id
        )
        .fetch_all(self.pool)
        .await?;
        Ok(history)
    }
}

/// Repository for processed_webhook_ids cache
pub struct ProcessedWebhookIdRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> ProcessedWebhookIdRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Check if webhook was already processed
    pub async fn is_processed(&self, platform: Platform, platform_event_id: &str) -> Result<bool, sqlx::Error> {
        let count: i64 = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM processed_webhook_ids
            WHERE platform = $1 AND platform_event_id = $2
            "#,
            platform as Platform,
            platform_event_id
        )
        .fetch_one(self.pool)
        .await?
        .unwrap_or(0);
        Ok(count > 0)
    }

    /// Mark webhook as processed
    pub async fn mark_processed(&self, platform: Platform, platform_event_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO processed_webhook_ids (platform, platform_event_id)
            VALUES ($1, $2)
            ON CONFLICT (platform, platform_event_id) DO NOTHING
            "#,
            platform as Platform,
            platform_event_id
        )
        .execute(self.pool)
        .await?;
        Ok(())
    }

    /// Cleanup expired entries (older than 7 days)
    pub async fn cleanup_expired(&self) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            DELETE FROM processed_webhook_ids
            WHERE received_at < now() - interval '7 days'
            "#
        )
        .execute(self.pool)
        .await?;
        Ok(result.rows_affected())
    }
}

/// Combined repository access for convenience
pub struct DbRepositories<'a> {
    pub niches: NicheRepository<'a>,
    pub production_runs: ProductionRunRepository<'a>,
    pub jobs: JobRepository<'a>,
    pub job_receipts: JobReceiptRepository<'a>,
    pub designs: DesignRepository<'a>,
    pub design_variants: DesignVariantRepository<'a>,
    pub curations: CurationRepository<'a>,
    pub products: ProductRepository<'a>,
    pub product_items: ProductItemRepository<'a>,
    pub platform_listings: PlatformListingRepository<'a>,
    pub sales_events: SalesEventRepository<'a>,
    pub fee_tables: FeeTableRepository<'a>,
    pub listing_states: ListingStateRepository<'a>,
    pub processed_webhook_ids: ProcessedWebhookIdRepository<'a>,
}

impl<'a> DbRepositories<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            niches: NicheRepository::new(pool),
            production_runs: ProductionRunRepository::new(pool),
            jobs: JobRepository::new(pool),
            job_receipts: JobReceiptRepository::new(pool),
            designs: DesignRepository::new(pool),
            design_variants: DesignVariantRepository::new(pool),
            curations: CurationRepository::new(pool),
            products: ProductRepository::new(pool),
            product_items: ProductItemRepository::new(pool),
            platform_listings: PlatformListingRepository::new(pool),
            sales_events: SalesEventRepository::new(pool),
            fee_tables: FeeTableRepository::new(pool),
            listing_states: ListingStateRepository::new(pool),
            processed_webhook_ids: ProcessedWebhookIdRepository::new(pool),
        }
    }
}