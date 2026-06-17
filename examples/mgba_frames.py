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

# Run to frame 560
for i in range(560):
    os.write(master, b"frame\n")
    read_until(master, b"> ", 0.02)

# Now check frames 560-575
def read_mem_word(addr):
    os.write(master, f"r/4 0x{addr:08X}\n".encode())
    out = read_until(master, b"> ", 1.0).decode('utf-8', errors='replace')
    for line in out.split('\n'):
        line = line.strip()
        if line.startswith('0x') and ':' not in line:
            return int(line, 16)
    return None

for frame in range(560, 580):
    os.write(master, b"frame\n")
    read_until(master, b"> ", 0.05)
    
    val74 = read_mem_word(0x02000074)
    val7c = read_mem_word(0x0200007C)
    val60 = read_mem_word(0x02000060)
    val64 = read_mem_word(0x02000064)
    
    print(f"Frame {frame}: [0074]={val74:08X} [007C]={val7c:08X} [0060]={val60:08X} [0064]={val64:08X}" if all(v is not None for v in [val74, val7c, val60, val64]) else f"Frame {frame}: read failed")

proc.terminate()
