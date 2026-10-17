# Etsy Credentials — Full Setup Guide for the Digital Product Studio

Everything you need to create the accounts, register the app, run the OAuth consent, and fill in **Settings → Credentials → Etsy** in the studio UI. Written for our exact use case: *publishing digital download listings to your own Etsy shop, server-side, on autopilot.*

Sources: developers.etsy.com (Open API v3 docs, verified 2026-10-17).

---

## 0. What the pipeline needs (the 7 fields you'll end up entering)

| DPS field | What it is | Where you get it |
|---|---|---|
| **API Keystring** | App identifier | Your Apps page (developer portal) |
| **Shared Secret** | App secret, paired with keystring in the `x-api-key` header | Your Apps page (eye icon) |
| **OAuth2 Access Token** | 1-hour bearer token | OAuth consent flow (§4) |
| **OAuth2 Refresh Token** | 90-day token used to mint new access tokens | Same consent flow |
| **Access Token Expiry** | RFC3339 timestamp, optional (pipeline refreshes automatically) | Same consent flow |
| **Shop ID** | The shop owner's numeric user ID | getMe endpoint / token prefix (§4.5) |
| **Webhook Secret** | Optional — only if you later enable Etsy webhooks | Skip for now |

Two key facts about how Etsy auth works (per their docs):

1. **Every** API call sends `x-api-key: <keystring>:<shared_secret>` — the two values joined by a colon in one header.
2. Write operations (creating listings, uploading files) additionally need `Authorization: Bearer <access_token>` from an OAuth2 Authorization Code flow with **PKCE** (mandatory).

---

## 1. Prerequisite — an Etsy seller account with an active shop

You cannot get working credentials without an Etsy account that OWNS a shop, because:

- The **Seller App** (the app type we want — see §2) requires "an active Etsy shop in good standing".
- The OAuth consent must be granted by the shop owner — that's you.
- Listings are published INTO that shop.

**Steps:**

1. Sign up / log in at **https://www.etsy.com** (or `.com/signin/register`). Use a real email you control; you'll verify it.
2. Open a shop: **https://www.etsy.com/sell** → *Get started*. You'll choose:
   - Shop name (check availability),
   - Country, currency, and whether it's a full-time/part-time venture,
   - Shop security: 2FA is strongly recommended (a shop that gets hijacked loses your listing history).
3. First-shop setup will ask for **billing info** (credit/debit card) — required for sellers. Identity verification may also be requested. For a **digital-only** shop you skip all shipping-address questions.
4. You do NOT need to manually create a listing in the browser — the pipeline will publish listings via API. But the shop must exist and be in good standing.

> Cost note: Etsy charges **$0.20 per published listing** (4 months per listing) + transaction fees on sales. Budget ~$0.20 per product the pipeline publishes.

---

## 2. Register a Seller App in the Etsy Developer Portal

Etsy has three app types. For our use case the choice is unambiguous:

| Type | For | Approval |
|---|---|---|
| **Seller App** ← use this | Tools for **your own shop only** | **Automated, near-instant** |
| Personal App | Limited tools for OTHER sellers | Manual review, slower |
| Commercial Access | Scaled apps for many sellers | Requires approved Personal App first |

**Steps:**

1. Go to **https://www.etsy.com/developers/register-seller-app** (Developer Portal → "Create a seller app") while logged into your Etsy account.
2. Fill the form:
   - **App name** — anything, e.g. `digital-product-studio`.
   - **Description** — e.g. "Publishes my own digital art products from my internal studio pipeline to my shop."
   - **Callback URL(s)** — enter **`https://studio.amzankos.local/etsy/callback`**
     - Must be `https://`.
     - It does NOT need to be reachable from the internet — only your own browser ever lands on it (§4 explains).
     - Case-sensitive, no trailing slash, must match exactly later.
3. Submit. Eligible sellers are **approved within minutes, no manual review**.

> If the portal says you already have a seller app: Etsy allows **one seller app per shop** — use the existing one and just add our callback URL to it (three-dot menu → *Edit callback URLs*).

---

## 3. Grab the keystring + shared secret

1. Go to **https://www.etsy.com/developers/your-apps** (Developer Portal → Your Apps / "Apps You've Made").
2. Your app is listed with:
   - **Keystring** — copy it (looks like `1aa2bb33c44d55eeeeee6fff`).
   - **Shared Secret** — hidden by default; click the **eye icon** to reveal, then copy.
3. While you're there, note your app's **rate limits** (shown on the same page). Defaults are plenty for us (thousands of calls/day); the pipeline publishes a handful of products per run.

Never paste the shared secret anywhere public. Treat it like a password.

---

## 4. One-time OAuth consent → tokens (the PKCE dance)

The access token proves *you* (the shop owner) granted *your app* access. This is a one-time manual step; after it, the pipeline refreshes tokens automatically for 90 days per cycle.

We wrote a helper so you don't have to hand-craft PKCE values: **`deploy/etsy_oauth_helper.py`** in the digital-product-studio repo (stdlib-only Python, runs anywhere, including inside the `dps-rust` container).

### 4.1 Run the auth URL generator

```bash
export ETSY_KEYSTRING=your_keystring_here
export ETSY_SHARED_SECRET=your_shared_secret_here
export ETSY_REDIRECT_URI="https://studio.amzankos.local/etsy/callback"
python3 deploy/etsy_oauth_helper.py auth-url
```

It prints a URL like:

```
https://www.etsy.com/oauth/connect?response_type=code&client_id=...&redirect_uri=...&scope=listings_r%20listings_w%20shops_r%20shops_w&state=...&code_challenge=...&code_challenge_method=S256
```

Scopes requested (all four are what the publish flow needs):
- `listings_r` — read your listings (verify what's live)
- `listings_w` — **create + edit listings** (the core publish permission)
- `shops_r` — read shop info (shop id, sections)
- `shops_w` — manage shop sections (lets the pipeline organize listings into "Abstract Art", "Wedding", etc.)

### 4.2 Approve in the browser

1. Paste the printed URL into your normal browser (the one where you're logged into Etsy).
2. Etsy shows the consent screen listing the permissions → **Grant access**.
3. Etsy redirects to `https://studio.amzankos.local/etsy/callback?code=LONG_CODE&state=...`
   - The page itself may show a 404 or login wall — **irrelevant**. The authorization `code` is in the **address bar**; copy the `code=` value (everything between `code=` and `&`).
   - Works because the redirect happens in YOUR browser, which resolves `studio.amzankos.local`.
4. The code is single-use and short-lived (~minutes) — exchange it promptly.

### 4.3 Exchange the code for tokens

```bash
python3 deploy/etsy_oauth_helper.py exchange PASTE_THE_CODE_HERE
```

(with the same three env vars exported). This POSTs to `https://openapi.etsy.com/v3/public/oauth/token` with the PKCE verifier, gets the tokens, then calls **getMe** to confirm identity. It prints **every field in the exact order of the DPS form**, ready to copy:

```
API Keystring          : ...
Shared Secret          : ...
OAuth2 Access Token    : 12345678.O1zLuwvee...   (prefix = your user_id)
OAuth2 Refresh Token   : 12345678.JNGIJtvLmwf...
Access Token Expiry    : 2026-10-17T15:30:00Z
Shop ID                : 12345678
```

### 4.4 Why the address-bar code capture is legitimate

Etsy requires the redirect_uri to exactly match a registered https callback, but it never verifies the URL is publicly reachable. The redirect is executed by your local browser. This is the standard "manual copy-paste" flow used by headless/server-side integrations with no public frontend.

### 4.5 Shop ID — where the number comes from

- The **numeric prefix of the access token** (`12345678.`) is your Etsy user_id (Etsy docs: "the internal user_id of the Etsy.com user who grants the application access").
- `getMe` (`GET /v3/application/users/me`) returns `{user_id, shop_id, shop_name}`. For single-shop accounts the pipeline uses this value as the `shop_id` path parameter for listing endpoints — which is exactly what our form's hint says ("getMe → user_id").

---

## 5. Enter everything in the studio UI

Open **https://studio.amzankos.local → Settings → Credentials → Etsy** and paste:

| Form field | Paste |
|---|---|
| API Keystring | your keystring |
| Shared Secret | your shared secret |
| OAuth2 Access Token | current access token (pipeline re-mints it via refresh anyway) |
| OAuth2 Refresh Token | the long-lived refresh token |
| Access Token Expiry | optional — the RFC3339 expiry the helper printed |
| Shop ID | from getMe / token prefix |
| Webhook Secret | leave empty for now |

Hit **Save**. The values are stored **AES-256-GCM encrypted** in Postgres (`platform_credentials`, encrypted with `CREDENTIALS_ENCRYPTION_KEY`); the UI only ever shows masked values afterwards. Verify with the round-trip: save → reload the page → masked values like `a12***bc` appear.

---

## 6. What happens after you save (pipeline behavior)

- **Publish**: Listings page → a product → Publish → Etsy. The publish stage will call, in order: `createDraftListing` (`type=download`, quantity, title, description, price, `who_made=i_did`, `when_made=2020_2026`, taxonomy) → `uploadListingImage` (the set preview) → `uploadListingFile` (the bundle zip) → activate. Digital listings need **no shipping profile**.
- **Token refresh**: access tokens last **1 hour**; the pipeline uses the refresh token (valid **90 days** of rolling use) to mint new ones automatically. You only repeat §4 if the refresh token lapses (90 days of NO publishing) or you revoke the app.
- **Rate limits**: enforced per API key (QPS/QPD, sliding 24h window, headers `x-limit-per-second` / `x-limit-per-day`). Our volume (a few listings per run) is nowhere near defaults.
- **Re-consent triggers**: changing scopes (e.g. adding `listings_d`) requires redoing §4 with the new scope list.

---

## 7. Troubleshooting

| Symptom | Cause / fix |
|---|---|
| Consent redirect 404s at `studio.amzankos.local/etsy/callback` | **Expected** — the code is in the address bar. |
| "redirect_uri mismatch" at consent | Callback URL in app ≠ the one in the auth URL. Case-sensitive, no trailing slash, exact host. Fix in Your Apps → Edit callback URLs, and re-run `auth-url`. |
| `401 invalid_request` at token exchange | Code expired (minutes) or already used; PKCE verifier not from the same `auth-url` run (the helper saves it — don't regenerate between steps). |
| `401` on API calls | Wrong `x-api-key` format — it must be `keystring:shared_secret` (colon-joined) AND a valid Bearer token for scoped endpoints. |
| Seller app registration blocked | Shop not active / not in good standing, or you already have one seller app on this shop (reuse it). |
| Tokens stopped working after ~90 idle days | Refresh token expired — rerun §4 (consent + exchange), update the two token fields. |

---

## 8. Security notes

- Shared secret + refresh token = full control of listing creation on your shop. Keep them in the DPS credential store (encrypted) — never in chat logs, git, or plain files.
- The helper script stores the transient PKCE verifier in `.etsy_pkce_state.json` next to itself and deletes it after exchange.
- Rotate by re-running §4 any time; new consent invalidates nothing else.

**Quick reference — all the Etsy URLs in this guide**

| Purpose | URL |
|---|---|
| Register seller app | https://www.etsy.com/developers/register-seller-app |
| Your Apps (keystring + secret) | https://www.etsy.com/developers/your-apps |
| OAuth consent | https://www.etsy.com/oauth/connect |
| Token exchange / refresh | https://openapi.etsy.com/v3/public/oauth/token |
| getMe | https://openapi.etsy.com/v3/application/users/me |
| API docs | https://developers.etsy.com/documentation/reference |
