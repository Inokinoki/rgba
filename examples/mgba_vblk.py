import pty, os, select, subprocess, time

rom = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba"

master, slave = pty.openpty()
proc = subprocess.Popen(
    ["/usr/games/mgba", "-d", rom],
    stdin=slave,
    stdout=slave,
    stderr=slave,
    env={**os.environ, "SDL_VIDEODRIVER": "dummy", "SDL_AUDIODRIVER": "dummy"},
    close_fds=True,
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


out = read_until(master, b"> ", 5.0)


def raw_cmd(cmd):
    os.write(master, f"{cmd}\n".encode())
    out = read_until(master, b"> ", 2.0).decode("utf-8", errors="replace")
    return out


# Read at frame 0
v = raw_cmd("r/4 0x03007FF8")
print(f"Frame 0: {v.strip()}")

# Advance 5 frames
for _ in range(5):
    raw_cmd("frame")
v = raw_cmd("r/4 0x03007FF8")
print(f"Frame 5: {v.strip()}")

# Advance 5 more
for _ in range(5):
    raw_cmd("frame")
v = raw_cmd("r/4 0x03007FF8")
print(f"Frame 10: {v.strip()}")

# Try byte reads
v = raw_cmd("r/1 0x03007FF8")
print(f"Byte FF8: {v.strip()}")
v = raw_cmd("r/1 0x03007FF9")
print(f"Byte FF9: {v.strip()}")
v = raw_cmd("r/1 0x03007FFA")
print(f"Byte FFA: {v.strip()}")
v = raw_cmd("r/1 0x03007FFB")
print(f"Byte FFB: {v.strip()}")

# Check handler pointer
v = raw_cmd("r/4 0x03007FFC")
print(f"Handler ptr: {v.strip()}")

# Read EWRAM to verify mGBA is working
v = raw_cmd("r/4 0x02000074")
print(f"EWRAM 74: {v.strip()}")

proc.terminate()
