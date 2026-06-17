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


read_until(master, b"> ", 5.0)


def raw_cmd(cmd):
    os.write(master, f"{cmd}\n".encode())
    return read_until(master, b"> ", 2.0).decode("utf-8", errors="replace")


def read_half(addr):
    out = raw_cmd(f"r/2 0x{addr:08X}")
    for line in out.split("\n"):
        line = line.strip()
        if line.startswith("0x") and ":" not in line:
            try:
                return int(line, 16)
            except:
                pass
    return None


def read_word(addr):
    out = raw_cmd(f"r/4 0x{addr:08X}")
    for line in out.split("\n"):
        line = line.strip()
        if line.startswith("0x") and ":" not in line:
            try:
                return int(line, 16)
            except:
                pass
    return None


# Advance to frame 400 (title screen)
for _ in range(400):
    raw_cmd("frame")

print("=== mGBA Frame 400 (title screen) ===")
for name, addr in [
    ("KEYINPUT (04000130)", 0x04000130),
    ("KEYCNT (04000132)", 0x04000132),
    ("DISPCNT", 0x04000000),
    ("IE", 0x04000200),
    ("IF", 0x04000202),
    ("IME", 0x04000208),
    ("TM0CNT_L", 0x04000100),
    ("TM0CNT_H", 0x04000102),
]:
    v = read_half(addr)
    print(f"  {name} = 0x{v:04X}" if v is not None else f"  {name} = None")

print(f"\n  State (02000074) = 0x{read_word(0x02000074):08X}")
print(f"  VBLK (03007FF8) = 0x{read_word(0x03007FF8):08X}")

proc.terminate()
