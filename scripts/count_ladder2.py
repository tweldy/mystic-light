#!/usr/bin/env python3
"""Walk the LED count down, each step in its own colour, on all three headers.

Rationale: every test so far has used count=240 (OpenRGB's GetMaxDirectLeds).
The board manual rates JARGB_V2 at 180. HID ioctl success only proves the kernel
delivered the report -- firmware may silently drop a report whose count exceeds
what the header accepts. One observation pass finds the highest count that lights.

Pushes the reference config array first (the documented enable step), then holds
each count for 8s while driving continuously at 10Hz so nothing can reclaim it.
"""
import fcntl
import sys
import time

HIDRAW = "/dev/hidraw2"
_IOC_WRITE, _IOC_READ = 1, 2
PERLED_LEN = 727

ACTIVE = [0x09, 0xFF, 0x00, 0x00, 0x00, 0xFF, 0x00, 0x00,
          0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0x03, 0x15, 0x78]
PIPE_ON = [0x09, 0xFF, 0x00, 0x00, 0x00, 0xFF, 0x00, 0x00,
           0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0x03, 0x95, 0x1E]
PIPE_OFF = [0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x94, 0x1E]

SETUP = [0x50]
for _row in (ACTIVE, ACTIVE, ACTIVE, ACTIVE,
             PIPE_ON, PIPE_OFF, PIPE_ON, PIPE_OFF, PIPE_OFF,
             PIPE_ON, PIPE_OFF,
             PIPE_OFF, PIPE_OFF, PIPE_OFF, PIPE_OFF, PIPE_OFF, PIPE_OFF,
             PIPE_ON):
    SETUP.extend(_row)
SETUP.append(0x00)

# (count, colour, name) -- single-channel colours keep current draw low
LADDER = [
    (180, (0xFF, 0x00, 0x00), "RED"),
    (120, (0x00, 0xFF, 0x00), "GREEN"),
    (60,  (0x00, 0x00, 0xFF), "BLUE"),
    (30,  (0xFF, 0xFF, 0x00), "YELLOW"),
    (16,  (0xFF, 0x00, 0xFF), "MAGENTA"),
    (8,   (0x00, 0xFF, 0xFF), "CYAN"),
    (4,   (0xFF, 0xFF, 0xFF), "WHITE"),
]


def _ioc(d, t, nr, size):
    return (d << 30) | (size << 16) | (ord(t) << 8) | nr


def set_feature(fd, payload):
    return fcntl.ioctl(fd, _ioc(_IOC_WRITE | _IOC_READ, "H", 0x06, len(payload)),
                       bytearray(payload), True)


def build(hdr1, rgb, count):
    buf = bytearray(PERLED_LEN)
    buf[0] = 0x51
    buf[1] = 0x09
    buf[2] = 0x04
    buf[3] = hdr1
    buf[6] = count
    for i in range(count):
        buf[7 + i * 3: 10 + i * 3] = bytes(rgb)
    return bytes(buf)


def main():
    hold = int(sys.argv[1]) if len(sys.argv) > 1 else 8
    fd = open(HIDRAW, "rb+", buffering=0)

    try:
        n = set_feature(fd, SETUP)
        print(f"config array pushed ({n} bytes)\n", flush=True)
    except OSError as e:
        print(f"config push FAILED ({e})\n", flush=True)

    for count, rgb, name in LADDER:
        print(f"[{time.strftime('%H:%M:%S')}] count={count:<4} {name}", flush=True)
        end = time.time() + hold
        ok = True
        while time.time() < end:
            for hdr1 in (0, 1, 2):
                try:
                    set_feature(fd, build(hdr1, rgb, count))
                except OSError as e:
                    if ok:
                        print(f"    write rejected at header {hdr1}: {e}", flush=True)
                        ok = False
            time.sleep(0.1)

    for hdr1 in (0, 1, 2):
        set_feature(fd, build(hdr1, (0, 0, 0), 180))
    print("\nladder complete, headers blanked", flush=True)
    fd.close()


if __name__ == "__main__":
    main()
