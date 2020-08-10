#!/usr/bin/env python
from pathlib import Path
import json, subprocess, os, shutil, sys

def err(msg):
    print(f"[!] {msg}")
    exit(1)
def list_boards():
    bsp = Path("src/bsp")
    if not bsp.is_dir():
        err("bsp directory does not exist!")
    print("=== Board listing ===")
    for (i, thing) in enumerate(bsp.iterdir(), start=1):
        if thing.is_dir():
            print(f"{i}. {thing.name}")

def build(board):
    bsp = Path("src/bsp")
    build_json = bsp / board / "build.json"
    link = bsp / board / "link.ld"
    if not build_json.is_file() or not link.is_file():
        err(f"incomplete board definition for `{board}`")
    bf = open(build_json)
    build = json.load(bf)
    bf.close()
    if build.get("name") != board:
        err("wrong board")
    target = build.get("target")
    features = build.get("features")
    rustflags = build.get("rustflags")
    rustflags.append(f"-C link-arg=-T{link}")
    rustflags = " ".join(rustflags)
    if "RUSTFLAGS" in os.environ:
        rustflags = f"{os.environ['RUSTFLAGS']} {rustflags}"
    command = [shutil.which("cargo"), "rustc", f"--target={target}", "--release"]
    for feature in features:
        command.extend(["--features", f"{feature}"])
    print(command)
    print(rustflags)
    subprocess.Popen(command, env={"RUSTFLAGS": rustflags})
def usage():
    err(f"Usage: {sys.argv[0]} build <board name>\n[!] Or {sys.argv[0]} list-boards")
def main():
    if len(sys.argv) < 2:
        usage()
        return
    if sys.argv[1] == "list-boards":
        list_boards()
        return
    elif sys.argv[1] == "build" and len(sys.argv) == 3:
        build(sys.argv[2])
    else:
        usage()
        return
main()
