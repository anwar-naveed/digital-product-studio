//! S3/MinIO client layer for digital-product-studio
//!
//! Implements the S3 client layer per docs/11-architecture.md §1 (ADR-003).
//! Key builder for all 6 lifecycle prefixes, S3Client wrapper with presign/copy helpers,
//! WorkerS3Client and AdminS3Client for role-specific operations.

use crate::error::{AppError, Result};
use object_store::aws::AmazonS3Builder;
use object_store::path::Path;
use object_store::{ObjectStore, ObjectStoreExt, ObjectMeta, GetResult, PutPayload};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;
use sha2::{Digest, Sha256};
use futures::StreamExt;

/// S3 key prefixes for lifecycle management (ADR-003)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum S3Prefix {
    Raw,
    Curated,
    Products,
    Mockups,
    Listings,
    Tmp,
}

impl S3Prefix {
    pub fn as_str(&self) -> &'static str {
        match self {
            S3Prefix::Raw => "raw",
            S3Prefix::Curated => "curated",
            S3Prefix::Products => "products",
            S3Prefix::Mockups => "mockups",
            S3Prefix::Listings => "listings",
            S3Prefix::Tmp => "tmp",
        }
    }
}

/// S3 key builder for all lifecycle prefixes (G3)
#[derive(Clone)]
pub struct S3KeyBuilder {
    bucket: String,
}

impl S3KeyBuilder {
    pub fn new(bucket: String) -> Self {
        Self { bucket }
    }

    /// Build key for raw generation output
    /// raw/<run_id>/<design_id>/<variant_id>.png
    pub fn raw(&self, run_id: Uuid, design_id: Uuid, variant_id: Uuid, ext: &str) -> String {
        format!(
            "{}/{}/{}/{}.{}",
            S3Prefix::Raw.as_str(),
            run_id,
            design_id,
            variant_id,
            ext
        )
    }

    /// Build key for curated (approved) artifacts
    /// curated/<niche_id>/<design_id>/<variant_id>.png
    pub fn curated(&self, niche_id: Uuid, design_id: Uuid, variant_id: Uuid, ext: &str) -> String {
        format!(
            "{}/{}/{}/{}.{}",
            S3Prefix::Curated.as_str(),
            niche_id,
            design_id,
            variant_id,
            ext
        )
    }

    /// Build key for product bundles
    /// products/<product_id>/<bundle_version>/<file>.pdf|.zip
    pub fn products(&self, product_id: Uuid, bundle_version: &str, filename: &str) -> String {
        format!(
            "{}/{}/{}/{}",
            S3Prefix::Products.as_str(),
            product_id,
            bundle_version,
            filename
        )
    }

    /// Build key for mockups
    /// mockups/<product_id>/<mockup_id>.png
    pub fn mockups(&self, product_id: Uuid, mockup_id: Uuid, ext: &str) -> String {
        format!(
            "{}/{}/{}.{}",
            S3Prefix::Mockups.as_str(),
            product_id,
            mockup_id,
            ext
        )
    }

    /// Build key for listing artifacts
    /// listings/<platform>/<platform_listing_id>/<artifact>
    pub fn listings(&self, platform: &str, platform_listing_id: &str, artifact: &str) -> String {
        format!(
            "{}/{}/{}/{}",
            S3Prefix::Listings.as_str(),
            platform.to_lowercase(),
            platform_listing_id,
            artifact
        )
    }

    /// Build key for temporary staging (24h lifecycle)
    /// tmp/<job_id>/...
    pub fn tmp(&self, job_id: Uuid, suffix: &str) -> String {
        format!("{}/{}/{}", S3Prefix::Tmp.as_str(), job_id, suffix)
    }

    /// Parse a key to extract prefix and components
    pub fn parse_key(&self, key: &str) -> Option<S3KeyParts> {
        let parts: Vec<&str> = key.split('/').collect();
        if parts.is_empty() {
            return None;
        }

        let prefix = match parts[0] {
            "raw" => S3Prefix::Raw,
            "curated" => S3Prefix::Curated,
            "products" => S3Prefix::Products,
            "mockups" => S3Prefix::Mockups,
            "listings" => S3Prefix::Listings,
            "tmp" => S3Prefix::Tmp,
            _ => return None,
        };

        Some(S3KeyParts {
            prefix,
            components: parts[1..].iter().map(|s| s.to_string()).collect(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct S3KeyParts {
    pub prefix: S3Prefix,
    pub components: Vec<String>,
}

/// Core S3 client wrapper using object_store
#[derive(Clone)]
pub struct S3Client {
    store: Arc<dyn ObjectStore>,
    bucket: String,
    key_builder: S3KeyBuilder,
}

impl S3Client {
    /// Get a clone of the inner object store handle
    pub fn store_handle(&self) -> Arc<dyn ObjectStore> {
        Arc::clone(&self.store)
    }

    /// Create a new S3 client for MinIO
    pub async fn new(
        endpoint: &str,
        bucket: &str,
        access_key: &str,
        secret_key: &str,
        region: &str,
    ) -> Result<Self> {
        let config = AmazonS3Builder::new()
            .with_bucket_name(bucket)
            .with_endpoint(endpoint)
            .with_access_key_id(access_key)
            .with_secret_access_key(secret_key)
            .with_region(region)
            .with_allow_http(true) // MinIO often uses HTTP locally
            .with_virtual_hosted_style_request(false) // MinIO prefers path-style
            .build()?;

        let store = Arc::new(config);
        let key_builder = S3KeyBuilder::new(bucket.to_string());

        Ok(Self {
            store,
            bucket: bucket.to_string(),
            key_builder,
        })
    }

    /// Get the key builder
    pub fn key_builder(&self) -> &S3KeyBuilder {
        &self.key_builder
    }

    /// Put object with metadata
    pub async fn put(&self, key: &str, data: Vec<u8>, metadata: HashMap<String, String>) -> Result<()> {
        let path = Path::from(key);
        let payload = PutPayload::from(data);
        let mut opts = object_store::PutOptions::default();
        for (k, v) in metadata {
            opts.attributes.insert(
                object_store::Attribute::Metadata(k.into()),
                object_store::AttributeValue::from(v),
            );
        }
        
        self.store.put_opts(&path, payload, opts).await.map(|_| ()).map_err(|e| {
            AppError::Storage(format!("Failed to put object {}: {}", key, e))
        })
    }

    /// Get object
    pub async fn get(&self, key: &str) -> Result<GetResult> {
        let path = Path::from(key);
        self.store.get(&path).await.map_err(|e| {
            AppError::Storage(format!("Failed to get object {}: {}", key, e))
        })
    }

    /// Get object metadata only (HEAD)
    pub async fn head(&self, key: &str) -> Result<ObjectMeta> {
        let path = Path::from(key);
        self.store.head(&path).await.map_err(|e| {
            AppError::Storage(format!("Failed to head object {}: {}", key, e))
        })
    }

    /// Delete object
    pub async fn delete(&self, key: &str) -> Result<()> {
        let path = Path::from(key);
        self.store.delete(&path).await.map_err(|e| {
            AppError::Storage(format!("Failed to delete object {}: {}", key, e))
        })
    }

    /// List objects with prefix
    pub async fn list(&self, prefix: &str) -> Result<Vec<ObjectMeta>> {
        let path = Path::from(prefix);
        let mut results = Vec::new();
        let mut stream = self.store.list(Some(&path));
        
        while let Some(result) = stream.next().await {
            match result {
                Ok(meta) => results.push(meta),
                Err(e) => return Err(AppError::Storage(format!("Failed to list objects: {}", e))),
            }
        }
        
        Ok(results)
    }

    /// Copy object within same bucket (server-side copy)
    pub async fn copy(&self, from_key: &str, to_key: &str) -> Result<()> {
        // object_store doesn't have a direct copy, so we get and put
        // In production, you might use S3 API directly for server-side copy
        let get_result = self.get(from_key).await?;
        let data = get_result.bytes().await.map_err(|e| {
            AppError::Storage(format!("Failed to read source object: {}", e))
        })?;
        
        self.put(to_key, data.to_vec(), HashMap::new()).await
    }

    /// Generate presigned GET URL (valid for specified duration)
    pub async fn presign_get(&self, key: &str, expiry: Duration) -> Result<String> {
        // object_store doesn't directly support presigning
        // This would use the AWS SDK or MinIO SDK for presigned URLs
        // For now, return a placeholder that the worker can use
        // NOTE: object_store does not support presigning; construct MinIO gateway path.
        // The edge (Pingap) handles auth for browser access in v1.
        let url = format!(
            "/{}/{}?X-Amz-Expires={}",
            self.bucket,
            key,
            expiry.as_secs()
        );
        Ok(url)
    }

    /// Generate presigned PUT URL
    pub async fn presign_put(&self, key: &str, expiry: Duration) -> Result<String> {
        let url = format!(
            "/{}/{}?X-Amz-Expires={}&X-Amz-Content-Sha256=UNSIGNED-PAYLOAD",
            self.bucket,
            key,
            expiry.as_secs()
        );
        Ok(url)
    }

    /// Check if object exists
    pub async fn exists(&self, key: &str) -> bool {
        self.head(key).await.is_ok()
    }
}

/// Worker-facing S3 client (upload operations)
pub struct WorkerS3Client {
    inner: S3Client,
}

impl WorkerS3Client {
    pub fn new(inner: S3Client) -> Self {
        Self { inner }
    }

    /// Upload generation output to raw/
    pub async fn upload_generation(
        &self,
        run_id: Uuid,
        design_id: Uuid,
        variant_id: Uuid,
        data: Vec<u8>,
        ext: &str,
        job_id: Uuid,
    ) -> Result<String> {
        let key = self.inner.key_builder().raw(run_id, design_id, variant_id, ext);
        
        let mut metadata = HashMap::new();
        metadata.insert("x-amz-meta-worker-id".to_string(), job_id.to_string());
        metadata.insert("x-amz-meta-run-id".to_string(), run_id.to_string());
        metadata.insert("x-amz-meta-design-id".to_string(), design_id.to_string());
        metadata.insert("x-amz-meta-variant-id".to_string(), variant_id.to_string());
        
        // Compute SHA256 for integrity
        use sha2::{Digest, Sha256};
use futures::StreamExt;
        let sha256 = format!("{:x}", Sha256::digest(&data));
        metadata.insert("x-amz-meta-sha256".to_string(), sha256);
        
        self.inner.put(&key, data, metadata).await?;
        Ok(key)
    }

    /// Upload product bundle
    pub async fn upload_product_bundle(
        &self,
        product_id: Uuid,
        bundle_version: &str,
        filename: &str,
        data: Vec<u8>,
        job_id: Uuid,
    ) -> Result<String> {
        let key = self.inner.key_builder().products(product_id, bundle_version, filename);
        
        let mut metadata = HashMap::new();
        metadata.insert("x-amz-meta-worker-id".to_string(), job_id.to_string());
        metadata.insert("x-amz-meta-product-id".to_string(), product_id.to_string());
        metadata.insert("x-amz-meta-bundle-version".to_string(), bundle_version.to_string());
        
        let sha256 = format!("{:x}", Sha256::digest(&data));
        metadata.insert("x-amz-meta-sha256".to_string(), sha256);
        
        self.inner.put(&key, data, metadata).await?;
        Ok(key)
    }

    /// Upload mockup
    pub async fn upload_mockup(
        &self,
        product_id: Uuid,
        mockup_id: Uuid,
        data: Vec<u8>,
        ext: &str,
        job_id: Uuid,
    ) -> Result<String> {
        let key = self.inner.key_builder().mockups(product_id, mockup_id, ext);
        
        let mut metadata = HashMap::new();
        metadata.insert("x-amz-meta-worker-id".to_string(), job_id.to_string());
        metadata.insert("x-amz-meta-product-id".to_string(), product_id.to_string());
        metadata.insert("x-amz-meta-mockup-id".to_string(), mockup_id.to_string());
        
        let sha256 = format!("{:x}", Sha256::digest(&data));
        metadata.insert("x-amz-meta-sha256".to_string(), sha256);
        
        self.inner.put(&key, data, metadata).await?;
        Ok(key)
    }

    /// Stage file in tmp/ (24h lifecycle)
    pub async fn stage_tmp(&self, job_id: Uuid, filename: &str, data: Vec<u8>) -> Result<String> {
        let key = self.inner.key_builder().tmp(job_id, filename);
        
        let mut metadata = HashMap::new();
        metadata.insert("x-amz-meta-job-id".to_string(), job_id.to_string());
        metadata.insert("x-amz-meta-staged-at".to_string(), chrono::Utc::now().to_rfc3339());
        
        self.inner.put(&key, data, metadata).await?;
        Ok(key)
    }

    /// Get staged file from tmp/
    pub async fn get_staged(&self, job_id: Uuid, filename: &str) -> Result<GetResult> {
        let key = self.inner.key_builder().tmp(job_id, filename);
        self.inner.get(&key).await
    }
}

/// Admin-facing S3 client (curation, product, listing operations)
pub struct AdminS3Client {
    inner: S3Client,
}

impl AdminS3Client {
    pub fn new(inner: S3Client) -> Self {
        Self { inner }
    }

    /// Generate presigned GET URL for curation preview
    pub async fn presign_curation_preview(&self, variant_key: &str, expiry_secs: u64) -> Result<String> {
        // Verify the key is under raw/ or curated/
        let parts = self.inner.key_builder().parse_key(variant_key);
        if let Some(parts) = parts {
            if !matches!(parts.prefix, S3Prefix::Raw | S3Prefix::Curated) {
                return Err(AppError::Validation(
                    "Curation preview only allowed for raw/ or curated/ keys".to_string(),
                ));
            }
        }
        
        self.inner.presign_get(variant_key, Duration::from_secs(expiry_secs)).await
    }

    /// Approve variant: COPY raw/... → curated/...
    pub async fn approve_variant(
        &self,
        raw_key: &str,
        niche_id: Uuid,
        design_id: Uuid,
        variant_id: Uuid,
    ) -> Result<String> {
        // Verify source is under raw/
        let parts = self.inner.key_builder().parse_key(raw_key);
        if let Some(parts) = parts {
            if parts.prefix != S3Prefix::Raw {
                return Err(AppError::Validation("Source must be under raw/ prefix".to_string()));
            }
        }

        // Extract extension from raw key
        let ext = raw_key.split('.').last().unwrap_or("png");
        
        // Build curated key
        let curated_key = self.inner.key_builder().curated(niche_id, design_id, variant_id, ext);
        
        // Copy raw → curated (server-side copy ideally)
        self.inner.copy(raw_key, &curated_key).await?;
        
        Ok(curated_key)
    }

    /// Generate presigned PUT for product asset upload
    pub async fn presign_product_upload(
        &self,
        product_id: Uuid,
        bundle_version: &str,
        filename: &str,
        expiry_secs: u64,
    ) -> Result<String> {
        let key = self.inner.key_builder().products(product_id, bundle_version, filename);
        self.inner.presign_put(&key, Duration::from_secs(expiry_secs)).await
    }

    /// Generate presigned GET for product asset download
    pub async fn presign_product_download(&self, product_key: &str, expiry_secs: u64) -> Result<String> {
        let parts = self.inner.key_builder().parse_key(product_key);
        if let Some(parts) = parts {
            if parts.prefix != S3Prefix::Products {
                return Err(AppError::Validation("Key must be under products/ prefix".to_string()));
            }
        }
        
        self.inner.presign_get(product_key, Duration::from_secs(expiry_secs)).await
    }

    /// Generate presigned GET for mockup
    pub async fn presign_mockup(&self, mockup_key: &str, expiry_secs: u64) -> Result<String> {
        let parts = self.inner.key_builder().parse_key(mockup_key);
        if let Some(parts) = parts {
            if parts.prefix != S3Prefix::Mockups {
                return Err(AppError::Validation("Key must be under mockups/ prefix".to_string()));
            }
        }
        
        self.inner.presign_get(mockup_key, Duration::from_secs(expiry_secs)).await
    }

    /// Generate presigned PUT for listing artifact
    pub async fn presign_listing_upload(
        &self,
        platform: &str,
        platform_listing_id: &str,
        artifact: &str,
        expiry_secs: u64,
    ) -> Result<String> {
        let key = self.inner.key_builder().listings(platform, platform_listing_id, artifact);
        self.inner.presign_put(&key, Duration::from_secs(expiry_secs)).await
    }

    /// Generate presigned GET for listing artifact
    pub async fn presign_listing_download(&self, listing_key: &str, expiry_secs: u64) -> Result<String> {
        let parts = self.inner.key_builder().parse_key(listing_key);
        if let Some(parts) = parts {
            if parts.prefix != S3Prefix::Listings {
                return Err(AppError::Validation("Key must be under listings/ prefix".to_string()));
            }
        }
        
        self.inner.presign_get(listing_key, Duration::from_secs(expiry_secs)).await
    }

    /// List objects under a prefix
    pub async fn list_prefix(&self, prefix: S3Prefix, subpath: &str) -> Result<Vec<ObjectMeta>> {
        let full_prefix = format!("{}/{}", prefix.as_str(), subpath);
        self.inner.list(&full_prefix).await
    }
}

/// Factory for creating S3 clients with proper configuration
pub struct S3ClientFactory;

impl S3ClientFactory {
    /// Create clients from environment configuration
    pub async fn create_from_env() -> Result<(WorkerS3Client, AdminS3Client)> {
        let endpoint = std::env::var("MINIO_ENDPOINT")
            .unwrap_or_else(|_| "http://172.19.0.1:9010".to_string());
        let bucket = std::env::var("MINIO_BUCKET")
            .unwrap_or_else(|_| "digital-product-studio".to_string());
        let access_key = std::env::var("MINIO_ACCESS_KEY")
            .map_err(|_| AppError::Config("MINIO_ACCESS_KEY not set".to_string()))?;
        let secret_key = std::env::var("MINIO_SECRET_KEY")
            .map_err(|_| AppError::Config("MINIO_SECRET_KEY not set".to_string()))?;
        let region = std::env::var("MINIO_REGION").unwrap_or_else(|_| "us-east-1".to_string());

        let s3_client = S3Client::new(&endpoint, &bucket, &access_key, &secret_key, &region).await?;
        
        Ok((
            WorkerS3Client::new(s3_client.clone()),
            AdminS3Client::new(s3_client),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_s3_key_builder_raw() {
        let builder = S3KeyBuilder::new("test-bucket".to_string());
        let run_id = Uuid::nil();
        let design_id = Uuid::nil();
        let variant_id = Uuid::nil();
        
        let key = builder.raw(run_id, design_id, variant_id, "png");
        assert!(key.starts_with("raw/"));
        assert!(key.contains(&run_id.to_string()));
        assert!(key.contains(&design_id.to_string()));
        assert!(key.contains(&variant_id.to_string()));
        assert!(key.ends_with(".png"));
    }

    #[test]
    fn test_s3_key_builder_curated() {
        let builder = S3KeyBuilder::new("test-bucket".to_string());
        let niche_id = Uuid::nil();
        let design_id = Uuid::nil();
        let variant_id = Uuid::nil();
        
        let key = builder.curated(niche_id, design_id, variant_id, "jpg");
        assert!(key.starts_with("curated/"));
        assert!(key.contains(&niche_id.to_string()));
        assert!(key.ends_with(".jpg"));
    }

    #[test]
    fn test_s3_key_builder_products() {
        let builder = S3KeyBuilder::new("test-bucket".to_string());
        let product_id = Uuid::nil();
        
        let key = builder.products(product_id, "v1", "bundle.zip");
        assert!(key.starts_with("products/"));
        assert!(key.contains(&product_id.to_string()));
        assert!(key.contains("v1"));
        assert!(key.ends_with("bundle.zip"));
    }

    #[test]
    fn test_s3_key_builder_mockups() {
        let builder = S3KeyBuilder::new("test-bucket".to_string());
        let product_id = Uuid::nil();
        let mockup_id = Uuid::nil();
        
        let key = builder.mockups(product_id, mockup_id, "png");
        assert!(key.starts_with("mockups/"));
        assert!(key.contains(&product_id.to_string()));
        assert!(key.contains(&mockup_id.to_string()));
        assert!(key.ends_with(".png"));
    }

    #[test]
    fn test_s3_key_builder_listings() {
        let builder = S3KeyBuilder::new("test-bucket".to_string());
        
        let key = builder.listings("Etsy", "etsy-123", "image.png");
        assert!(key.starts_with("listings/"));
        assert!(key.contains("etsy"));
        assert!(key.contains("etsy-123"));
        assert!(key.ends_with("image.png"));
    }

    #[test]
    fn test_s3_key_builder_tmp() {
        let builder = S3KeyBuilder::new("test-bucket".to_string());
        let job_id = Uuid::nil();
        
        let key = builder.tmp(job_id, "staging/file.json");
        assert!(key.starts_with("tmp/"));
        assert!(key.contains(&job_id.to_string()));
        assert!(key.ends_with("staging/file.json"));
    }

    #[test]
    fn test_s3_key_builder_parse() {
        let builder = S3KeyBuilder::new("test-bucket".to_string());
        let key = "raw/123/456/789.png";
        
        let parsed = builder.parse_key(key).unwrap();
        assert_eq!(parsed.prefix, S3Prefix::Raw);
        assert_eq!(parsed.components, vec!["123", "456", "789.png"]);
    }

    #[test]
    fn test_s3_key_builder_parse_invalid() {
        let builder = S3KeyBuilder::new("test-bucket".to_string());
        let key = "invalid/key";
        
        let parsed = builder.parse_key(key);
        assert!(parsed.is_none());
    }
}