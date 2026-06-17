#!/usr/bin/env python3
"""Compare register + memory state between mGBA and RGBA at 5th VBlank handler entry."""
import subprocess, os, time, struct

ROM = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba"
RGBA_DIR = "/home/ubuntu/Builds/RGBA"

os.system("pkill -f Xvfb 2>/dev/null; sleep 0.3")
os.system("Xvfb :99 -screen 0 640x480x24 &")
time.sleep(0.5)

# mGBA: break at IRQ handler entry 5 times (5 VBlanks), dump state each time
# Use r/N addr (read N bytes) and dis N for disassembly
script = """b 0x03000958
c
status
r/4 0x02008D2C
r/4 0x04000000
r/4 0x03000410
r/4 0x03007FF8
r/2 0x04000200
r/2 0x04000202
r/2 0x04000208
c
status
r/4 0x02008D2C
r/4 0x04000000
r/4 0x03000410
r/4 0x03007FF8
c
status
r/4 0x02008D2C
r/4 0x04000000
r/4 0x03007FF8
c
status
r/4 0x02008D2C
r/4 0x04000000
r/4 0x03007FF8
c
status
r/4 0x02008D2C
r/4 0x04000000
r/4 0x03007FF8
r/4 0x02009208
r/4 0x020091EC
r/4 0x020091F0
r/2 0x04000006
r/2 0x04000008
r/2 0x0400000A
r/2 0x0400000C
r/2 0x04000200
r/2 0x04000202
r/2 0x04000208
r/2 0x040000E0
r/2 0x040000E2
q
"""

with open("/tmp/mgba_cmp2.txt", "w") as f:
    f.write(script)

env = os.environ.copy()
env["DISPLAY"] = ":99"

proc = subprocess.run(
    ["/usr/games/mgba", "-d", ROM],
    input=script, capture_output=True, text=True, timeout=30, env=env
)

print(proc.stdout)
if proc.stderr:
    print("STDERR:", proc.stderr[-200:])
