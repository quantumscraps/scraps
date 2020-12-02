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

def build(board, debug=False):
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
    command = ["cargo", "rustc", f"--target={target}"]
    if not debug:
        command.append("--release")
    for feature in features:
        command.extend(["--features", f"{feature}"])
    print(f"executing: RUSTFLAGS=\"{rustflags}\" {' '.join(command)}")
    command.extend(["--color", "always"])
    e = {"RUSTFLAGS": rustflags}
    evars = ["PATH", "TMP", "TEMP", "SYSTEMROOT"]
    for evar in evars:
        if evar in os.environ:
            e[evar] = os.environ[evar]
    p = subprocess.Popen(command, env=e)
    p.communicate()
    if p.returncode == 0:
        print(":) success")
        return True
    else:
        print(":( failure")
        return False
def binary(board):
    print(f"Building for {board}")
    if not build(board):
        return
    bsp = Path("src/bsp")
    build_json = bsp / board / "build.json"
    bf = open(build_json)
    build_dict = json.load(bf)
    bf.close()
    kernel_name = build_dict.get("kernel_name")
    target = build_dict.get("target")
    executable = Path("target") / target / "release" / NAME;
    kernels = Path("obj")
    if not kernels.exists():
        print("Creating `obj`")
        os.mkdir("obj")
    
    cmd = ["rust-objcopy", "-Obinary", str(executable), str(kernels / kernel_name)]
    print(f"executing: {' '.join(cmd)}")
    p = subprocess.Popen(cmd)
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
def debug(board):
    print(f"Debugging {board}")
    if not build(board, debug=True):
        return
    bsp = Path("src/bsp")
    build_json = bsp / board / "build.json"
    bf = open(build_json)
    build_dict = json.load(bf)
    bf.close()
    runcmd = build_dict.get("runcmd")
    target = build_dict.get("target")
    runcmd.append(f"target/{target}/debug/{NAME}")
    runcmd.extend(["-s", "-S"])
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
        "rust-analyzer.cargo.noDefaultFeatures": True,
        "rust-analyzer.checkOnSave.allTargets": False,
        "rust-analyzer.cargo.target": target,
        "rust-analyzer.cargo.allFeatures": False,
        "rust-analyzer.diagnostics.disabled": [
            # required until rust-analyzer implements support for
            # #![feature(break_label_value)]
            "break-outside-of-loop"
        ]
    }
    vspath = Path(".vscode")
    vspath.mkdir(parents=True, exist_ok=True)
    with open(".vscode/settings.json", "w") as f:
        json.dump(settings, f, indent=4)
    print(f"Written settings to .vscode/settings.json")
def clean():
    target = Path("target")
    kernels = Path("obj")
    cleaned = False
    if target.exists():
        print("cleaning target...")
        shutil.rmtree("target")
        cleaned = True
    if kernels.exists():
        print("cleaning obj...")
        shutil.rmtree("obj")
        cleaned = True
    if not cleaned:
        print("nothing to do")
def usage():
    err(f"""
Usage: {sys.argv[0]} build <board name>
Or {sys.argv[0]} run <board name>
Or {sys.argv[0]} debug <board name> (starts gdbserver on localhost:1234)
Or {sys.argv[0]} list-boards
Or {sys.argv[0]} clean
Or {sys.argv[0]} generate-vscode <board name>
""")
def main():
    if len(sys.argv) < 2:
        usage()
    if sys.argv[1] == "list-boards":
        list_boards()
    elif sys.argv[1] == "build" and len(sys.argv) == 3:
        build(sys.argv[2])
    elif sys.argv[1] == "binary" and len(sys.argv) == 3:
        binary(sys.argv[2])
    elif sys.argv[1] == "run" and len(sys.argv) == 3:
        run(sys.argv[2])
    elif sys.argv[1] == "debug" and len(sys.argv) == 3:
        debug(sys.argv[2])
    elif sys.argv[1] == "generate-vscode" and len(sys.argv) == 3:
        generate_vscode(sys.argv[2])
    elif sys.argv[1] == "clean":
        clean()
    else:
        usage()
main()
