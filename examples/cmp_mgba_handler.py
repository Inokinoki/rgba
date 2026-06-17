#!/usr/bin/env python3
"""Read game IRQ handler code from mGBA"""

import subprocess, time, select, os, pty, re, struct

ROM = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba"
env = os.environ.copy()
env["SDL_VIDEODRIVER"] = "dummy"
env["SDL_AUDIODRIVER"] = "dummy"
env["PATH"] = "/usr/games:" + env.get("PATH", "")

master_fd, slave_fd = pty.openpty()
proc = subprocess.Popen(
    ["mgba", "-d", ROM],
    stdin=subprocess.PIPE,
    stdout=slave_fd,
    stderr=slave_fd,
    env=env,
)
os.close(slave_fd)


def read_pty(timeout=0.2):
    buf = b""
    while select.select([master_fd], [], [], timeout)[0]:
        try:
            buf += os.read(master_fd, 65536)
        except:
            break
    return buf.decode(errors="replace")


def read_word(addr):
    proc.stdin.write(f"r/4 0x{addr:08X}\n".encode())
    proc.stdin.flush()
    time.sleep(0.02)
    out = read_pty(0.05)
    m = re.search(r"0x([0-9A-Fa-f]{8})", out)
    return int(m.group(1), 16) if m else None


time.sleep(1)
read_pty(0.5)
proc.stdin.write(b"c\n")
proc.stdin.flush()
time.sleep(5)
read_pty(1)
import signal

proc.send_signal(signal.SIGINT)
time.sleep(0.5)
read_pty(0.5)

# Read IRQ handler address from 0x03007FFC
handler = read_word(0x03007FFC)
print(f"IRQ handler address: 0x{handler:08X}")

# Read some instructions at the handler address (THUMB mode if bit 0 is set)
if handler & 1:
    thumb_addr = handler & ~1
    print(f"THUMB mode, reading from 0x{thumb_addr:08X}")
    for i in range(32):
        addr = thumb_addr + i * 2
        proc.stdin.write(f"r/2 0x{addr:08X}\n".encode())
        proc.stdin.flush()
        time.sleep(0.02)
        out = read_pty(0.05)
        m = re.search(r"0x([0-9A-Fa-f]{4})", out)
        hw = int(m.group(1), 16) & 0xFFFF if m else 0
        print(f"  0x{addr:08X}: 0x{hw:04X}")
else:
    print(f"ARM mode")
    for i in range(16):
        addr = handler + i * 4
        w = read_word(addr)
        print(f"  0x{addr:08X}: 0x{w:08X}")

proc.stdin.write(b"quit\n")
proc.stdin.flush()
try:
    proc.wait(timeout=5)
except:
    proc.kill()
os.close(master_fd)
