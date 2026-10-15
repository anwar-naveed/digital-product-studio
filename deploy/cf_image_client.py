#!/usr/bin/env python3
"""
cf_image_client.py — Minimal client for the Cloudflare Workers AI image-generation bridge.

Endpoint: https://free-image-generation-api.nav-anwar021.workers.dev/v1
API:      OpenAI Images API compatible
            POST /v1/images/generations  {"model": "cf/...", "prompt": "...", "size": "WxH", "n": 1}
              → {"created": ..., "data": [{"b64_json": "<base64 image>"}]}
            GET  /v1/models              → OpenAI-style model catalog
            GET  /                       → health check (via /v1 root)
Auth:     Bearer token in the Authorization header.

Usage (CLI):
    python3 cf_image_client.py "a serene mountain lake at sunrise"
    python3 cf_image_client.py "cyberpunk city" --model cf/black-forest-labs/flux-1-schnell --size 1024x1024
    python3 cf_image_client.py "logo sketch" --n 2 --out-dir ./out --prefix logo
    python3 cf_image_client.py --models
    python3 cf_image_client.py --health

Usage (Python):
    from cf_image_client import generate, list_models
    paths = generate("a cat astronaut, studio lighting")   # → ["image_0.png", ...]
    print(list_models())

Env overrides:
    CF_IMAGE_BASE_URL   (default: the production worker URL below)
    CF_IMAGE_API_KEY    (default: the key below)

Zero dependencies — Python 3 stdlib only (urllib), safe to run anywhere.
"""

import argparse
import base64
import json
import os
import sys
import urllib.error
import urllib.request

BASE_URL = os.environ.get(
    "CF_IMAGE_BASE_URL",
    "https://wild-snowflake-0e48.nav-anwar021.workers.dev/v1",
).rstrip("/")

DEFAULT_API_KEY = os.environ.get(
    "CF_IMAGE_API_KEY",
    "c8831188a39ea7973572d09b0478b38dc54d7683b3ebd99e61194c6c5cec7c40",
)

DEFAULT_MODEL = "@cf/black-forest-labs/flux-2-klein-4b"


def _request(method, path, api_key, payload=None, timeout=300):
    """Single HTTP request to the bridge. Returns parsed JSON. Raises RuntimeError on HTTP errors."""
    url = f"{BASE_URL}{path}"
    data = json.dumps(payload).encode("utf-8") if payload is not None else None
    req = urllib.request.Request(url, data=data, method=method)
    req.add_header("Authorization", f"Bearer {api_key}")
    req.add_header("Content-Type", "application/json")
    # Cloudflare error 1010 blocks the default "Python-urllib/x.y" signature — send a browser UA.
    req.add_header(
        "User-Agent",
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) "
        "Chrome/126.0.0.0 Safari/537.36",
    )
    req.add_header("Accept", "application/json")
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            return json.loads(resp.read().decode("utf-8"))
    except urllib.error.HTTPError as e:
        body = e.read().decode("utf-8", errors="replace")
        try:
            msg = json.loads(body).get("error", {}).get("message", body)
        except (ValueError, AttributeError):
            msg = body
        raise RuntimeError(f"HTTP {e.code}: {msg}") from None
    except urllib.error.URLError as e:
        raise RuntimeError(f"Connection failed: {e.reason}") from None


def _ext(raw):
    """Detect image format from magic bytes."""
    if raw.startswith(b"\x89PNG\r\n\x1a\n"):
        return ".png"
    if raw.startswith(b"\xff\xd8\xff"):
        return ".jpg"
    if raw.startswith(b"GIF8"):
        return ".gif"
    if raw.startswith(b"RIFF") and raw[8:12] == b"WEBP":
        return ".webp"
    return ".png"  # Workers AI default output is PNG


def health(api_key=DEFAULT_API_KEY):
    """GET /v1/models — doubles as auth+service health check (401 = bad key, 200 = alive)."""
    return list_models(api_key)


def list_models(api_key=DEFAULT_API_KEY):
    """GET /v1/models → list of allowed model ids."""
    out = _request("GET", "/models", api_key)
    return [m["id"] for m in out.get("data", [])]


def generate(
    prompt,
    model=DEFAULT_MODEL,
    size="1024x1024",
    n=1,
    api_key=DEFAULT_API_KEY,
    negative_prompt=None,
    steps=None,
    guidance=None,
    seed=None,
    out_dir=".",
    prefix="image",
):
    """Generate image(s) from a prompt. Decodes b64_json and saves files.

    Returns: list of saved file paths.
    """
    payload = {"model": model, "prompt": prompt, "size": size, "n": n}
    for key, val in (
        ("negative_prompt", negative_prompt),
        ("steps", steps),
        ("guidance", guidance),
        ("seed", seed),
    ):
        if val is not None:
            payload[key] = val

    result = _request("POST", "/images/generations", api_key, payload)

    os.makedirs(out_dir, exist_ok=True)
    paths = []
    for i, item in enumerate(result.get("data", [])):
        raw = base64.b64decode(item["b64_json"])
        path = os.path.join(out_dir, f"{prefix}_{i}{_ext(raw)}")
        with open(path, "wb") as f:
            f.write(raw)
        paths.append(path)
    if not paths:
        raise RuntimeError("Bridge returned no images in response data")
    return paths


def main():
    p = argparse.ArgumentParser(description="Generate images via Cloudflare Workers AI bridge")
    p.add_argument("prompt", nargs="?", help="Text prompt for the image")
    p.add_argument("--model", default=DEFAULT_MODEL, help=f"Model id (default: {DEFAULT_MODEL})")
    p.add_argument("--size", default="1024x1024", help='Image size, e.g. "1024x1024" (256-2048, multiple of 64)')
    p.add_argument("--n", type=int, default=1, help="Number of images (1-4, default 1)")
    p.add_argument("--negative-prompt", default=None, help="Negative prompt (model-dependent)")
    p.add_argument("--steps", type=int, default=None, help="Diffusion steps (model-dependent)")
    p.add_argument("--guidance", type=float, default=None, help="Guidance scale (model-dependent)")
    p.add_argument("--seed", type=int, default=None, help="Random seed for reproducibility")
    p.add_argument("--out-dir", default=".", help="Directory to save images (default: cwd)")
    p.add_argument("--prefix", default="image", help="Output filename prefix (default: image)")
    p.add_argument("--models", action="store_true", help="List available models and exit")
    p.add_argument("--health", action="store_true", help="Health/auth check (calls GET /v1/models)")
    args = p.parse_args()

    try:
        if args.health or args.models:
            models = list_models()
            if args.models:
                print(json.dumps(models, indent=2))
            else:
                print(f"OK — service alive, auth valid, {len(models)} models available")
            return 0
        if not args.prompt:
            p.error("prompt is required (or use --models / --health)")
        paths = generate(
            args.prompt,
            model=args.model,
            size=args.size,
            n=args.n,
            negative_prompt=args.negative_prompt,
            steps=args.steps,
            guidance=args.guidance,
            seed=args.seed,
            out_dir=args.out_dir,
            prefix=args.prefix,
        )
        print(json.dumps({"created": True, "files": paths}))
        return 0
    except RuntimeError as e:
        print(json.dumps({"error": str(e)}), file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
