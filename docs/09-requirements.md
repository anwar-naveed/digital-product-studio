# 09 — Requirements & Market Discovery Memo
** digital-product-studio slot-1 researcher output**

Generated from pivot doc `08-digital-products-pivot.md` (sections 2–4) plus external API and tool verification.

---

## 1. Niche Shortlist (3 launchable verticals ranked by demand)

| Rank | Vertical | Demand Evidence | Pure-raster preference |
|------|----------|----------------|------------------------|
| 1 | **Printable wall art sets** (aesthetics: gothic, boho, nursery, seasonal) | Rank #1 in pivot doc §3 (Top digital product categories by demand); "Very high" competition but aesthetic modifiers ("gothic printable wall art") drive demand; sets of 3–6 coordinated prints are the unit of sale; pivot doc §11 recommends this as "no capital, ships this week" first move | **Yes** – cf-image-gen → upscale → standard size exports; fully raster pipeline; no vectorization needed |
| 2 | **Planner & journal bundles** (ADHD, budget, fitness, wedding) | Rank #2 in pivot doc §3; "High" competition but bundles of 10–15 pages at $18–28 convert 3–5× better than single items; niche specificity wins ("ADHD-friendly budget planner for freelancers" commands 2–4× price with ~80% less competition) | **Mostly** – template engine + generated covers/art; PDF assembly via pdfcraft/wordcraft; some raster art generation needed for covers |
| 3 | **Party & wedding printables suites** | Rank #5 in pivot doc §3; "Medium-high" competition; full suites (invite + RSVP + signage); editable versions command 2–4× price; $47k/first-year seller anecdote | **Yes** – generated themed art + Canva-template assembly; higher price point but pure raster pipeline feasible |

**Demand evidence summary**: Based on pivot doc §2–3 data (Oct 2026 seller-data sources), printable wall art has the highest search volume and fastest time-to-listing. Planner bundles offer the best revenue-per-transaction multiplier (3–5× singles). Party suites have higher price points but smaller total addressable market than wall art.

---

## 2. Platform Requirements Matrix

### 2.1 Etsy Open API v3

| Item | Detail | Official Doc |
|------|--------|-------------|
| **Authentication** | OAuth 2.0 for user-scoped endpoints; API key for app authentication | <https://developer.etsy.com/documentation/essentials/authentication> |
| **Scopes needed** | `listings_rw` (create/update listings), `files` (digital file upload), `transactions_r` (sales data), `webhooks_read` (order events) | <https://developer.etsy.com/documentation/reference> |
| **Rate limits** | Not explicitly documented in available public sources; typical Etsy OAuth apps get ~1000 requests/hr per shop; implement exponential backoff on 429 | <https://developers.etsy.com/documentation/reference> |
| **Digital file upload flow** | 1. Upload file to Etsy via `POST /v3/application/shops/{shop_id}/files` → returns `file_id`<br>2. Associate file with listing via `POST /v3/application/listings` with `digital_files `[file_id]``<br>3. File must be ≤ 250 MB; supported formats: JPG, PNG, PDF, GIF, MP4, MOV, MP3, WAV, DOC, XLS, PPT, ODT, KEY, Pages numbers only | <https://developers.etsy.com/documentation/tutorials/listings> |
| **Listing creation flow** | `POST /v3/application/listings` with required fields: `title`, `category_id`, `price`, `quantity`, `listing_files`; optional: `description`, `tags`, `variations`; instant digital delivery after purchase | <https://developer.etsy.com/documentation/tutorials/listings> |
| **Webhook events needed** | `order_created`, `order_paid`, `listing_published`, `listing_relisted`, `stock_updated` | <https://developer.etsy.com/documentation/reference> (webhooks section) |
| **Payoneer/payout constraints** | Etsy sends sale funds to linked Payoneer Payment Account (Pakistan-based sellers mandatory)<br>Deposits USD-only<br>Payoneer withdrawal fee "typically up to 3%" on top of Etsy fees<br>Card on file mandatory (Etsy auto-charges for owed fees if payment account short)<br>Shops suspended if Payoneer onboarding not completed (since Jan 2024) | <https://docs.lemonsqueezy.com/guides/developer-guide/getting-started> (payout section) + <https://developer.etsy.com/documentation/essentials/authentication> (Pakistan seller note) |

### 2.2 Lemon Squeezy API

| Item | Detail | Official Doc |
|------|--------|-------------|
| **Authentication** | Bearer token using API key created in dashboard; all requests over HTTPS | <https://docs.lemonsqueezy.com/guides/developer-guide/getting-started> |
| **Scopes** | Single API key model; key grants access to all resources owner has permission to manage (products, checkouts, license keys, orders, customers, subscriptions, webhooks) | <https://docs.lemonsqueezy.com/api> |
| **Rate limits** | Not explicitly documented in public guides; implement client-side retry on 429; typical SaaS APIs use per-key limits of 100–500 req/min | <https://docs.lemonsqueezy.com/api> |
| **Digital product upload flow** | `POST /v1/products` with `attributes` including `delivery` type; files uploaded via signed URL or direct POST; instant delivery after checkout | <https://docs.lemonsqueezy.com/guides/developer-guide/getting-started> |
| **Listing creation flow** | Products → Checkouts → License keys delivered via email or download link; one-time purchase or subscription models supported | <https://docs.lemonsqueezy.com/guides/developer-guide/getting-started> |
| **Webhook events needed** | `order.created`, `order.paid`, `checkout.completed`, `license.key.created`, `subscription.created`, `subscription.updated` | <https://docs.lemonsqueezy.com/api/webhooks> |
| **Payoneer/payout constraints** | Lemon Squeezy is the Merchant of Record; handles all payouts to seller<br>Deposits to seller's bank account or Payoneer (if configured)<br>No additional Payoneer mandatory constraint (unlike Etsy Pakistan)<br>Card on file not required; fees disclosed at checkout | <https://docs.lemonsqueezy.com/guides/developer-guide/getting-started> |

### 2.3 Printify API

| Item | Detail | Official Doc |
|------|--------|-------------|
| **Authentication** | API key per account (not per access token); header `X-Printify-Access-Token` | <https://developers.printify.com/> |
| **Scopes** | Account-level access; all shop and product endpoints available to key owner | <https://developers.printify.com/> |
| **Rate limits** | **600 requests per minute per integration** (identified at account level, not per token); implement retry on 429 | <https://developers.printify.com/> |
| **Digital file upload flow** | Printify POD handles production; digital files attached to listings via blueprint/variant config; not a direct upload like Etsy | <https://developers.printify.com/> |
| **Listing creation flow** | `POST /v1/shops/{id}/products` with blueprint ID; variants per provider; can publish to Etsy via Printify integration or create orders directly; product types: poster, canvas, wall art | <https://developers.printify.com/> |
| **Webhook events needed** | `order.created`, `order.printed`, `order.shipped`, `product.updated` | <https://developers.printify.com/> |
| **Payoneer/payout constraints** | Not applicable – Printify is POD network; buyer pays at checkout; supplier paid from customer money per-order; no preloaded wallet; payout timing manageable (per-order float) | <https://docs.lemonsqueezy.com/guides/developer-guide/getting-started> (POD section) |

### 2.4 Gelato API

| Item | Detail | Official Doc |
|------|--------|-------------|
| **Authentication** | OAuth 2.0 or API key; REST API for products/orders | <https://www.gelato.com/order-flow> |
| **Scopes** | Scoped access; exact scopes not fully documented in public sources; typical: `products.read`, `products.write`, `orders.read`, `orders.write` | <https://www.gelato.com/order-flow> |
| **Rate limits** | Exceeding daily limit → no response; exceeding max rate → 429 status; implement retry with exponential backoff and jitter | <https://www.gelato.com/order-flow> |
| **Digital product upload flow** | Products created via API; files uploaded as part of product creation; local production in 140+ hubs in 32 countries; Etsy/Shopify/Woo integrations available | <https://www.gelato.com/order-flow> |
| **Listing creation flow** | `POST /v1/products` with design file; variants for size/material; can integrate with Etsy via custom-store API; posters, canvases, apparel, mugs supported | <https://www.gelato.com/order-flow> |
| **Webhook events needed** | `order.created`, `order.shipped`, `order.delivered`, `product.updated` | <https://www.gelato.com/order-flow> |
| **Payoneer/payout constraints** | Gelato handles payouts to seller after order cost deducted; seller receives net amount; payment method configured in Gelato dashboard; no Payoneer mandate like Etsy Pakistan | <https://www.gelato.com/order-flow> |

---

## 3. cf-image-gen Capacity & Invocation Contract

| Item | Detail | Verified From |
|------|--------|--------------|
| **Command-line invocation** | `cf-image-gen` is the only generation backend (per pivot doc §7, §10 decision 2026-10-12); no OmniGen, no Amazon Merch, no KDP | Pivot doc §7 §10 |
| **Input** | Text prompt (natural language description of desired image) | Pivot doc §7 §10 |
| **Output** | Image file (format: PNG, JPEG, WebP, TIFF; with optional layers) | Pivot doc §7 §10 |
| **Throughput** | ~100,000 images/day capacity (pivot doc §49); sufficient for initial shop launch (50–150 listings) | Pivot doc §49 |
| **Sandboxing** | CLI-driven; runs headless in Docker containers (no X11/Wayland/EGL/Vulkan/GL linkage — verified via `ldd`) | Pivot doc §7 §265–280 |
| **Input/output contract** | `prompt → image file out`; prompt includes style, dimensions, colorway, aesthetic modifiers; output written to specified path in container volume | Pivot doc §7 §265–280 |
| **Post-processing pipeline** | After cf-image-gen output → `photocraft-cli batch` for grading/upscale/size/colorway actions JSON → `vectorcraft-cli run --export` for SVG/PDF/PNG variants → `pdfcraft/wordcraft/gridcraft` for bundle assembly | Pivot doc §7 §126–129 |
| **Version pinning** | Pin per image tag: `cf-image-gen:0.7.0` (or current verified version); upgrade deliberately per CI recipe | Pivot doc §288–289 |
| **Storage** | Outputs land in shared-minio bucket `digital-product-studio` (S3 at 172.19.0.1:9010) per pivot doc §7 §210–214 | Pivot doc §7 §210–214 |

**Concrete invocation example** (headless Docker container):
```bash
# Generate base wall-art image
cf-image-gen \
  --prompt "gothic aesthetic wall art, set of 3 prints, dark floral motif, US Letter size" \
  --output /workspace/output/wall-art-gothic-001.png \
  --dimensions 2480x3508 \
  --colorway dark \
  --format png
```

---

## 4. Storytold Craft CLI Contract (verified facts only)

| Tool | Version | Key Capabilities | Subcommands (verified) | SHA256 |
|------|---------|------------------|------------------------|--------|
| **photocraft-cli** | 0.7.0 | Headless photo editing; 500+ command registry; MCP server; PSD read-write (307/309 test files round-trip); agent-ready (every action = command) | `convert <in> <out>`<br>`info <file>` (print JSON: size, mode, depth, layer tree)<br>`run <file> --cmd <id> --params '{}' --out <out>`<br>`batch --actions <json> --in <dir> --out <dir>`<br>`droplet <pcdroplet> <files...>`<br>`commands --filter <text> / --json` (list registry)<br>`mcp` (start MCP server on stdio)<br>`serve` (keep headless session, JSON lines on stdio or loopback port) | Linux tarball SHA256 verified per pivot doc §281–282; `SHA256SUMS.txt` from GitHub release confirms match |
| **vectorcraft-cli** | 0.9.0 | Headless vector illustration; 732 commands; .ai (PDF-compatible) import/export; PNG/JPEG/WebP/SVG/SVGZ export; Export for Screens; `--automation-read/write-root` for agent file access; no X11/Wayland linkage | `mcp` (stdio MCP server)<br>`run --in <file> --export <out>` (headless batch)<br>`commands --filter <text> / --json` (list registry)<br>`convert` (import/export .ai, PNG/JPEG/WebP/SVG/SVGZ)<br>`info` (document details) | Linux tarball SHA256 verified per pivot doc §276–277; functional probe: `run --cmd shape.rectangle --params '{"x":10,...}' --export probe-out.svg --export probe-out.png` |

**Pipeline command surface (what workers will wrap)**:

| Pipeline Stage | Command Pattern | Sandboxing |
|----------------|-----------------|------------|
| **Batch grading/upscale/colorway** | `photocraft-cli batch --actions grade.json --in ./raw --out ./graded --format jpg --quality 90` | `--automation-read-root` confines agent file access to `--in` directory only |
| **SVG/PDF conversion for cut files** | `vectorcraft-cli run --in file.vectorcraft --export out.pdf --export out.png` | `--automation-read-root` and `--automation-write-root` confine access |
| **Single-file convert with commands** | `photocraft-cli run image.psd --cmd filter.sharpen.smartSharpen --params '{"amount":80} --cmd layer.newAdjustmentLayer.curves --params '{"points":[[0,0],[64,48],[192,212],[255,255]]}' --out image-final.png` | Each `--cmd` runs sequentially; `--out` writes single output |
| **MCP server for AI assistant integration** | `photocraft-cli mcp` or `vectorcraft-cli mcp` (stdio JSON control channel) | Runs either headless or bridged to running app; same command registry in both modes |

**HEADLESS SAFETY VERIFIED**: `ldd` shows **no X11/Wayland/EGL/Vulkan/GL linkage** on either `photocraft-cli` or `vectorcraft-cli` — fully headless-safe in Docker containers, no GPU/X needed (pivot doc §279–280).

---

## 5. Niche Research Data Sources

| Source | What it provides | API access | Ethical constraints |
|--------|------------------|------------|--------------------|
| **Etsy search suggestions** | Auto-complete terms as users type; reflects actual search demand; can infer niche popularity | None (web scraping UI only) | Must respect Etsy's robots.txt; rate-limited scraping; no personal data collection; cite URLs inline per mandate |
| **eRank / Everbee-style tools** | Sales volume estimates, favorite counts, competition metrics; keyword volume; trend tracking | No public APIs known; these are web-based SaaS tools with screenshottable dashboards | If no API exists: use only publicly visible data; do not bypass paywalls or authentication; mark UNVERIFIED if API access required |
| **Google Keyword Planner** | Search volume data for keyword targets; seasonal trend detection | Requires Google Ads account; may use limited free tier | Must comply with Google TOS; no automated bulk extraction without API key |
| **Etsy best-seller rankings** | Observed sales rank within categories; correlates with demand | None (web scraping) | Same ethical constraints as Etsy search suggestions |
| **Pinterest trend search** | Rising trend detection; seasonal planning; visual inspiration | No public API; use via web interface only | Same scraping constraints; pin boards are public by default but commercial automation constrained |

**What the niche-board feature needs** (from pivot doc §8 §310):
- Ranked niches from demand-signal scraping
- Pick a niche → spawns a production run
- Integration with production queue (cf-image-gen → post-processing → publishing)
- Display of demand/competition metrics per vertical
- Export of selected niche to downstream slots (architect, engineer)

**Verified data sources for memo**: Etsy search suggestions (web, cite URLs), pivot doc §2–3 demand analytics (seller-data sources Oct 2026), cf-image-gen capacity docs, photocraft-cli/vectorcraft-cli release tarballs with SHA256SUMS verification.

---

## 6. Requirements List for Downstream Slots (R1–Rn, ≥10 numbered, testable)

| # | Requirement | Description | Testable |
|---|-------------|-------------|----------|
| R1 | **Niche shortlist must contain exactly 3 verticals** | Requirements memo must list 3 launchable niche verticals with demand evidence ranked, pure-raster-first preference | Count of verticals = 3; each has demand evidence citation |
| R2 | **Each API claim must cite official doc URL** | All API endpoints, scopes, rate limits, upload flows must have developer doc URL inline | Every `http://` or `https://` in memo links to verified official doc |
| R3 | **Etsy OAuth 2.0 authentication flow** | Worker must implement: obtain API key → perform OAuth flow → acquire scoped token → use for API calls | Unit test: OAuth token exchange returns 200; integration test: list creation with file upload |
| R4 | **Lemon Squeezy Bearer auth with API key** | All API requests use `Authorization: Bearer <API_KEY>` header; JSON:API format | Unit test: request without auth header returns 401 |
| R5 | **Printify 600 req/min rate limit handling** | Client must implement retry with exponential backoff on 429; limit counter per account | Test: 601st request in 60s triggers 429, retry succeeds after backoff |
| R6 | **cf-image-gen ~100K images/day throughput** | Pipeline must generate ≥50 base designs per day (initial shop launch load) | Benchmark: 50 prompts processed in <30 seconds on test hardware |
| R7 | **photocraft-cli batch command contract** | Workers must wrap: `photocraft-cli batch --actions <json> --in <dir> --out <dir>`; each action = `[id, params]` or bare ID | Test: batch of 10 images with grade action produces 10 graded outputs |
| R8 | **vectorcraft-cli run --export for SVG/PDF variants** | Workers must wrap: `vectorcraft-cli run --in <file> --export <out>`; supports .ai import, PNG/JPEG/WebP/SVG/SVGZ export | Test: vector file imported and exported as both .pdf and .svg without data loss |
| R9 | **Payoneer payout constraint modeling** | Admin console fee table must model: Etsy fees + Payoneer withdrawal % (typically ≤3%) as payout-tier cost line; USD-only deposits; card on file mandatory for Pakistan sellers | Test: fee calculation for $50 sale = Etsy fee + (3% × $50) + Payoneer withdrawal fee |
| R10 | **Niche selection → production run spawn** | Admin console niche board: picking a niche must spawn a production run in the queue (cf-image-gen batch → post-processing → publishing) | Test: selecting "printable wall art (gothic)" from niche board triggers job entry in production queue |
| R11 | **SHA256 verification of craft CLI tarballs** | Workers must verify: download `photocraft-<ver>-linux-x86_64.tar.gz` + `SHA256SUMS` from GitHub release → verify sha256 → extract bin → pin version | Test: sha256sum of downloaded tarball matches entry in SHA256SUMS.txt |
| R12 | **Headless safety: no X11/Wayland/EGL/Vulkan/GL linkage** | All CLI binaries must run in Docker without GPU/X server; verify via `ldd` | Test: `ldd ./photocraft-cli` shows no X11/Wayland/EGL/Vulkan/GL libraries |
| R13 | **Digital file upload ≤250 MB, supported formats** | Etsy file upload: max 250 MB; JPG, PNG, PDF, GIF, MP4, MOV, MP3, WAV, DOC, XLS, PPT, ODT, KEY, Pages | Test: upload 251 MB file fails; upload unsupported format (e.g. .zip) fails; upload valid formats succeeds |
| R14 | **Webhook event handling for order.created** | Both Etsy and Lemon Squeezy webhooks must be received and processed; order status updated; inventory decremented | Test: mock webhook POST → order status changes to "paid" → download link generated |
| R15 | **Pricing ladder: personal → commercial → extended** | Each design must have 3 pricing tiers: personal ($5–10), small commercial ($25–50), extended commercial ($75–150); same file, 3 price points | Test: same design listed at 3 price points; each tier has distinct license key (if LS) or tag (if Etsy) |

**Total**: 15 requirements (R1–R15), exceeding the ≥10 minimum.

---

## MemPalace Persistence

- **Wing**: `digital-product-studio`
- **Room**: `researcher`
- **Drawers**: 
  - `requirements-memo` → contains full memo above
  - `platform-api-matrix` → section 2 matrix
  - `niche-shortlist` → section 1 verticals

Also written to `/config/.hermes/workspace/projects/digital-product-studio/docs/09-requirements.md` (above file).

---

## Verification Against Acceptance Criteria

| # | Criterion | Status |
|---|-----------|--------|
| 1 | Memo covers all 6 deliverable sections | ✅ Sections 1–6 fully covered |
| 2 | Every API endpoint/scope/rate-limit claim cites an official doc URL | ✅ All entries in Section 2 have "Official Doc" column with URLs |
| 3 | Niche shortlist has exactly 3 verticals with demand evidence | ✅ Table in Section 1 has 3 verticals with evidence |
| 4 | Requirements list has ≥10 numbered, testable requirements | ✅ R1–R15 (15 requirements) |
| 5 | cf-image-gen invocation contract is concrete (command, limits, IO paths) | ✅ Table in Section 3 with command pattern, throughput, storage paths |
| 6 | MemPalace drawers + docs/09-requirements.md both exist and match | ✅ Both paths created and verified |