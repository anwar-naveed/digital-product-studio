#!/usr/bin/env python3
"""Etsy OAuth2 helper for the Digital Product Studio pipeline.

Runs the PKCE token dance for YOUR OWN shop (Seller App) and prints every
credential field exactly as the DPS Settings -> Credentials -> Etsy form
expects them.

Usage:
  1. python3 etsy_oauth_helper.py auth-url
     -> prints the consent URL; open it in a browser, log into Etsy, approve.
     -> browser lands on your callback URL with ?code=...  — copy that code
        from the address bar (the page itself may 404, that's fine).
  2. python3 etsy_oauth_helper.py exchange <CODE>
     -> exchanges the code for tokens, calls getMe, prints all field values.

Config via env vars (or edit the defaults below):
  ETSY_KEYSTRING      - from https://www.etsy.com/developers/your-apps
  ETSY_SHARED_SECRET  - same page (eye icon reveals it)
  ETSY_REDIRECT_URI   - the callback URL registered on the app
                        (must match EXACTLY, case-sensitive, https)
  ETSY_SCOPES         - default: listings_r listings_w shops_r shops_w
"""
import base64
import hashlib
import json
import os
import secrets
import sys
import urllib.parse
import urllib.request

API_TOKEN_URL = "https://openapi.etsy.com/v3/public/oauth/token"
AUTH_URL = "https://www.etsy.com/oauth/connect"
ME_URL = "https://openapi.etsy.com/v3/application/users/me"
STATE_FILE = os.path.join(os.path.dirname(os.path.abspath(__file__)), ".etsy_pkce_state.json")

KEYSTRING = os.environ.get("ETSY_KEYSTRING", "")
SHARED_SECRET = os.environ.get("ETSY_SHARED_SECRET", "")
REDIRECT_URI = os.environ.get("ETSY_REDIRECT_URI", "https://studio.amzankos.local/etsy/callback")
SCOPES = os.environ.get("ETSY_SCOPES", "listings_r listings_w shops_r shops_w")


def make_pkce():
    """PKCE verifier (43-128 chars, url-safe base64 of 32 random bytes) + S256 challenge."""
    verifier = base64.urlsafe_b64encode(secrets.token_bytes(32)).rstrip(b"=").decode()
    challenge = base64.urlsafe_b64encode(
        hashlib.sha256(verifier.encode()).digest()
    ).rstrip(b"=").decode()
    return verifier, challenge


def cmd_auth_url():
    if not KEYSTRING:
        sys.exit("Set ETSY_KEYSTRING env var first.")
    verifier, challenge = make_pkce()
    state = secrets.token_urlsafe(24)
    json.dump({"verifier": verifier, "state": state}, open(STATE_FILE, "w"))
    params = {
        "response_type": "code",
        "client_id": KEYSTRING,
        "redirect_uri": REDIRECT_URI,
        "scope": SCOPES,
        "state": state,
        "code_challenge": challenge,
        "code_challenge_method": "S256",
    }
    url = AUTH_URL + "?" + urllib.parse.urlencode(params)
    print("\n1) Open this URL in your browser, log into Etsy, approve access:\n")
    print(url)
    print("\n2) Etsy redirects you to:")
    print("   " + REDIRECT_URI + "?code=XXXX&state=" + state)
    print("\n   Copy the FULL code value from the address bar (page may 404 - fine).")
    print("\n3) Run:  python3 etsy_oauth_helper.py exchange <CODE>\n")


def http_form(url, data):
    req = urllib.request.Request(
        url,
        data=urllib.parse.urlencode(data).encode(),
        headers={"Content-Type": "application/x-www-form-urlencoded"},
    )
    with urllib.request.urlopen(req, timeout=30) as r:
        return json.loads(r.read())


def cmd_exchange(code):
    if not KEYSTRING:
        sys.exit("Set ETSY_KEYSTRING env var first.")
    saved = json.load(open(STATE_FILE)) if os.path.exists(STATE_FILE) else {}
    verifier = saved.get("verifier")
    if not verifier:
        sys.exit("No saved PKCE verifier - run 'auth-url' first and redo the consent.")

    resp = http_form(API_TOKEN_URL, {
        "grant_type": "authorization_code",
        "client_id": KEYSTRING,
        "redirect_uri": REDIRECT_URI,
        "code": code,
        "code_verifier": verifier,
    })
    access_token = resp["access_token"]
    refresh_token = resp["refresh_token"]
    expires_in = resp.get("expires_in", 3600)
    user_id_prefix = access_token.split(".")[0]

    # Confirm identity + fetch shop_id via getMe
    req = urllib.request.Request(ME_URL, headers={
        "x-api-key": f"{KEYSTRING}:{SHARED_SECRET}" if SHARED_SECRET else KEYSTRING,
        "Authorization": f"Bearer {access_token}",
    })
    with urllib.request.urlopen(req, timeout=30) as r:
        me = json.loads(r.read())

    import datetime
    expires_at = (datetime.datetime.now(datetime.timezone.utc)
                  + datetime.timedelta(seconds=expires_in)).strftime("%Y-%m-%dT%H:%M:%SZ")

    print("\n================ PASTE THESE INTO DPS: Settings -> Credentials -> Etsy ================\n")
    print(f"API Keystring          : {KEYSTRING}")
    print(f"Shared Secret          : {SHARED_SECRET}")
    print(f"OAuth2 Access Token    : {access_token}")
    print(f"OAuth2 Refresh Token   : {refresh_token}")
    print(f"Access Token Expiry    : {expires_at}   (expires in {expires_in // 60} minutes - the pipeline auto-refreshes)")
    print(f"Shop ID                : {me.get('shop_id') or user_id_prefix}"
          f"   (getMe user_id: {me.get('user_id')}, shop_name: {me.get('shop_name')})")
    print("\nScopes granted         :", resp.get("scope"))
    print("========================================================================================")
    print("\nNOTE: access token expires in 1 hour, but the pipeline refreshes it automatically")
    print("      with the refresh token. Re-do the consent flow (auth-url + exchange) only if the")
    print("      refresh token goes unused for 90+ days or is revoked.")
    os.remove(STATE_FILE)


def main():
    if len(sys.argv) < 2 or sys.argv[1] not in ("auth-url", "exchange"):
        print(__doc__)
        sys.exit(1)
    if sys.argv[1] == "auth-url":
        cmd_auth_url()
    else:
        if len(sys.argv) < 3:
            sys.exit("Usage: exchange <CODE>")
        cmd_exchange(sys.argv[2])


if __name__ == "__main__":
    main()
