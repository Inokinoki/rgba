#!/usr/bin/env python3
"""
Dump IWRAM from mGBA at multiple frame counts to find when data appears.
"""

import pty, os, select, time, subprocess

ROM = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba"
IWSTART = 0x03006DD8
NENTRIES = 512


def ru(fd, marker, timeout=30):
    buf = b""
    t0 = time.time()
    while time.time() - t0 < timeout:
        r, _, _ = select.select([fd], [], [], 0.05)
        if r:
            try:
                d = os.read(fd, 4096)
                if not d:
                    break
                buf += d
                if marker in buf:
                    return buf
            except:
                break
    return buf


def sc(fd, cmd):
    os.write(fd, (cmd + "\n").encode())


def read_hw(fd, addr):
    sc(fd, f"r/2 0x{addr:08X}")
    buf = ru(fd, b">", timeout=3)
    text = buf.decode(errors="replace")
    for line in text.split("\n"):
        line = line.strip()
        if "=" in line:
            parts = line.split("=")
            if len(parts) >= 2:
                try:
                    return int(parts[-1].strip(), 16)
                except:
                    pass
    return 0xFFFF


def main():
    mf, sf = pty.openpty()
    env = os.environ.copy()
    env["SDL_VIDEODRIVER"] = "dummy"
    env["SDL_AUDIODRIVER"] = "dummy"
    proc = subprocess.Popen(
        ["/usr/games/mgba", "-d", ROM], stdin=sf, stdout=sf, stderr=sf, env=env
    )
    os.close(sf)

    print("Waiting for debugger...")
    ru(mf, b">", timeout=15)
    print("Ready.")

    for target in [240, 480, 720, 960]:
        print(f"\n=== Checking at frame {target} ===")
        for i in range(target):
            sc(mf, "frame")
            if (i + 1) % 120 == 0:
                ru(mf, b">", timeout=10)
        ru(mf, b">", timeout=10)

        # Read first 32 entries to check if data is populated
        non_ff = 0
        entries = []
        for i in range(NENTRIES):
            val = read_hw(mf, IWSTART + i * 2)
            entries.append(val)
            if val != 0xFFFF:
                non_ff += 1

        pal_counts = [0] * 16
        for e in entries:
            if e != 0xFFFF:
                pal_counts[(e >> 12) & 0xF] += 1

        print(f"  Non-0xFFFF entries: {non_ff}/{NENTRIES}")
        if non_ff > 0:
            print(
                f"  Palette distribution (non-FFFF): {[(p, c) for p, c in enumerate(pal_counts) if c > 0]}"
            )
            # Print first 32 entries as hex
            for i in range(0, min(NENTRIES, 64), 16):
                line = f"  {IWSTART - 0x03000000 + i * 2:04X}: "
                for j in range(16):
                    line += f"{entries[i + j]:04X} "
                print(line)

            # Save full dump
            if non_ff > 10:
                with open(f"/tmp/mgba_iwram_frame{target}.txt", "w") as f:
                    for i in range(0, NENTRIES, 16):
                        line = f"{IWSTART - 0x03000000 + i * 2:04X}: "
                        for j in range(16):
                            line += f"{entries[i + j]:04X} "
                        f.write(line.strip() + "\n")
                print(f"  Saved to /tmp/mgba_iwram_frame{target}.txt")
        else:
            print("  Still all 0xFFFF (uninitialized)")

    sc(mf, "quit")
    time.sleep(0.5)
    proc.kill()
    os.close(mf)


if __name__ == "__main__":
    main()
