# 08 — Digital-Products Pivot: Demand Analytics & Asset Production Plan

**Why this pivot solves the capital problem:** digital products eliminate the
seller-platform wallet/float entirely. There is no supplier to pre-pay — the product
is an artifact we generate ourselves (image gen, open-source Adobe-class tooling),
marketplace delivery is instant download, and every sale is pure margin minus
platform fees. Zero working capital, zero inventory, zero policy exposure on
sourcing. This is the "intermediary" model collapsed to its purest form: **we are
both the seller platform and the seller**.

**Loan research:** dropped per your instruction — noted as dead end.

## 1. Platform policy reality check (research 2026-10)

| Platform | Digital products allowed? | Constraints |
|---|---|---|
| **eBay** | Yes, but gated. Electronically delivered items are restricted to **Everything Else → Information Products** category (classified-ad format) unless eBay-approved as a digital seller; application/apply process required; no automated-download links for unapproved sellers — delivery is typically email/attachment after sale. Not a high-volume channel; fine for ebooks/guides/asset packs sold as "information products." |
| **Amazon** | Kindle Direct Publishing (ebooks), **Merch on Demand** (designs on physical goods — Amazon prints, we earn royalty; zero fulfillment by us, zero capital), Amazon Apps, and digital software via specific programs. Merch on Demand requires application + tier system (start tier caps daily uploads, tiers rise with sales). **June 2026: royalty model changed to three-tier traffic-based royalties** — older flat-rate guides are outdated. |
| **Etsy** | Digital downloads = a top category. Instant automated delivery. Listing fee $0.20 + ~6.5% transaction + payment processing. This is where the printable/SVG/template market actually lives. |
| **Gumroad** | No approval gate; 10% flat fee (2026); best with external traffic. Median creator revenue modest (~$650/mo in saturated design category) — long tail is brutal without niche. |
| **Creative Market / Creative Fabrica** | Curated/marketplace for design assets (fonts, templates, graphics, SVG). Reviewer gate for quality; good fit for high-volume asset production. |
| **Lemon Squeezy (you have it)** | Merchant of record for digital products — **perfect fit now**. Checkout + global tax + instant delivery for our own storefront; also the MoR if we ever sell this pipeline as SaaS. |

## 2. Demand analytics — what actually sells (2026 snapshot)

### Top digital product categories by demand
| Rank | Category | Price band | Competition | Notes |
|---|---|---|---|---|
| 1 | **Printable wall art** | $5–15/set | Very high | Etsy's dominant digital search; aesthetic modifiers ("gothic printable wall art") drive demand; sets of 3–6 coordinated prints |
| 2 | **Digital planners & journals** | $8–28/bundle | High | ADHD planners, budget, meal, student planners; bundles of 10–15 pages at $18–28 convert 3–5× better than single items |
| 3 | **Educational printables** | $4–20/bundle | Medium | Homeschool boom; flashcards, worksheets, classroom decor |
| 4 | **Wedding printables** | $25–65/suite | Medium-high | Full suites (invite + RSVP + signage); editable versions command 2–4× price; $47k/first-year seller anecdote |
| 5 | **Party printables** | $8–30/set | Medium-high | Complete party kits by theme (dinosaur, mermaid, unicorn) |
| 6 | **SVG cut files (Cricut/Silhouette)** | $3–8/bundle | Very high | Among Etsy's most-downloaded digital products; large catalogues of well-tagged niche bundles win |
| 7 | **Business templates** | $12–45/bundle | Medium | Social templates, invoices, brand kits; industry-specific niches |
| 8 | **Digital stickers (GoodNotes/Notability)** | $3–8/sheet | Very high | Hyperlinked formats premium |
| 9 | **Fonts & typography packs** | $10–30 | Medium | Harder to produce; fontlab/AI-assisted pipelines exist |
| 10 | **AI prompt packs / photo presets / LUTs** | $5–40 | Rising | Newer category; saturation risk high; niching essential |

**Key market patterns (from seller-data sources, Oct 2026):**
- **Bundling beats singles**: 10–15 item bundles at $18–28 outperform individual $3–5 items 3–5× on revenue per transaction.
- **Niche specificity wins**: "ADHD-friendly budget planner for freelancers" commands 2–4× price with ~80% less competition than generic "budget planner."
- **Sizes/formats matter**: US Letter + A4 + A5 variants, PDF+JPG+PNG, editable Canva links — expected, not premium.
- **Timing**: 2027 calendars list Sep–Oct 2026; seasonal/holiday kits list 4–6 weeks ahead.
- **Licensing ladder**: personal use ($5–10) → small commercial ($25–50) → extended commercial ($75–150) — same file, 3 price points.
- **Saturation warning**: generic design assets on Gumroad median ≈ $650/mo. Differentiation = niche + problem-solving + quality previews, not volume of me-too products.

### Where our production capacity creates an edge
We can generate **~100,000 images/day** (cf-image-gen) plus the **storytold
ArtCraft suite** — pure-Rust, MIT/Apache-2.0, clean-room reimplementations of the
Adobe tools, each with a **headless CLI + MCP server + command registry** so an
agent (or our worker pipeline) can drive every feature programmatically:

| Tool (storytold) | Adobe equivalent | Automation surface (verified from repo) |
|---|---|---|
| **photocraft** (50.4k★) | Photoshop | `photocraft-cli run file.psd --cmd ... --params '{...}' --out x.png`; `photocraft-cli batch --actions grade.json --in ./raw --out ./graded`; `photocraft-cli mcp` — MCP server (headless or bridged to running app); 500+ command registry; loopback JSON control channel; recorded-action playback; full PSD/PSB read-write (307/309 corpus round-trip) |
| **vectorcraft** (7.5k★) | Illustrator | `vectorcraft-cli mcp` (stdio MCP server); `vectorcraft-cli run --in file.vectorcraft --export out.pdf` headless batch; command API = the same one the MCP server exposes; .ai (PDF-compatible) import/export; PNG/JPEG/WebP/SVG/SVGZ export; Export for Screens; automation read/write roots for agents; no-panic engine with guarded commands |
| **artcraft** (19.9k★) | AI creation IDE ("the IDE for artists") | ~~OmniGen API~~ — **excluded per user decision 2026-10-12: no API access**; generation stays on cf-image-gen. Still potentially useful for **manual 3D scene crafting via its MCP surface**: image-to-3D-mesh, character posing, scene blocking/kitbashing, 3D compositing — but verify whether artcraft's MCP toolset works against local scene state or requires the OmniGen backend before relying on it (unverified; gate behind a spike) |
| **lightcraft** (10.6k★) | Lightroom | non-destructive photo editor; same craft automation family |
| **filmcraft** (10.7k★) | Premiere Pro | nonlinear video editor (Rust) |
| **effectcraft** (5.4k★) | After Effects | motion graphics/compositing |
| **gridcraft** (3.3k★) | Excel | spreadsheets — relevant for template products |
| **wordcraft** (7.4k★) | Word | document editor — ebook/worksheet/template assembly |
| **pdfcraft** (10.3k★) | Acrobat | PDF reader/editor — bundle/PDF product assembly |
| **cadcraft** (3k★) | AutoCAD | CAD — niche asset products (DXF/SVG files) |
| **soundcraft** (2k★) | Pro Tools | audio — sound-effect packs, music loops |

**Why this is a step-change vs generic open-source tools (GIMP/Inkscape):**
these are agent-first by design — every menu action maps to a command callable from
CLI, JSON control channel, or MCP; that means our Kanban workers can drive the
entire Adobe-class workflow headlessly in Docker containers (no GUI, no X11),
which fits the house Docker-only rule perfectly. License is MIT OR Apache-2.0 —
commercial sale of produced artifacts is unambiguous; only the "ArtCraft" brand and
bundled third-party assets carry restrictions (check `NOTICE`/`ATTRIBUTION.md` per
app; use our own fonts/assets).
market's constraint is NOT production volume — it's **niche discovery + SEO +
listing quality**. Therefore the strategy is:

1. **Catalog depth per niche** — a winning shop has 50–150 listings; we can saturate
   a chosen niche in days, not months.
2. **Variant multiplication** — each design automatically exported as wall-art set,
   planner insert, sticker sheet, SVG-cut version (where vectorizable), in 3 sizes
   and 2–3 colorways = dozens of listings per base design.
3. **Data-driven niche selection** — scrape/track Etsy search & sales proxies
   (Everbee-class signals via public listing counts/favorites), rank niches by
   demand/competition, then mass-produce into the winner.

## 3. Recommended product verticals (ranked for our toolset)

| # | Vertical | Why it fits our capacity | Effort to first listing |
|---|---|---|---|
| 1 | **Printable wall art sets** (niche aesthetics: gothic, boho, Scandinavian, nursery, seasonal) | cf-image-gen output → upscale → standard size exports; pure raster pipeline | Days |
| 2 | **Planner & journal bundles** (ADHD, budget, fitness, wedding) | Template engine + generated covers/art; PDF assembly scripted | 1–2 weeks |
| 3 | **Party & wedding printables suites** | Generated themed art + Canva-template assembly; higher price point | 2–3 weeks |
| 4 | **SVG cut-file bundles** (Cricut niches) | Needs vectorization pass (potrace-style tracing of generated art); highest repeat-purchase audience | 2–4 weeks |
| 5 | **Digital stickers for GoodNotes** | Raster pipeline + hyperlinked packaging script | 1–2 weeks |
| 6 | **Merch-on-Demand designs** (Amazon) | Same image gen; Amazon prints & ships; royalty income; no fulfillment, no capital; application + tier ramp required | Weeks (application gate) |
| 7 | **AI prompt packs / preset bundles** | Meta-product from our own pipeline knowledge; low marginal cost | Days |

## 4. Distribution strategy

1. **Etsy = primary storefront** (instant delivery, biggest digital-download buyer
   pool, predictable fees). Niche shops per vertical; 50–150 listings each.
2. **Lemon Squeezy = own branded storefront** (you already have the gateway):
   bundles, extended licenses, and email-list capture — the only channel where we
   own the customer relationship and pay no marketplace cut beyond card fees.
3. **Creative Market / Creative Fabrica** for bulk design-asset plays.
4. **Amazon Merch on Demand** as a parallel royalty channel for the same design
   catalog (reprint of our raster work on physical goods — zero capital, zero
   fulfillment).
5. **eBay Information Products** opportunistically for guide-type products where
   classified-ad format is acceptable — lowest priority (gated, manual delivery).
6. **Gumroad** as backup/direct channel, low priority without an audience.

## 5. Pipeline architecture (digital-product-studio)

The original bridge architecture from dropship-bridge doc 04 collapses into a
simpler **production-and-listing pipeline** (own Rust/Go core, own Openship
project — this project shares no pipeline, host, or storage bucket with
dropship-bridge):

```
 niche research (scrape demand signals) ─▶ niche ranking board (UI)
        │
        ▼
 production queue ─▶ cf-image-gen (base art) — the ONLY generation backend
        │           + photocraft-cli batch (grade/upscale/size/colorway actions JSON)
        │           + vectorcraft-cli run --export (SVG/PDF/PNG product variants, .ai compat)
        │           + pdfcraft/wordcraft/gridcraft (planner/worksheet/template/bundle assembly)
        │           + artcraft MCP [gated spike] (manual 3D scene crafting: mesh posing,
        │             kitbashing, compositing — NOT generation; verify local-state MCP
        │             viability before pipeline commitment)
        ▼
 product assembly (bundle PDFs, metadata, pricing ladder, mockups)
        │
        ▼
 listing publisher ─▶ Etsy API (bulk create/update) · Lemon Squeezy API
        │              · Creative Market (manual/CSV) · Amazon Merch (upload)
        ▼
 sales analytics ◀─ order/webhook feeds ── listing performance tracker
        │
        └──▶ repricer/retirer: double-down on winners, retire dead listings
```

- Etsy API (OAuth, listing CRUD, digital file upload) and Lemon Squeezy API
  (products, checkouts, webhooks) are both well-documented — same adapter-trait
  pattern from doc 03 applies.
- Production workers scale horizontally in Docker; image storage in MinIO/volume.
- Everything else from the house rules applies unchanged: Openship deploys, Pingap
  edge, no host builds, secrets in vault, runtime-updatable config.

## 6. Honesty notes (from the research)

- "Passive income" framing in seller blogs is survivorship-heavy: **median** Etsy
  digital shop income is modest; winners are niche + SEO + volume machines. Our edge
  is production volume + automation + data-driven niching, not luck.
- Etsy digital downloads is competitive but demand is real (digital downloads
  est. $1.2B+/yr category); printables carry 80–95% margins.
- AI-generated art is widespread on Etsy already (many top printables are AI-assisted);
  differentiator is curation/consistency/niche fit, and disclosed quality. Etsy
  requires sellers to disclose AI use per its creativity policy if asked/when
  material to the buyer — check current Etsy seller policy at listing time.
- Amazon Merch June-2026 royalty restructure (three-tier, traffic-based) makes most
  pre-2026 "Merch income" guides unreliable — model royalties from the live program
  terms only.
- storytold suite caveats: (a) **status is pre-1.0 / active development** (vectorcraft
  v0.9–0.10, README badge "in active development") — production use should pin
  releases and keep an export-format regression test per pipeline; (b) apps are
  desktop-first — automation runs via their CLIs (headless `run`/`batch`) and MCP
  servers inside Linux containers; verify container builds work for the specific
  tool before committing a pipeline stage to it; (c) only produced artifacts are
  unambiguously ours to sell — keep bundled third-party assets, fonts and the
  ArtCraft brand OUT of products (read each repo's `NOTICE`/`ATTRIBUTION.md`);
  (d) models reachable through artcraft/OmniGen are third-party services with their
  own output licenses — for commercial resale prefer cf-image-gen and tools where
  we control the model license, or verify each provider's commercial terms.
- (removed 2026-10-12) the OmniGen exclusion is user-locked: **cf-image-gen is the
  only generation backend**; craft tools are for post-processing/assembly/scene work.

## 7. Web UI & deployment (Openship)

**Can the stack be used via web UI? Yes — partially, by design.**

- The craft engines are **desktop-first, but the engine + UI compile to WebAssembly
  and run in the browser**: PhotoCraft ships a static `photocraft-web-<ver>.zip`
  ("host it on any static server", with a documented **Docker web-hosting** recipe
  in `packaging/web/README.md`); VectorCraft also compiles to WASM
  (`apps/vectorcraft-web`); the Crafting Apps share the convention "native on
  macOS, Windows and Linux, in the browser via WebAssembly, and fully drivable by
  agents."
- **Architecture implication:** two deployment tiers.
  1. **Pipeline tier (headless, Openship):** workers run the CLIs
     (`photocraft-cli batch`, `vectorcraft-cli run --export`) in containers — no
     GUI. This is where products get made.
  2. **Editor tier (web): DEFERRED by user decision 2026-10-12 — not a priority.**
     The WASM builds (photocraft-web/vectorcraft-web zips) would be self-hosted as
     static sites behind Pingap for **your own** manual touch-up only. **Out of
     scope for the initial build**: the entire production pipeline runs headless
     via the CLIs, and curation is preview-based approve/reject in the admin
     console (no editor involved). The recipe stays in this doc (download official
     release web zips in CI → static nginx image → Pingap route) so the tier can
     be added later with zero pipeline changes if ever wanted — it is purely
     additive and touches no other service.
- **Openship fit:** exactly the house pattern, as its OWN Openship project +
  repo (separate from dropship-bridge). Compose services: `pipeline-core`
  (our Rust orchestrator), `pipeline-workers` (CLI-driven craft containers),
  `admin-ui` (React console). **No editor-tier service in the initial build**
  (deferred, see above). **No `asset-store` service either — storage is the
  existing `shared-minio` Openship project** (`proj_9pg0PYaYOLsjdRvO`,
  S3 at `http://172.19.0.1:9010`, bucket+scoped access key per site, added by
  appending `digital-product-studio:accesskey:secretkey` to its `MINIO_SITES`
  secret and redeploying; or bind the bucket via
  `post_projects_by_id_storage`). Ports on `172.19.0.1`; secrets
  `isSecret: true`.
- **Admin UI hosting — webtop-internal only (user directive 2026-10-12):** the
  console is NOT exposed as `pipeline.<domain>`; it follows the
  `openship.amzankos.local` pattern — a `.local` hostname that resolves only
  inside the webtop, served by Pingap with a certificate signed by the existing
  **Amzanko Local Root CA** (`backend/devops/ca/myCA.*`). **This is the
  digital-product-studio project — separate from dropship-bridge** (separate
  host, cert, bucket; only the research lineage is shared). Concretely:
  - Host: `studio.amzankos.local` (admin UI + its API on one host).
  - Cert: leaf for `CN=studio.amzankos.local` with `SAN DNS:studio.amzankos.local`,
    fullchain = leaf + myCA — same layout as `openship.amzankos.local.fullchain.crt`.
  - Pingap: `[certificates.studio]` entry + upstream `studioUpstream` pointing
    at `172.19.0.1:<port>` + `[locations.studioUi]` with
    `host = "studio.amzankos.local"`, `path = "/"`.
  - Webtop `/etc/hosts` (or DNS) already resolves `*.amzankos.local` — same as
    openship/desktop hosts.
  - No public DNS, no Let's Encrypt, no public port. Accessed only from the
    webtop browser where the CA is already trusted.
  - **Verification note (live probe 2026-10-12):** the existing
    `openship.amzankos.local` cert is actually **self-signed**
    (`issuer = subject = CN=openship.amzankos.local`, openssl-verify against
    myCA fails), while the `desktop.amzankos.local` cert IS myCA-issued with
    SANs. So "just like openship" literally = self-signed leaf; the myCA path
    (as used for desktop) is the cleaner option if the browser should trust it
    without a per-site exception. Either pattern works through Pingap; pick at
    cert-generation time.
- **Image sourcing — verified 2026-10-12 (pre-built first, GitHub Actions only as
  fallback, per user directive):**
  1. **Pre-built status of craft apps (checked live):**
     - storytold publishes **no official container images** — releases are
       native binaries + static WASM zips (`photocraft-web-<ver>.zip` etc. from
       GitHub Releases, built by their CI `release.yml`, not published to any registry).
     - Third-party **ghcr.io/daemonless/photocraft** / **vectorcraft** etc. exist,
       but are **FreeBSD-only images** (`docker manifest inspect` shows only
       `os: freebsd`, amd64 + arm64) — they cannot run on our Linux Docker host
       (pull fails: "no matching manifest for linux/amd64"). NOT usable.
     - **photocraft ships a community-maintained `Dockerfile`** (multi-stage:
       rust builder + Trunk WASM build → nginx:alpine runtime, port 8080,
       `/healthz`, precompressed gzip assets) — vectorcraft and the other craft
       repos have **no Dockerfile** (404), only `packaging/web/package.sh` static-zip scripts.
  2. **Resulting image strategy per service:**
     - **WASM editor tier (web curation):** for photocraft, prefer the
       **official release artifact** `photocraft-web-<version>.zip` downloaded in
       CI (no build at all — it's the exact binary their CI produced) and packed
       into a trivial static-server image; the community Dockerfile is fallback.
       vectorcraft (no Dockerfile): same pattern — CI downloads its
       `vectorcraft-web` release artifact if shipped, else builds once via a
       fork-pinned workflow. **Never build on this server** — the Openship skill's
       rule (GitHub-hosted runner builds, server only pulls) stands; the only
       change is the CI step is `download release zip + COPY into nginx` instead
       of `cargo build`, which is faster, cheaper, and exactly the upstream binary.
     - **Headless CLI tier (pipeline workers):** **PROBED AND RESOLVED 2026-10-12 —
       zero-compile path confirmed (option (a) won).** The Linux desktop tarballs
       **contain both binaries**: `photocraft-0.7.0-linux-x86_64/bin/{photocraft,
       photocraft-cli}` and `vectorcraft-0.9.0-linux-x86_64/bin/{vectorcraft,
       vectorcraft-cli}`. Verified end-to-end in scratch:
       - `photocraft-cli 0.7.0` runs headless: 858 commands in registry;
         functional probe `run test.png --cmd filter.sharpen.smartSharpen
         --params '{"amount":80}' --out test-graded.png` → JSON result, output
         written. Subcommands: convert/info/run/batch/droplet/commands/mcp/serve.
       - `vectorcraft-cli 0.9.0` runs headless: 732 commands; functional probe
         `run --cmd shape.rectangle --params '{"x":10,...}' --export probe-out.svg
         --export probe-out.png` → both exports written. Subcommands:
         mcp/run/commands/convert/info; `--automation-read/write-root` confine
         agent file access.
       - `ldd` shows **no X11/Wayland/EGL/Vulkan/GL linkage** on either CLI —
         fully headless-safe in containers, no GPU/X needed.
       - Upstream `SHA256SUMS.txt` verifies the tarballs (photocraft linux
         tarball sha256 match confirmed).
       **CI recipe (no Rust toolchain anywhere):** download
       `photocraft-<ver>-linux-x86_64.tar.gz` + SHA256SUMS from the GitHub
       release → verify sha256 → extract `bin/photocraft-cli` (and
       `vectorcraft-cli` from its tarball) → `COPY` into
       `debian:bookworm-slim` → push → Openship pulls. Option (b) (fork-pin
       CI build) is no longer needed. **Pin the version** per image tag
       (`photocraft-cli:0.7.0`), upgrade deliberately.
     - **pipeline-core / admin-ui (our own code):** these are OURS — the standard
       GitHub Actions build→Docker Hub→Openship pipeline applies (no pre-built
       option exists for code we write).
- **artcraft via web:** artcraft itself is an AI studio whose generation side
  needs the OmniGen backend — with that excluded, artcraft is NOT deployed as a
  web service. If the MCP spike (§5) proves local 3D-scene crafting works, artcraft
  stays a **desktop tool driven by agents over MCP**, not a web tier.
- **WASM limits to expect:** early-alpha web builds (a third-party review notes
  "same early-alpha limits as the desktop app"); GPU compositing may be reduced vs
  native; large-file handling is browser-constrained. Fine for curation workflows;
  heavy batch work stays on the headless tier.

## 8. The admin console (what it's FOR in this pivot)

The dropship-bridge admin-ui design was the starting sketch, so here is the
concrete use case for THIS project's console — it is **not** an order monitor;
it's the **production & publishing management console**:

| Console page | What it manages |
|---|---|
| **Niche research board** | ranked niches from demand-signal scraping; pick a niche → spawns a production run |
| **Production runs** | queue of generation/assembly jobs: cf-image-gen batches, photocraft/vectorcraft CLI jobs; status, retries, artifacts preview; curation queue (approve/reject each generated design before it becomes a listing — remember: curation IS the product) |
| **Catalog & listings** | every SKU: which platforms it's published to (Etsy, LS, Printify/Gelato), listing state, price, licensing tier (personal/commercial/extended), which bundle contains it |
| **Publishing status** | per-platform listing sync state: published / failed / delisted; Etsy API or LS API errors surfaced for retry |
| **Sales & analytics** | per-listing views/favorites/sales pulled from platform APIs; margin after fees; winners/losers → retire/double-down decisions |
| **Settings** | API credentials (Etsy OAuth, LS API key, Printify/Gelato keys) via vault; fee tables; pricing ladder rules |

So yes — your reading is correct: **it's the "we know what we published where, and
how it's doing" system**. The Rust backend (pipeline-core) exposes this as REST;
the React console is just the window onto it. It's also the bridge surface for the
API integrations:

- **Etsy Open API v3** — REST + OAuth2: shop/listing management, digital-file
  upload/association per listing, order/sales data. This is the primary
  listing-publisher + sales-analytics adapter.
- **Lemon Squeezy API** — products, checkouts, license keys, orders + webhooks.
  Our own storefront adapter.
- **Etsy payouts for Pakistan: CONFIRMED via Payoneer** (Etsy Payments Policy,
  updated 2026-07-31, + Etsy Help "Countries Eligible for Etsy Payments"):
  sellers resident in **Pakistan** are in the Payoneer-mandatory group — Etsy
  sends sale funds to a linked **Payoneer Payment Account** (not directly to a
  local bank), and the seller withdraws from Payoneer to their bank. Key terms
  to plan around: deposits to Payoneer are **USD-only**; Payoneer withdrawal
  fees "typically up to 3%" apply on top of Etsy fees; a **card on file is
  mandatory** (Etsy auto-charges it for owed fees if the payment account is
  short); shops are suspended if Payoneer onboarding isn't completed (since
  Jan 2024 for this program). Fee tables in the admin console must therefore
  model: Etsy fees + Payoneer withdrawal % as a payout-tier cost line.
- **Printify / Gelato APIs** (see §9) — POD publishing adapters for physical
  print products.

## 9. Amazon Merch excluded — POD alternatives (user context: KDP ban)

**Amazon Merch on Demand is OUT** for us: the account application runs through
the same Amazon risk posture that already banned this user's KDP account — approval
is unlikely and attempting it risks deeper account entanglement. The royalty-
channel slot in the distribution strategy is instead served by **POD networks
reachable via API**, all of which integrate with Etsy/marketplaces AND offer
direct order creation:

| Platform | Why it fits | API |
|---|---|---|
| **Printify** | 80+ print providers; blueprints/variants per provider; publish to Etsy via its integration or create orders directly; poster/canvas/wall-art products (matches our wall-art catalog) | REST + webhooks (`/v1/shops/{id}/products`, `/orders.json`) |
| **Gelato** | 140+ production hubs in 32 countries (local production = fast+cheap shipping); posters, canvases, apparel, mugs; Etsy/Shopify/Woo integrations + custom-store API | REST API for products/orders |
| **Printful** | strongest branding options (inside labels, packing slips); posters/framed prints; Etsy integration | REST + webhooks |
| **Redbubble / Society6** (optional, no API) | royalty-style passive marketplaces for art uploads — no API, manual/bulk upload only; treat as low-effort secondary channels | none (manual) |

**How POD works for us with zero capital:** buyer orders the physical print on
Etsy (or our LS storefront) → order webhook hits pipeline-core → pipeline calls
Printify/Gelato create-order with the buyer's ship-to → POD provider prints,
white-labels, ships → tracking flows back → Etsy marked shipped. The supplier is
paid from the customer's money (payout timing is manageable because POD order
costs are per-order, not prepaid in bulk — the wallet problem shrinks to a
per-order float, and Printify/Gelato/Printful bill on order placement from a
payment method, not a preloaded balance).

⚠️ One policy note: POD "dropshipping" through Etsy is **fully allowed** — Etsy
explicitly supports print-on-demand as a production partner (you must list the
production partner on the listing per Etsy's manufacturing disclosure). This is
NOT the marketplace-arbitrage risk from the physical dropshipping plan.

## 10. "Printables" — two different products, don't conflate

The word "printable" covers two different business flows in this plan:

1. **Digital printable downloads** (PDF/JPG/SVG the buyer prints themselves):
   our pure-digital product. Buyer pays on Etsy/LS → instant automated download
   (Etsy: upload file via API, buyer downloads from purchase history; LS: delivers
   via its checkout). No printing, no shipping, 80–95% margin. This is the core
   pivot product.
2. **Physical printed products** (buyer receives a real poster/canvas/mug):
   we do NOT print. We publish the SAME design catalog through a POD network
   (§9) — Etsy buyer orders physical print → POD partner prints & ships
   white-label → we're the seller of record, production partner disclosed.

So "if a user wants printables" — both are covered by one catalog, two delivery
channels: **download** (instant, pure margin) and **printed & shipped** (POD
per-order cost, no capital, no inventory). The listing pipeline tags each design
with both capabilities where it makes sense (wall art: both; planners: download
only; mugs/apparel: POD only).

## 11. Recommended first move (no capital, ships this week)
1. Pick ONE niche vertical — suggestion: **printable wall-art sets for a specific
   aesthetic** (validated demand, pure raster pipeline, fastest to first listing).
2. Generate + curate 10 base designs (image gen + manual curation for quality —
   curation IS the product), 5 sizes each, 3 sets of bundles.
3. Open Etsy shop + Lemon Squeezy product for the same bundles; list 15–30 SKUs.
   (Printify/Gelato POD publishing for the same wall-art designs is the natural
   phase-2 add-on — same catalog, physical channel; Amazon Merch excluded per
   KDP-ban context.)
4. Instrument listing performance (favorites/views/sales per day) in the analytics UI.
5. Reinvest nothing but compute; scale the winning set; retire losers weekly.

## 12. Tier split — what the pipeline needs vs. what's for you

| Concern | Belongs to |
|---|---|
| Image generation (cf-image-gen), grading/upscale/size/colorway (photocraft-cli), vectorization + SVG/PDF exports (vectorcraft-cli), bundle assembly, Etsy/LS/POD listing publishing | **Headless pipeline tier — no web UI involved, fully automated** |
| Curation decisions | **Admin console** (preview-based approve/reject grid) — no editor needed for this |
| Publishing status, sales analytics, fees/margin | **Admin console** |
| Manual touch-up of a design that's almost right | ~~Editor tier~~ **DEFERRED** (user decision 2026-10-12) — regen-or-ship instead; recipe preserved in §7 for later, zero pipeline coupling |
