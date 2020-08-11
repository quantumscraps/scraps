#!/usr/bin/env python3
from pathlib import Path
import json, subprocess, os, shutil, sys

NAME = "scraps"

def err(msgs):
    for msg in msgs.split("\n"):
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
    command = ["cargo", "rustc", f"--target={target}", "--release"]
    for feature in features:
        command.extend(["--features", f"{feature}"])
    print(f"executing: RUSTFLAGS={rustflags} {' '.join(command)}")
    command.extend(["--color", "always"])
    p = subprocess.Popen(command, env={"RUSTFLAGS": rustflags, "PATH": os.environ["PATH"]})
    p.communicate()
    if p.returncode == 0:
        print(":) success")
    else:
        print(":( failure")
def run(board):
    bsp = Path("src/bsp")
    build_json = bsp / board / "build.json"
    bf = open(build_json)
    build_dict = json.load(bf)
    bf.close()
    runcmd = build_dict.get("runcmd")
    target = build_dict.get("target")
    runcmd.append(f"target/{target}/release/{NAME}")
    print(f"Running {' '.join(runcmd)}")
    try:
        subprocess.check_call(runcmd, env=os.environ)
    except KeyboardInterrupt:
        print("Exited on interrupt (^C)")
def clean():
    pth = Path("target")
    if pth.exists():
        print("cleaning...")
        shutil.rmtree("target")
    else:
        print("nothing to do")
def usage():
    err(f"Usage: {sys.argv[0]} build <board name>\nOr {sys.argv[0]} run <board name>\nOr {sys.argv[0]} list-boards\nOr {sys.argv[0]} clean")
def main():
    if len(sys.argv) < 2:
        usage()
        return
    if sys.argv[1] == "list-boards":
        list_boards()
        return
    elif sys.argv[1] == "build" and len(sys.argv) == 3:
        build(sys.argv[2])
    elif sys.argv[1] == "run" and len(sys.argv) == 3:
        run(sys.argv[2])
    elif sys.argv[1] == "clean":
        clean()
        return
    else:
        usage()
        return
main()
