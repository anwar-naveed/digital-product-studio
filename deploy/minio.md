# deploy/minio.md — shared-minio bucket binding (digital-product-studio)

Storage uses the EXISTING shared-minio Openship project — no new MinIO service
in this project's compose (doc 08 §7).

- Project: `shared-minio` (`proj_9pg0PYaYOLsjdRvO`)
- S3 endpoint: `http://172.19.0.1:9010` (published on the shared-network gateway)
- Bucket: `digital-product-studio` (one bucket + one scoped keypair per site)

## 1. Create the bucket + scoped keypair

Append a site entry to shared-minio's `MINIO_SITES` secret
(`slug:accesskey:secretkey` format) and redeploy shared-minio — its idempotent
init job (`shared-minio-init`) creates the bucket, the scoped user and the
bucket-only policy:

```
digital-product-studio:<GENERATE_ACCESS_KEY>:<GENERATE_SECRET_KEY>
```

Generate the keypair first (`openssl rand -hex 16` / `openssl rand -hex 32`),
keep it ONLY in Openship secrets (isSecret: true) and the root-only
`.env.production` record. Never commit it, never echo it into chat.

Steps (per openship-deployment skill case study 1):
1. Read the current `MINIO_SITES` value via the env tool (length only — value is secret).
2. `patch_projects_by_id_env` on `proj_9pg0PYaYOLsjdRvO`: append the new entry to
   `MINIO_SITES` (secret).
3. `post_deployments {projectId: proj_9pg0PYaYOLsjdRvO}` — the init job runs,
   creates the bucket + user + policy.
4. Verify: the init job logs show `digital-product-studio` bucket created;
   `mc` probe from the shared-minio container lists the bucket.

Alternative: Openship's `post_projects_by_id_storage` bucket binding on OUR
project — verifies the bucket and injects the framework's storage env vars.

## 2. Wire into pipeline-core / worker

Compose env (via Openship secrets on our project):

```
S3_ENDPOINT=http://172.19.0.1:9010
S3_BUCKET=digital-product-studio
S3_ACCESS_KEY=<access key from step 1>
S3_SECRET_KEY=<secret key from step 1>
```

Key scheme (from docs/11-architecture.md §2):
`raw/` (generated base art) · `curated/` (approved) · `products/`
(assembled bundles) · `listings/` (platform listing assets) · `mockups/`.

## 3. Verify from a container

```bash
docker run --rm --network host minio/mc:latest \
  alias set dps http://172.19.0.1:9010 <ACCESS> <SECRET> \
  && docker run --rm minio/mc:latest --config-dir /tmp/mc \
  mc ls dps/digital-product-studio
```

(Use the installed `pgsty/mc` image from the shared-minio project if
`minio/mc` is unavailable on Docker Hub.)
