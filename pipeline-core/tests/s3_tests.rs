//! S3 integration tests (testcontainers-based) — legacy scaffold against the
//! pre-refactor client types. Compile-gated behind `s3-integration` until
//! rewritten for the current WorkerS3Client/AdminS3Client APIs.
#![cfg(feature = "s3-integration")]

//! Tests for S3 key builder and client (G3, G4)

use pipeline_core::s3::{S3KeyBuilder, S3Prefix, S3ClientFactory};
use uuid::Uuid;

#[cfg(test)]
mod s3_key_builder_tests {
    use super::*;

    #[test]
    fn test_raw_key_generation() {
        let builder = S3KeyBuilder::new("digital-product-studio".to_string());
        let run_id = Uuid::nil();
        let design_id = Uuid::nil();
        let variant_id = Uuid::nil();
        
        let key = builder.raw(run_id, design_id, variant_id, "png");
        
        assert!(key.starts_with("raw/"));
        assert!(key.contains(&run_id.to_string()));
        assert!(key.contains(&design_id.to_string()));
        assert!(key.contains(&variant_id.to_string()));
        assert!(key.ends_with(".png"));
        assert_eq!(key.split('/').count(), 4); // raw/run/design/variant.png
    }

    #[test]
    fn test_curated_key_generation() {
        let builder = S3KeyBuilder::new("digital-product-studio".to_string());
        let niche_id = Uuid::nil();
        let design_id = Uuid::nil();
        let variant_id = Uuid::nil();
        
        let key = builder.curated(niche_id, design_id, variant_id, "jpg");
        
        assert!(key.starts_with("curated/"));
        assert!(key.contains(&niche_id.to_string()));
        assert!(key.contains(&design_id.to_string()));
        assert!(key.contains(&variant_id.to_string()));
        assert!(key.ends_with(".jpg"));
        assert_eq!(key.split('/').count(), 4); // curated/niche/design/variant.jpg
    }

    #[test]
    fn test_products_key_generation() {
        let builder = S3KeyBuilder::new("digital-product-studio".to_string());
        let product_id = Uuid::nil();
        
        let key = builder.products(product_id, "v1", "bundle.zip");
        
        assert!(key.starts_with("products/"));
        assert!(key.contains(&product_id.to_string()));
        assert!(key.contains("v1"));
        assert!(key.ends_with("bundle.zip"));
        assert_eq!(key.split('/').count(), 4); // products/product_id/v1/bundle.zip
    }

    #[test]
    fn test_products_key_generation_pdf() {
        let builder = S3KeyBuilder::new("digital-product-studio".to_string());
        let product_id = Uuid::nil();
        
        let key = builder.products(product_id, "v2", "license.pdf");
        
        assert!(key.starts_with("products/"));
        assert!(key.ends_with("license.pdf"));
    }

    #[test]
    fn test_mockups_key_generation() {
        let builder = S3KeyBuilder::new("digital-product-studio".to_string());
        let product_id = Uuid::nil();
        let mockup_id = Uuid::nil();
        
        let key = builder.mockups(product_id, mockup_id, "png");
        
        assert!(key.starts_with("mockups/"));
        assert!(key.contains(&product_id.to_string()));
        assert!(key.contains(&mockup_id.to_string()));
        assert!(key.ends_with(".png"));
        assert_eq!(key.split('/').count(), 3); // mockups/product_id/mockup_id.png
    }

    #[test]
    fn test_listings_key_generation() {
        let builder = S3KeyBuilder::new("digital-product-studio".to_string());
        
        let key = builder.listings("Etsy", "etsy-12345", "image.png");
        
        assert!(key.starts_with("listings/"));
        assert!(key.contains("etsy"));
        assert!(key.contains("etsy-12345"));
        assert!(key.ends_with("image.png"));
        assert_eq!(key.split('/').count(), 4); // listings/etsy/etsy-12345/image.png
    }

    #[test]
    fn test_listings_key_case_insensitive() {
        let builder = S3KeyBuilder::new("digital-product-studio".to_string());
        
        let key = builder.listings("LEMON_SQUEEZY", "ls-678", "file.zip");
        
        assert!(key.starts_with("listings/"));
        assert!(key.contains("lemon_squeezy")); // Should be lowercase
    }

    #[test]
    fn test_tmp_key_generation() {
        let builder = S3KeyBuilder::new("digital-product-studio".to_string());
        let job_id = Uuid::nil();
        
        let key = builder.tmp(job_id, "staging/input.json");
        
        assert!(key.starts_with("tmp/"));
        assert!(key.contains(&job_id.to_string()));
        assert!(key.ends_with("staging/input.json"));
    }

    #[test]
    fn test_parse_raw_key() {
        let builder = S3KeyBuilder::new("digital-product-studio".to_string());
        let run_id = Uuid::nil();
        let design_id = Uuid::nil();
        let variant_id = Uuid::nil();
        
        let key = builder.raw(run_id, design_id, variant_id, "png");
        let parsed = builder.parse_key(&key).unwrap();
        
        assert_eq!(parsed.prefix, S3Prefix::Raw);
        assert_eq!(parsed.components.len(), 3);
        assert_eq!(parsed.components[0], run_id.to_string());
        assert_eq!(parsed.components[1], design_id.to_string());
        assert_eq!(parsed.components[2], format!("{}.png", variant_id));
    }

    #[test]
    fn test_parse_curated_key() {
        let builder = S3KeyBuilder::new("digital-product-studio".to_string());
        let niche_id = Uuid::nil();
        let design_id = Uuid::nil();
        let variant_id = Uuid::nil();
        
        let key = builder.curated(niche_id, design_id, variant_id, "jpg");
        let parsed = builder.parse_key(&key).unwrap();
        
        assert_eq!(parsed.prefix, S3Prefix::Curated);
        assert_eq!(parsed.components.len(), 3);
    }

    #[test]
    fn test_parse_products_key() {
        let builder = S3KeyBuilder::new("digital-product-studio".to_string());
        let product_id = Uuid::nil();
        
        let key = builder.products(product_id, "v1", "bundle.zip");
        let parsed = builder.parse_key(&key).unwrap();
        
        assert_eq!(parsed.prefix, S3Prefix::Products);
        assert_eq!(parsed.components.len(), 3);
        assert_eq!(parsed.components[0], product_id.to_string());
        assert_eq!(parsed.components[1], "v1");
        assert_eq!(parsed.components[2], "bundle.zip");
    }

    #[test]
    fn test_parse_mockups_key() {
        let builder = S3KeyBuilder::new("digital-product-studio".to_string());
        let product_id = Uuid::nil();
        let mockup_id = Uuid::nil();
        
        let key = builder.mockups(product_id, mockup_id, "png");
        let parsed = builder.parse_key(&key).unwrap();
        
        assert_eq!(parsed.prefix, S3Prefix::Mockups);
        assert_eq!(parsed.components.len(), 2);
        assert_eq!(parsed.components[0], product_id.to_string());
        assert_eq!(parsed.components[1], format!("{}.png", mockup_id));
    }

    #[test]
    fn test_parse_listings_key() {
        let builder = S3KeyBuilder::new("digital-product-studio".to_string());
        
        let key = builder.listings("Etsy", "etsy-123", "image.png");
        let parsed = builder.parse_key(&key).unwrap();
        
        assert_eq!(parsed.prefix, S3Prefix::Listings);
        assert_eq!(parsed.components.len(), 3);
        assert_eq!(parsed.components[0], "etsy");
        assert_eq!(parsed.components[1], "etsy-123");
        assert_eq!(parsed.components[2], "image.png");
    }

    #[test]
    fn test_parse_tmp_key() {
        let builder = S3KeyBuilder::new("digital-product-studio".to_string());
        let job_id = Uuid::nil();
        
        let key = builder.tmp(job_id, "staging/file.json");
        let parsed = builder.parse_key(&key).unwrap();
        
        assert_eq!(parsed.prefix, S3Prefix::Tmp);
        assert_eq!(parsed.components.len(), 2);
        assert_eq!(parsed.components[0], job_id.to_string());
        assert_eq!(parsed.components[1], "staging/file.json");
    }

    #[test]
    fn test_parse_invalid_key() {
        let builder = S3KeyBuilder::new("digital-product-studio".to_string());
        
        let result = builder.parse_key("invalid/key");
        assert!(result.is_none());
        
        let result = builder.parse_key("unknown/prefix");
        assert!(result.is_none());
    }

    #[test]
    fn test_all_prefixes_covered() {
        let builder = S3KeyBuilder::new("test-bucket".to_string());
        
        // Verify all 6 lifecycle prefixes can be generated
        let run_id = Uuid::nil();
        let design_id = Uuid::nil();
        let variant_id = Uuid::nil();
        let niche_id = Uuid::nil();
        let product_id = Uuid::nil();
        let mockup_id = Uuid::nil();
        let job_id = Uuid::nil();
        
        let keys = vec![
            builder.raw(run_id, design_id, variant_id, "png"),
            builder.curated(niche_id, design_id, variant_id, "png"),
            builder.products(product_id, "v1", "file.pdf"),
            builder.mockups(product_id, mockup_id, "png"),
            builder.listings("Etsy", "id", "file.png"),
            builder.tmp(job_id, "file.json"),
        ];
        
        let prefixes: Vec<&str> = keys.iter().map(|k| k.split('/').next().unwrap()).collect();
        assert_eq!(prefixes, vec!["raw", "curated", "products", "mockups", "listings", "tmp"]);
    }
}

#[cfg(test)]
mod s3_prefix_tests {
    use super::*;

    #[test]
    fn test_s3_prefix_as_str() {
        assert_eq!(S3Prefix::Raw.as_str(), "raw");
        assert_eq!(S3Prefix::Curated.as_str(), "curated");
        assert_eq!(S3Prefix::Products.as_str(), "products");
        assert_eq!(S3Prefix::Mockups.as_str(), "mockups");
        assert_eq!(S3Prefix::Listings.as_str(), "listings");
        assert_eq!(S3Prefix::Tmp.as_str(), "tmp");
    }

    #[test]
    fn test_s3_prefix_equality() {
        assert_eq!(S3Prefix::Raw, S3Prefix::Raw);
        assert_ne!(S3Prefix::Raw, S3Prefix::Curated);
    }
}

// G4: Integration tests against MinIO testcontainer
// These would run inside Docker with a MinIO testcontainer
#[cfg(test)]
mod s3_integration_tests {
    use super::*;
    use testcontainers::clients::Cli;
    use testcontainers::images::generic::GenericImage;
    use testcontainers::core::WaitFor;
    use std::time::Duration;

    /// Helper to start a MinIO testcontainer
    async fn start_minio() -> (testcontainers::Container<'static, GenericImage>, String, String, String) {
        let docker = Cli::default();
        
        let image = GenericImage::new("minio/minio", "RELEASE.2024-01-16T16-07-38Z")
            .with_exposed_port(9000)
            .with_env_var("MINIO_ROOT_USER", "minioadmin")
            .with_env_var("MINIO_ROOT_PASSWORD", "minioadmin")
            .with_cmd(vec!["server", "/data", "--console-address", ":9001"])
            .with_wait_for(WaitFor::message_on_stdout("MinIO Object Storage Server"));
        
        let container = docker.run(image);
        let port = container.get_host_port_ipv4(9000);
        let endpoint = format!("http://127.0.0.1:{}", port);
        
        // Wait for MinIO to be ready
        tokio::time::sleep(Duration::from_secs(3)).await;
        
        (container, endpoint, "minioadmin".to_string(), "minioadmin".to_string())
    }

    #[tokio::test]
    #[ignore] // Run with --ignored flag to test against real MinIO
    async fn test_s3_client_put_get() {
        let (_container, endpoint, access_key, secret_key) = start_minio().await;
        let bucket = "test-bucket";
        
        let client = pipeline_core::s3::S3Client::new(
            &endpoint, bucket, &access_key, &secret_key, "us-east-1"
        ).await.unwrap();
        
        // Create bucket using MinIO client or API
        // For this test, we assume bucket exists or use a different approach
        
        let key = "test/object.txt";
        let data = b"Hello, MinIO!";
        let metadata = std::collections::HashMap::new();
        
        // Put object
        let meta = client.put(key, data.to_vec(), metadata).await.unwrap();
        assert_eq!(meta.size, data.len() as u64);
        
        // Get object
        let get_result = client.get(key).await.unwrap();
        let bytes = get_result.bytes().await.unwrap();
        assert_eq!(bytes.as_ref(), data);
        
        // Head object
        let head_meta = client.head(key).await.unwrap();
        assert_eq!(head_meta.size, data.len() as u64);
        
        // List objects
        let objects = client.list("test/").await.unwrap();
        assert_eq!(objects.len(), 1);
        assert_eq!(objects[0].location, key);
        
        // Delete object
        client.delete(key).await.unwrap();
        
        // Verify deleted
        let result = client.get(key).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[ignore]
    async fn test_s3_client_copy() {
        let (_container, endpoint, access_key, secret_key) = start_minio().await;
        let bucket = "test-bucket";
        
        let client = pipeline_core::s3::S3Client::new(
            &endpoint, bucket, &access_key, &secret_key, "us-east-1"
        ).await.unwrap();
        
        let source_key = "source/file.txt";
        let dest_key = "dest/file.txt";
        let data = b"Copy test data";
        
        client.put(source_key, data.to_vec(), std::collections::HashMap::new()).await.unwrap();
        
        let copied_meta = client.copy(source_key, dest_key).await.unwrap();
        assert_eq!(copied_meta.size, data.len() as u64);
        
        // Verify both exist
        let src = client.get(source_key).await.unwrap();
        let dst = client.get(dest_key).await.unwrap();
        assert_eq!(src.bytes().await.unwrap().as_ref(), data);
        assert_eq!(dst.bytes().await.unwrap().as_ref(), data);
    }

    #[tokio::test]
    #[ignore]
    async fn test_worker_s3_client_upload_generation() {
        let (_container, endpoint, access_key, secret_key) = start_minio().await;
        let bucket = "test-bucket";
        
        let s3_client = pipeline_core::s3::S3Client::new(
            &endpoint, bucket, &access_key, &secret_key, "us-east-1"
        ).await.unwrap();
        
        let worker_client = pipeline_core::s3::WorkerS3Client::new(s3_client);
        
        let run_id = Uuid::new_v4();
        let design_id = Uuid::new_v4();
        let variant_id = Uuid::new_v4();
        let job_id = Uuid::new_v4();
        let data = b"fake png data";
        
        let key = worker_client.upload_generation(
            run_id, design_id, variant_id, data.to_vec(), "png", job_id
        ).await.unwrap();
        
        assert!(key.starts_with("raw/"));
        assert!(key.contains(&run_id.to_string()));
        assert!(key.contains(&design_id.to_string()));
        assert!(key.contains(&variant_id.to_string()));
        
        // Verify metadata
        let head = worker_client.inner.head(&key).await.unwrap();
        assert!(head.metadata.contains_key("x-amz-meta-worker-id"));
        assert!(head.metadata.contains_key("x-amz-meta-sha256"));
    }

    #[tokio::test]
    #[ignore]
    async fn test_admin_s3_client_approve_variant() {
        let (_container, endpoint, access_key, secret_key) = start_minio().await;
        let bucket = "test-bucket";
        
        let s3_client = pipeline_core::s3::S3Client::new(
            &endpoint, bucket, &access_key, &secret_key, "us-east-1"
        ).await.unwrap();
        
        let admin_client = pipeline_core::s3::AdminS3Client::new(s3_client);
        
        // First upload a raw file
        let worker_client = pipeline_core::s3::WorkerS3Client::new(admin_client.inner.clone());
        let run_id = Uuid::new_v4();
        let design_id = Uuid::new_v4();
        let variant_id = Uuid::new_v4();
        let niche_id = Uuid::new_v4();
        let job_id = Uuid::new_v4();
        let data = b"raw image data";
        
        let raw_key = worker_client.upload_generation(
            run_id, design_id, variant_id, data.to_vec(), "png", job_id
        ).await.unwrap();
        
        // Now approve (copy raw -> curated)
        let curated_key = admin_client.approve_variant(
            &raw_key, niche_id, design_id, variant_id
        ).await.unwrap();
        
        assert!(curated_key.starts_with("curated/"));
        assert!(curated_key.contains(&niche_id.to_string()));
        assert!(curated_key.contains(&design_id.to_string()));
        assert!(curated_key.contains(&variant_id.to_string()));
        
        // Verify both exist
        assert!(admin_client.inner.exists(&raw_key).await);
        assert!(admin_client.inner.exists(&curated_key).await);
    }

    #[tokio::test]
    #[ignore]
    async fn test_presign_urls() {
        let (_container, endpoint, access_key, secret_key) = start_minio().await;
        let bucket = "test-bucket";
        
        let s3_client = pipeline_core::s3::S3Client::new(
            &endpoint, bucket, &access_key, &secret_key, "us-east-1"
        ).await.unwrap();
        
        let admin_client = pipeline_core::s3::AdminS3Client::new(s3_client);
        
        // Test presign for curation preview
        let key = "raw/test/image.png";
        // Put a test object first
        admin_client.inner.put(key, b"test".to_vec(), std::collections::HashMap::new()).await.unwrap();
        
        let presigned = admin_client.presign_curation_preview(key, 3600).await.unwrap();
        assert!(presigned.contains(key));
        assert!(presigned.contains("X-Amz-Expires=3600"));
    }
}