import pty, os, select, subprocess

rom = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba"

master, slave = pty.openpty()
proc = subprocess.Popen(
    ["/usr/games/mgba", "-d", rom],
    stdin=slave, stdout=slave, stderr=slave,
    env={**os.environ, "SDL_VIDEODRIVER": "dummy", "SDL_AUDIODRIVER": "dummy"},
    close_fds=True
)
os.close(slave)

def read_output(fd, timeout=0.5):
    data = b""
    while True:
        r, _, _ = select.select([fd], [], [], timeout)
        if r:
            try:
                chunk = os.read(fd, 4096)
                data += chunk
            except:
                break
        else:
            break
    return data

# Wait for startup  
out = read_output(master, 2.0)
# Consume prompt
read_output(master, 0.2)

# Run 600 frames to reach title
for i in range(600):
    os.write(master, b"frame\n")
    read_output(master, 0.02)

read_output(master, 0.3)

# Dump EWRAM first 256 bytes
print("=== mGBA EWRAM at frame 600 ===")
for offset in range(0, 0x100, 4):
    addr = 0x02000000 + offset
    cmd = f"r/4 0x{addr:08X}\n".encode()
    os.write(master, cmd)
    out = read_output(master, 0.1).decode('utf-8', errors='replace')
    val = out.strip().split('\n')[-1].strip()
    print(f"  [{addr:08X}] = {val}")

# Also dump key IWRAM addresses
print("\n=== mGBA IWRAM key addresses ===")
for offset in [0x7FF8, 0x7FFC]:
    addr = 0x03000000 + offset
    cmd = f"r/4 0x{addr:08X}\n".encode()
    os.write(master, cmd)
    out = read_output(master, 0.1).decode('utf-8', errors='replace')
    val = out.strip().split('\n')[-1].strip()
    print(f"  [{addr:08X}] = {val}")

# Check PC
os.write(master, b"r/4 15\n")
out = read_output(master, 0.1).decode('utf-8', errors='replace')
print(f"\nPC: {out.strip().split(chr(10))[-1].strip()}")

proc.terminate()
