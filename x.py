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
    for (i, thing) in enumerate(filter(lambda x: x.is_dir(), bsp.iterdir()), start=1):
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
        return True
    else:
        print(":( failure")
        return False
def run(board):
    print(f"Building for {board}")
    if not build(board):
        return
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
        subprocess.check_call(runcmd)
    except KeyboardInterrupt:
        print("Exited on interrupt (^C)")
def generate_vscode(board):
    print(f"Generating vscode settings for {board}")
    bsp = Path("src/bsp")
    build_json = bsp / board / "build.json"
    with open(build_json) as bf:
        build_dict = json.load(bf)
    target = build_dict.get("target")
    features = build_dict.get("features")
    settings = {
        "rust-analyzer.cargo.features": features,
        "rust-analyzer.cargo.target": target,
        "rust-analyzer.checkOnSave.allTargets": False,
        "rust-analyzer.checkOnSave.extraArgs": [
            "--target",
            target,
        ],
    }
    vspath = Path(".vscode")
    vspath.mkdir(parents=True, exist_ok=True)
    with open(".vscode/settings.json", "w") as f:
        json.dump(settings, f)
    print(f"Written settings to .vscode/settings.json")
def clean():
    pth = Path("target")
    if pth.exists():
        print("cleaning...")
        shutil.rmtree("target")
    else:
        print("nothing to do")
def usage():
    err(f"""
Usage: {sys.argv[0]} build <board name>
Or {sys.argv[0]} run <board name>
Or {sys.argv[0]} list-boards
Or {sys.argv[0]} clean
Or {sys.argv[0]} generate-vscode <board name>
""")
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
    elif sys.argv[1] == "generate-vscode" and len(sys.argv) == 3:
        generate_vscode(sys.argv[2])
        return
    elif sys.argv[1] == "clean":
        clean()
        return
    else:
        usage()
        return
main()
