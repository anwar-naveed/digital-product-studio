# deploy/openship.md — Openship deployment runbook (digital-product-studio)

Per `devops/openship-deployment` skill + doc 08 §7. Golden rules: never
`docker compose up` on an Openship project; restart with `docker restart
openship-<project>-<service>`; ports publish on `${HOST_BIND_IP:-172.19.0.1}`;
secrets via Openship `isSecret: true`, never in committed `.env`.

## 1. Repo

- Private GitHub repo `anwar-naveed/digital-product-studio` (token: `/config/.gitpat`).
- Push the project as-is (pipeline-core, worker, adapters, admin-ui, deploy/,
  `.github/workflows/build.yml`). The workflow builds ALL images on GitHub-hosted
  runners (HARD CLAUSE 2) — nothing is ever built on this server.

## 2. Secrets for the Openship deployment

Set via `post_deployments_build_access` `envVars` (all `isSecret: true`):

| Var | Purpose |
|---|---|
| `DATABASE_URL` | Postgres connection string (pipeline-core + worker) |
| `S3_ENDPOINT` | `http://172.19.0.1:9010` (shared-minio, proj_9pg0PYaYOLsjdRvO) |
| `S3_BUCKET` | `digital-product-studio` |
| `S3_ACCESS_KEY` / `S3_SECRET_KEY` | scoped keypair from shared-minio `MINIO_SITES` append `digital-product-studio:<access>:<secret>` + redeploy shared-minio |
| `ETSY_CLIENT_KEY` / `ETSY_CLIENT_SECRET` / `ETSY_REFRESH_TOKEN` | Etsy Open API v3 OAuth2 |
| `LEMONSQUEEZY_API_KEY` | LS storefront adapter |
| `PRINTIFY_API_KEY` / `GELATO_API_KEY` | POD adapters |
| `WEBHOOK_SECRET` | HMAC verification for Etsy/LS webhook ingestion |
| `ADMIN_SESSION_SECRET` | admin-ui session signing |
| `CORE_IMAGE` / `WORKER_IMAGE` / `UI_IMAGE` | CI-pushed tags (committed `.env`, NOT Openship vars — Openship vars would override CI bumps; see skill golden rule 5) |

Non-secret `.env` (committed): image tags (`CORE_IMAGE=<owner>/digital-product-studio-core:<sha>`,
etc.), `PORT_*` assignments, `STUDIO_HOST=studio.amzankos.local`.

## 3. Compose (deploy/docker-compose.yml)

Openship requires: no `env_file:` (unsupported by its parser) — use `${VAR:?}` /
`${VAR:-default}`; each service `image: ${X_IMAGE:?}` so CI bumps flow via git;
ports published on the gateway `172.19.0.1` only; volumes as named volumes or
absolute HOST paths (`/opt/vps-ui/webtop/config/...`), never `./relative`.

Services: `pipeline-core` (8080), `pipeline-worker`, `admin-ui` (nginx 8080→host
3081), `postgres` (supabase/postgres or postgres:16, data in named volume).
No `minio` service — storage is the EXISTING shared-minio project
(`http://172.19.0.1:9010`), bucket `digital-product-studio`.

## 4. Deploy sequence

```text
post_deployments_prepare (repo, branch=main)
→ post_projects (hasBuild: false; images come from CI)
→ post_deployments_build_access (secrets from table above)
→ poll deployment → verify containers:
   docker ps | grep openship-digital-product-studio
   docker inspect openship-digital-product-studio-pipeline-core \
     --format '{{.Config.Image}} | {{.State.Health.Status}}'
```

## 5. Pingap + cert (webtop-internal host)

- Host: `studio.amzankos.local` — resolves only inside the webtop (Docker
  embedded DNS / hosts entry), like `openship.amzankos.local`.
- Cert: leaf `CN=studio.amzankos.local`, `SAN DNS:studio.amzankos.local`,
  fullchain = leaf + Amzanko Local Root CA
  (`/config/.hermes/workspace/projects/amzankos-trader/backend/devops/ca/myCA.*`).
  (Alternative: self-signed like the existing openship cert — see doc 08 §7 note.)
- Pingap conf (projects/amzankos-trader/pingap/conf/):
  - `certificates.toml`: `[certificates.studio]` entry
  - `upstreams.toml`: `[upstreams.studioUpstream] addrs = ["172.19.0.1:<core-or-ui-port>"]`
  - `locations.toml`: `[locations.studioUi] host = "studio.amzankos.local"`, `path = "/"`
    with X-Forwarded-* header set; plus a `/api/` location → core upstream.
- No public DNS, no Let's Encrypt, no public port.

## 6. Post-deploy verification

```bash
docker ps --format 'table {{.Names}}\t{{.Image}}\t{{.Status}}' | grep openship-digital-product-studio
docker inspect <container> --format '{{.Config.Image}} | started {{.State.StartedAt}} | {{.State.Health.Status}}'
docker logs --since 5m <container> 2>&1 | tail -30
curl -sk -o /dev/null -w '%{http_code}\n' https://studio.amzankos.local/     # via Pingap, inside webtop
curl -sk https://studio.amzankos.local/api/v1/health
```

Host cleanliness (HARD CLAUSE 4): `which cargo rustc node npm` → empty;
no `node_modules/`, no `target/` under the project on this host.
