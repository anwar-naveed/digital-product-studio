#!/usr/bin/env python3
"""cf-image-gen sandbox entrypoint.

Speaks the pipeline's tool-job contract (worker/src/exec.rs):
    cf-image-gen run --params-file P --automation-read-root R \
                     --automation-write-root W [--out F]...

Reads `prompt` (plus optional model/size/n) from the params JSON,
generates via the Cloudflare Workers AI bridge, and copies the result
into the write root at the --out path (or a default name).
"""
import argparse
import json
import os
import shutil
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import cf_image_client  # noqa: E402


def main() -> int:
    # The exec.rs harness prepends the tool name as the first positional
    # (argv contract shared with photocraft/vectorcraft): normalize it away
    # before argparse — accepts both `cf-image-gen run ...` and `run ...`.
    argv = sys.argv[1:]
    subcommand = "run"
    if argv and argv[0] == "cf-image-gen":
        argv.pop(0)
    if argv and argv[0] in ("run", "health"):
        subcommand = argv.pop(0)

    ap = argparse.ArgumentParser(prog="cf-image-gen")
    ap.add_argument("--params-file", required=False)
    ap.add_argument("--automation-read-root", default="/workspace/read")
    ap.add_argument("--automation-write-root", default="/workspace/write")
    ap.add_argument("--out", action="append", default=[])
    args = ap.parse_args(argv)

    if subcommand == "health":
        print(json.dumps({"ok": True}))
        return 0

    if not args.params_file:
        print("params-file is required for run", file=sys.stderr)
        return 2

    with open(args.params_file) as fh:
        params = json.load(fh)

    prompt = params.get("prompt")
    if not prompt:
        print("params JSON is missing 'prompt'", file=sys.stderr)
        return 2

    out_dir = args.automation_write_root
    os.makedirs(out_dir, exist_ok=True)

    gen_kwargs = {}
    if params.get("model"):
        gen_kwargs["model"] = params["model"]
    if params.get("size"):
        gen_kwargs["size"] = params["size"]
    if params.get("n"):
        gen_kwargs["n"] = int(params["n"])

    paths = cf_image_client.generate(prompt, out_dir=out_dir, prefix="img", **gen_kwargs)
    if not paths:
        print("generation returned no files", file=sys.stderr)
        return 3

    # Map generated files onto the expected --out targets (relative paths
    # under the write root).
    for i, out_path in enumerate(args.out):
        if i >= len(paths):
            break
        target = os.path.join(out_dir, out_path)
        os.makedirs(os.path.dirname(target), exist_ok=True)
        shutil.copyfile(paths[i], target)

    print(json.dumps({"created": True, "files": paths, "outputs": args.out}))
    return 0


if __name__ == "__main__":
    sys.exit(main())
