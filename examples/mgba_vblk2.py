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

def r4(addr):
    os.write(master, f"r/4 0x{addr:08X}\n".encode())
    out = read_until(master, b"> ", 1.0).decode('utf-8', errors='replace')
    for line in out.split('\n'):
        line = line.strip()
        if line.startswith('0x') and ':' not in line and len(line) <= 12:
            return int(line, 16)
    return None

for target in [100, 200, 600, 700]:
    for i in range(target):
        os.write(master, b"frame\n")
        read_until(master, b"> ", 0.02)
    vblk = r4(0x03007FF8)
    if vblk is not None:
        print(f"mGBA Frame {target}: VBLK={vblk:08X} (ratio={vblk/target:.2f})")
    else:
        print(f"mGBA Frame {target}: VBLK read failed")
    break  # Can only run to one target per session

proc.terminate()
