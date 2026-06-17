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


print("=== mGBA Init Sequence ===")
current = 0
prev_dispcnt = 0
prev_state = 0

for target in (
    list(range(5, 50, 5)) + list(range(50, 250, 10)) + list(range(250, 800, 50))
):
    while current < target:
        raw_cmd("frame")
        current += 1

    dispcnt = read_half(0x04000000)
    state = read_word(0x02000074)

    if dispcnt != prev_dispcnt or state != prev_state:
        print(
            f"Frame {current:4}: DISPCNT=0x{dispcnt:04X} State=0x{state:08X}"
            if dispcnt is not None and state is not None
            else f"Frame {current:4}: read error"
        )
        prev_dispcnt = dispcnt
        prev_state = state

# Also check specific frames
for t in [500, 600, 650, 700, 750]:
    while current < t:
        raw_cmd("frame")
        current += 1
    state = read_word(0x02000074)
    if state != prev_state:
        print(f"Frame {current:4}: State=0x{state:08X}")
        prev_state = state

proc.terminate()
