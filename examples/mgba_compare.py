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


# Advance to frame 100
for _ in range(100):
    raw_cmd("frame")

print("=== mGBA Frame 100 ===")
vblk = read_word(0x03007FF8)
print(f"VBLK (03007FF8): 0x{vblk:08X}" if vblk is not None else "VBLK: None")
handler = read_word(0x03007FFC)
print(
    f"Handler (03007FFC): 0x{handler:08X}" if handler is not None else "Handler: None"
)

# EWRAM first 256 bytes
print("\nEWRAM [0x02000000 - 0x02000100]:")
for off in range(0, 0x100, 4):
    addr = 0x02000000 + off
    v = read_word(addr)
    if v is not None and v != 0:
        print(f"  [{off:04X}] = 0x{v:08X}")

# IO registers
print("\nIO registers:")
for name, addr in [
    ("DISPCNT", 0x04000000),
    ("DISPSTAT", 0x04000004),
    ("VCOUNT", 0x04000006),
    ("IE", 0x04000200),
    ("IF", 0x04000202),
    ("IME", 0x04000208),
    ("TM0CNT_L", 0x04000100),
    ("TM0CNT_H", 0x04000102),
    ("TM1CNT_L", 0x04000104),
    ("TM1CNT_H", 0x04000106),
    ("TM2CNT_L", 0x04000108),
    ("TM2CNT_H", 0x0400010A),
    ("TM3CNT_L", 0x0400010C),
    ("TM3CNT_H", 0x0400010E),
]:
    v = read_half(addr)
    print(f"  {name} = 0x{v:04X}" if v is not None else f"  {name} = None")

# Key game state addresses from previous analysis
print("\nGame state:")
for name, addr in [
    ("State", 0x02000074),
    ("State2", 0x0200007C),
    ("Counter", 0x02000064),
    ("Timer", 0x02000060),
]:
    v = read_word(addr)
    print(f"  {name} [{addr:08X}] = 0x{v:08X}" if v is not None else f"  {name} = None")

proc.terminate()
