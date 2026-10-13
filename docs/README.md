# Digital Product Studio — Project Overview

> **Self-hosted, automated digital product creation & publishing platform.**
> Spun off from the dropship-bridge research (2026-10-12) as a SEPARATE project:
> no shared pipeline, host, storage bucket, certificates, or code with the
> dropshipping project. This project = generate → curate → assemble → publish →
> analyze. The dropshipping project = physical-goods arbitrage bridging. They
> share research lineage only.

## Docs

| # | File | What it holds |
|---|------|---------------|
| 08 | [08-digital-products-pivot.md](./08-digital-products-pivot.md) | The full pivot plan: demand analytics, platform policy reality, distribution strategy (Etsy primary + Lemon Squeezy + POD), production pipeline (cf-image-gen + storytold craft CLIs), deployment (Openship + shared-minio + webtop-internal host), admin console spec, image-sourcing verification, Etsy/Payoneer payout mechanics. Continues the 08 numbering so research references stay intact. |

## Identity (host / certs / storage — project-unique, do NOT collide with dropship-bridge)

| Item | Value |
|---|---|
| Webtop host | `studio.amzankos.local` (NOT `dropship.*`) |
| Pingap cert files | `/etc/pingap/certs/studio.amzankos.local.fullchain.crt` + `.key` (leaf + Amzanko Local Root CA, SAN `DNS:studio.amzankos.local`) |
| Pingap conf entries | `[certificates.studio]`, upstream `studioUpstream` (`172.19.0.1:<port>`), `[locations.studioUi]` (`host = "studio.amzankos.local"`, `path = "/"`) |
| S3 storage | shared-minio project (`proj_9pg0PYaYOLsjdRvO`, `http://172.19.0.1:9010`), bucket name **`digital-product-studio`** (NOT dropship-bridge's) with its own scoped access key |
| Deploy | Openship project (own compose, own repo) — `pipeline-core`, `pipeline-workers`, `admin-ui` services |

## Scope rules

- Editor tier (WASM web zips): **deferred** — not in initial build.
- cf-image-gen: **the only generation backend** (OmniGen excluded, user-locked).
- Amazon Merch: **excluded** (KDP-ban context) — Printify/Gelato/Printful instead.
- Everything headless; curation = preview approve/reject grid in admin-ui.
- Docs 01–07 of dropship-bridge remain there; only research cross-references point across.

## Cross-references (research lineage, read-only)

- dropship-bridge docs 01–07 live at `../dropship-bridge/docs/` — the dropshipping
  (physical goods) research and implementation plan. The use of shared-minio and
  the Openship deployment pattern is shared house infrastructure, not shared
  project scope.
