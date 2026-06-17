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

# Wait for initial prompt
out = read_until(master, b"> ", 5.0)
print("INIT:", out.decode('utf-8', errors='replace')[-200:])

# Run frames to title screen
for i in range(600):
    os.write(master, b"frame\n")
    read_until(master, b"> ", 0.05)

# Now read memory
def read_mem(addr):
    os.write(master, f"r/4 0x{addr:08X}\n".encode())
    out = read_until(master, b"> ", 1.0).decode('utf-8', errors='replace')
    # Parse the output - look for hex value
    for line in out.split('\n'):
        line = line.strip()
        if line.startswith('0x') and not line.startswith('0x04') and ':' not in line:
            return line
    return out.strip()

# Read EWRAM
print("\n=== mGBA EWRAM ===")
for offset in range(0, 0x100, 4):
    addr = 0x02000000 + offset
    val = read_mem(addr)
    print(f"  [{addr:08X}] = {val}")

proc.terminate()
