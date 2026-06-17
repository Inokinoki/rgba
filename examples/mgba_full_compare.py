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
read_until(master, b"> ", 5.0)

# Run 600 frames
for i in range(600):
    os.write(master, b"frame\n")
    read_until(master, b"> ", 0.05)

# Read EWRAM
def read_mem_word(addr):
    os.write(master, f"r/4 0x{addr:08X}\n".encode())
    out = read_until(master, b"> ", 1.0).decode('utf-8', errors='replace')
    for line in out.split('\n'):
        line = line.strip()
        if line.startswith('0x') and ':' not in line:
            return int(line, 16)
    return None

print("=== mGBA frame 600 ===")
# Check key addresses that differed
key_addrs = [
    0x02000000, 0x02000004, 0x02000040, 0x02000044,
    0x02000050, 0x02000054, 0x02000060, 0x02000064,
    0x02000074, 0x0200007C, 0x020000A0, 0x020000C0,
    0x020000E0, 0x020000E4, 0x020000F0, 0x020000F4,
]
for addr in key_addrs:
    val = read_mem_word(addr)
    if val is not None:
        print(f"  [{addr:08X}] = 0x{val:08X}")
    else:
        print(f"  [{addr:08X}] = ???")

# Now press START + A and check what happens  
print("\n=== mGBA: pressing START ===")
os.write(master, b"key 3\n")  # START = button 3
for i in range(250):
    os.write(master, b"frame\n")
    read_until(master, b"> ", 0.02)
os.write(master, b"unkey 3\n")
read_until(master, b"> ", 0.3)

dispcnt = read_mem_word(0x04000000)
print(f"After START: DISPCNT = 0x{dispcnt:04X}" if dispcnt else "After START: DISPCNT = ???")

# Check key addresses again
for addr in [0x02000000, 0x02000074, 0x0200007C, 0x020000C0]:
    val = read_mem_word(addr)
    if val is not None:
        print(f"  [{addr:08X}] = 0x{val:08X}")

print("\n=== mGBA: pressing A ===")
os.write(master, b"key 0\n")  # A = button 0
for i in range(250):
    os.write(master, b"frame\n")
    read_until(master, b"> ", 0.02)
os.write(master, b"unkey 0\n")
read_until(master, b"> ", 0.3)

dispcnt = read_mem_word(0x04000000)
print(f"After A: DISPCNT = 0x{dispcnt:04X}" if dispcnt else "After A: DISPCNT = ???")

# Check EWRAM
for addr in key_addrs:
    val = read_mem_word(addr)
    if val is not None:
        print(f"  [{addr:08X}] = 0x{val:08X}")

proc.terminate()
