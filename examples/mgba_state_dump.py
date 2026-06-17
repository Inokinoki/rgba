#!/usr/bin/env python3
"""Dump mGBA EWRAM/IWRAM at the same execution point as our emulator."""
import subprocess, os, time, signal

ROM = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba"

# Start Xvfb if not running
os.system("pkill -f Xvfb 2>/dev/null; sleep 0.5")
os.system("Xvfb :99 -screen 0 640x480x24 &")
time.sleep(1)

# Run mGBA with debugger to dump memory
# We'll break at the VBlank handler (0x03000958) and dump state
script = """b 0x03000958
c
c
c
c
c
c
c
c
c
c
c
c
r/4 0x02008D2C
x/4w 0x02008D2C
x/4w 0x020091EC
x/4w 0x02009208
x/4w 0x03007FF8
x/2w 0x03007FFC
r/4 0x03000410
x/2w 0x03000410
x/2w 0x03000450
dis 30
status
q
"""

with open("/tmp/mgba_cmp_script.txt", "w") as f:
    f.write(script)

env = os.environ.copy()
env["DISPLAY"] = ":99"

proc = subprocess.run(
    ["/usr/games/mgba", "-d", ROM],
    input=script, capture_output=True, text=True, timeout=60, env=env
)

print("=== mGBA stdout ===")
print(proc.stdout[-3000:] if len(proc.stdout) > 3000 else proc.stdout)
print("\n=== mGBA stderr ===")
print(proc.stderr[-500:] if proc.stderr else "(none)")
