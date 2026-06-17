import pty, os, select, subprocess, time

rom = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba"
master, slave = pty.openpty()
proc = subprocess.Popen(
    ["/usr/games/mgba", "-d", rom],
    stdin=slave, stdout=slave, stderr=slave,
    env={**os.environ, "SDL_VIDEODRIVER": "dummy", "SDL_AUDIODRIVER": "dummy"},
    close_fds=True
)
os.close(slave)

def read_until(fd, pattern=b"> ", timeout=3.0):
    data = b""
    start = time.time()
    while time.time() - start < timeout:
        r, _, _ = select.select([fd], [], [], 0.1)
        if r:
            try:
                chunk = os.read(fd, 4096)
                data += chunk
                if pattern in data:
                    return data
            except:
                break
    return data

read_until(master, b"> ", 5.0)

# Run to frame 600
for i in range(600):
    os.write(master, b"frame\n")
    read_until(master, b"> ", 0.03)

# Check key values
def r4(addr):
    os.write(master, f"r/4 0x{addr:08X}\n".encode())
    out = read_until(master, b"> ", 1.0).decode('utf-8', errors='replace')
    for line in out.split('\n'):
        line = line.strip()
        if line.startswith('0x') and ':' not in line and len(line) <= 12:
            return int(line, 16)
    return None

print("=== mGBA Frame 600 ===")
addrs = {
    0x02000074: "state1",
    0x0200007C: "state2", 
    0x02000060: "float_val",
    0x02000064: "counter",
    0x020000C0: "ptr",
    0x020000F0: "val_f0",
}
for addr, name in sorted(addrs.items()):
    v = r4(addr)
    print(f"  [{addr:08X}] {name:12s} = 0x{v:08X}" if v is not None else f"  [{addr:08X}] {name:12s} = FAILED")

# Now continue to frame 700 (without any input)
for i in range(100):
    os.write(master, b"frame\n")
    read_until(master, b"> ", 0.03)

print("\n=== mGBA Frame 700 ===")
for addr, name in sorted(addrs.items()):
    v = r4(addr)
    print(f"  [{addr:08X}] {name:12s} = 0x{v:08X}" if v is not None else f"  [{addr:08X}] {name:12s} = FAILED")

proc.terminate()
