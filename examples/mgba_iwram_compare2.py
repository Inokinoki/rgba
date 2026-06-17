#!/usr/bin/env python3
"""Compare IWRAM decompression output between our emulator and mGBA."""

import pty
import os
import select
import time
import struct


def read_until_prompt(master, timeout=5):
    data = b""
    start = time.time()
    while time.time() - start < timeout:
        r, _, _ = select.select([master], [], [], 0.1)
        if r:
            try:
                chunk = os.read(master, 4096)
                data += chunk
                if b">" in data[-10:]:
                    break
            except:
                break
    return data


def send_cmd(master, cmd):
    os.write(master, (cmd + "\n").encode())
    time.sleep(0.1)
    return read_until_prompt(master)


def main():
    master, slave = pty.openpty()
    pid = os.fork()
    if pid == 0:
        os.close(master)
        os.setsid()
        os.dup2(slave, 0)
        os.dup2(slave, 1)
        os.dup2(slave, 2)
        os.close(slave)
        os.execvp(
            "/usr/games/mgba",
            ["mgba", "-d", "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba"],
        )

    os.close(slave)
    time.sleep(2)

    # Read initial output
    read_until_prompt(master, timeout=3)

    # Run to a point where the game has loaded (after 300 frames)
    # We need to let the game boot and reach the farm scene
    # Let's run for many frames
    for _ in range(300):
        send_cmd(master, "c")
        time.sleep(0.001)

    # Now read IWRAM at 0x03006DD8 (64 halfwords = 128 bytes)
    print("=== mGBA IWRAM at 0x03006DD8 ===")
    for i in range(64):
        addr = 0x03006DD8 + i * 2
        result = send_cmd(master, f"r/2 {addr:#010x}")
        # Parse the halfword from output
        lines = result.decode("utf-8", errors="replace").strip().split("\n")
        for line in lines:
            if ":" in line and ("0x" in line.lower()):
                # Try to extract the hex value
                parts = line.split(":")
                if len(parts) >= 2:
                    val_str = parts[-1].strip().split()[0]
                    try:
                        val = int(val_str, 16)
                        tile = val & 0x3FF
                        pal = (val >> 12) & 0xF
                        print(
                            f"  [{i:2}] {addr:#010x}: {val:#06x} (tile={tile} pal={pal})"
                        )
                    except:
                        print(
                            f"  [{i:2}] {addr:#010x}: PARSE ERROR from: {line.strip()}"
                        )
                break

    os.write(master, b"q\n")
    time.sleep(0.5)
    os.close(master)
    os.waitpid(pid, 0)


if __name__ == "__main__":
    main()
