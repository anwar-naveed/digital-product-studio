#!/usr/bin/env python3
"""product-assembly: turn a curated design variant into a print-ready bundle.

Runs inside a sandboxed, network-less container spawned by the DPS worker.
CLI contract (identical to the other DPS tools):

    product-assembly run \
        --params-file /workspace/params/job-<id>.json \
        --automation-read-root /workspace/read \
        --automation-write-root /workspace/write \
        --out products/<design_id>/bundle.zip

params.json carries: design_id, variant_id, curated_key, base_prompt, niche_id.

Behaviour:
  * loads the curated image staged by the worker at <read_root>/inputs/<curated_key>
  * renders a standard 5-ratio print pack at 300 DPI (2:3, 3:4, 4:5, 11:14, ISO A)
  * merges the pack into the design's bundle.zip (append-safe: approving more
    variants of the same design grows the set bundle instead of replacing it)
  * writes README + license + manifest so the bundle is a complete Etsy-style
    digital download the publish stage can push to marketplaces as-is.

Stdlib + Pillow only. No network, no shell, exits 0 with a JSON summary.
"""
from __future__ import annotations

import argparse
import datetime
import hashlib
import io
import json
import os
import sys
import tempfile
import zipfile

from PIL import Image, ImageOps, ImageFilter

DPI = 300
BG = (255, 255, 255)  # print-safe white letterbox

# ratio-slug -> (width_px, height_px, label) at 300 DPI — the 5 standard
# printable ratios Etsy sellers ship; buyers resize at home from these.
RATIOS = {
    "2x3": (6000, 9000, "2:3 — 12x18, 16x24, 20x30, 24x36 in"),
    "3x4": (5400, 7200, "3:4 — 12x16, 18x24, 30x40 in"),
    "4x5": (4800, 6000, "4:5 — 8x10, 16x20, 24x30 in"),
    "11x14": (3300, 4200, "11:14 — 11x14 in"),
    "iso-a": (4961, 7016, "ISO A — A2/A3/A4/A5 (210x297mm base)"),
}

README_TEMPLATE = """DIGITAL PRODUCT STUDIO — PRINTABLE ART BUNDLE
=============================================

What you purchased
------------------
A {variant_count}-piece printable art set in 5 industry-standard aspect
ratios at 300 DPI (print quality). Print at home, at a local print shop,
or via an online service — on paper, cardstock, canvas or poster stock.

Files
-----
{file_list}

How to print
------------
1. Pick the ratio that matches your frame (or the ISO A file outside the US).
2. Open the file in any photo/viewer software and select "Actual size" /
   100% scale. The files carry 300 DPI metadata so print dialogs size them
   correctly.
3. Use matte or semi-gloss paper 200gsm+ for best results. For large
   formats (24x36in / A2), use a poster printing service.

License
-------
Personal use. You may print unlimited copies for yourself or as gifts.
Reselling, redistribution, or uploading the files elsewhere is not allowed.
Commercial licensing is available separately.

Support
-------
Questions or a file size you need that isn't included? Message the shop —
replacement ratios are provided free of charge.
"""

LICENSE_TXT = """PERSONAL USE LICENSE

This digital download is licensed for personal, non-commercial use by the
purchaser. The purchaser may print unlimited copies for personal display
or gifting. The files may not be resold, shared, redistributed, or used
for commercial purposes (including print-on-demand) without an explicit
commercial license from the seller.
"""


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def load_params(path: str) -> dict:
    with open(path, "r", encoding="utf-8") as fh:
        return json.load(fh)


def render_ratio(img: Image.Image, width: int, height: int) -> Image.Image:
    """Letterbox-fit the source image onto a white canvas of exact px size."""
    canvas = Image.new("RGB", (width, height), BG)
    scale = min(width / img.width, height / img.height)
    new_w = max(1, round(img.width * scale))
    new_h = max(1, round(img.height * scale))
    resized = img.resize((new_w, new_h), Image.Resampling.LANCZOS)
    # Gentle sharpen to recover crispness after large upscale.
    resized = resized.filter(ImageFilter.UnsharpMask(radius=2, percent=60, threshold=2))
    offset = ((width - new_w) // 2, (height - new_h) // 2)
    canvas.paste(resized, offset)
    return canvas


def encode_jpeg(image: Image.Image) -> bytes:
    buf = io.BytesIO()
    image.save(buf, format="JPEG", quality=90, optimize=True, dpi=(DPI, DPI))
    return buf.getvalue()


def build_variant_pack(params: dict, read_root: str) -> dict:
    """Render all ratios for this variant; return {zip_path: bytes}."""
    curated_key = params["curated_key"]
    src_path = os.path.join(read_root, "inputs", curated_key)
    if not os.path.exists(src_path):
        raise FileNotFoundError(f"curated input not staged: {src_path}")

    with Image.open(src_path) as raw:
        img = ImageOps.exif_transpose(raw).convert("RGB")

    variant_id = params["variant_id"]
    design_id = params["design_id"]
    files: dict[str, bytes] = {}
    file_index = []
    for slug, (w, h, label) in RATIOS.items():
        rendered = render_ratio(img, w, h)
        data = encode_jpeg(rendered)
        arc = f"prints/{variant_id}/{slug}.jpg"
        files[arc] = data
        file_index.append(
            {
                "path": arc,
                "ratio": slug,
                "label": label,
                "size_px": [w, h],
                "dpi": DPI,
                "sha256": sha256_bytes(data),
            }
        )

    manifest_entry = {
        "variant_id": variant_id,
        "design_id": design_id,
        "niche_id": params.get("niche_id"),
        "base_prompt": params.get("base_prompt", ""),
        "source_key": curated_key,
        "files": file_index,
    }
    return {"files": files, "manifest_entry": manifest_entry}


def merge_into_bundle(out_path_abs: str, pack: dict, params: dict) -> dict:
    """Append-safe merge: approving more variants grows the set bundle."""
    os.makedirs(os.path.dirname(out_path_abs), exist_ok=True)
    # Read existing manifest if a previous variant already merged.
    existing_manifest = None
    written: list[str] = []

    with tempfile.NamedTemporaryFile(
        prefix="bundle-", suffix=".zip", delete=False
    ) as tmp:
        tmp_path = tmp.name

    if os.path.exists(out_path_abs):
        # Copy original then rewrite: zipfile append mode cannot replace
        # entries, so we rebuild skipping files the new pack provides.
        with zipfile.ZipFile(out_path_abs, "r") as zin, zipfile.ZipFile(
            tmp_path, "w", zipfile.ZIP_DEFLATED
        ) as zout:
            names = zin.namelist()
            if "manifest.json" in names:
                try:
                    existing_manifest = json.loads(
                        zin.read("manifest.json").decode("utf-8")
                    )
                except Exception:
                    existing_manifest = None
            for name in names:
                if name in pack["files"] or name == "manifest.json":
                    continue
                zout.writestr(name, zin.read(name))
    with zipfile.ZipFile(tmp_path, "a", zipfile.ZIP_DEFLATED) as zf:
        for arc, data in pack["files"].items():
            zf.writestr(arc, data)
            written.append(arc)

        # merged manifest
        variants = []
        seen = set()
        if existing_manifest:
            for v in existing_manifest.get("variants", []):
                if v.get("variant_id") != params["variant_id"]:
                    variants.append(v)
                    seen.add(v.get("variant_id"))
        variants.append(pack["manifest_entry"])
        seen.add(params["variant_id"])

        manifest = {
            "product": "printable-art-set",
            "design_id": params["design_id"],
            "niche_id": params.get("niche_id"),
            "variant_count": len(variants),
            "ratios": {k: v[2] for k, v in RATIOS.items()},
            "dpi": DPI,
            "format": "JPEG",
            "generated_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
            "variants": variants,
        }
        zf.writestr("manifest.json", json.dumps(manifest, indent=2))
        zf.writestr("README.txt", README_TEMPLATE.format(
            variant_count=len(variants),
            file_list="\n".join(
                f"- {k}: {v[2]}" for k, v in RATIOS.items()
            ),
        ))
        zf.writestr("LICENSE.txt", LICENSE_TXT)

    os.replace(tmp_path, out_path_abs)
    return {"written": written, "variant_count": len(variants)}


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(prog="product-assembly")
    sub = parser.add_subparsers(dest="cmd", required=True)
    run_p = sub.add_parser("run")
    run_p.add_argument("--params-file", required=True)
    run_p.add_argument("--automation-read-root", required=True)
    run_p.add_argument("--automation-write-root", required=True)
    run_p.add_argument("--out", required=True)
    args, _unknown = parser.parse_known_args(argv)

    params = load_params(args.params_file)
    pack = build_variant_pack(params, args.automation_read_root)
    out_abs = os.path.join(args.automation_write_root, args.out)
    result = merge_into_bundle(out_abs, pack, params)

    print(
        json.dumps(
            {
                "created": True,
                "design_id": params["design_id"],
                "variant_id": params["variant_id"],
                "bundle_variant_count": result["variant_count"],
                "files": result["written"],
                "outputs": [args.out],
            }
        )
    )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
