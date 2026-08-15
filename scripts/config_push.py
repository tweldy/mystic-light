#!/usr/bin/env python3
"""Send OpenRGB's initial_setup_array (report 0x50), then drive per-LED colours.

This is NOT brute-forced mode bytes -- it is the verbatim array from OpenRGB's
MSIMysticLight761Controller, the board-specific reference implementation for
MS-7E59 (this exact board), sent at detection time under the comment
"Push config so setting colors works". OpenRGB already sent it on this machine
last session with no ill effect.

    config_push.py dump     # show current config, no writes
    config_push.py push     # dump, push config, dump again
    config_push.py run [s]  # push config then hold RED/GREEN/BLUE on JARGB1/2/3
"""
import fcntl
import sys
import time

HIDRAW = "/dev/hidraw2"
_IOC_WRITE, _IOC_READ = 1, 2
SETUP_SIZE = 290
PERLED_LEN = 727

ZONE_NAMES = [
    "JARGB1", "JARGB2", "JARGB3", "JAF",
    "JPIPE1", "JPIPE2", "JPIPE3", "JPIPE4", "JPIPE5",
    "JRGB1", "JRGB2",
    "Onboard1", "Onboard2", "Onboard3", "Onboard4", "Onboard5", "Onboard6",
    "SelectAll",
]

ACTIVE = [0x09, 0xFF, 0x00, 0x00, 0x00, 0xFF, 0x00, 0x00,
          0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0x03, 0x15, 0x78]
PIPE_ON = [0x09, 0xFF, 0x00, 0x00, 0x00, 0xFF, 0x00, 0x00,
           0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0x03, 0x95, 0x1E]
PIPE_OFF = [0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x94, 0x1E]

SETUP = [0x50]
for _row in (ACTIVE, ACTIVE, ACTIVE, ACTIVE,      # JARGB 1/2/3 + JAF
             PIPE_ON, PIPE_OFF, PIPE_ON, PIPE_OFF, PIPE_OFF,   # JPIPE 1-5
             PIPE_ON, PIPE_OFF,                    # JRGB 1/2
             PIPE_OFF, PIPE_OFF, PIPE_OFF, PIPE_OFF, PIPE_OFF, PIPE_OFF,  # Onboard
             PIPE_ON):                             # "Select all?"
    SETUP.extend(_row)
SETUP.append(0x00)
assert len(SETUP) == SETUP_SIZE, len(SETUP)


def _ioc(d, t, nr, size):
    return (d << 30) | (size << 16) | (ord(t) << 8) | nr


def get_feature(fd, report_id, length):
    buf = bytearray(length)
    buf[0] = report_id
    fcntl.ioctl(fd, _ioc(_IOC_WRITE | _IOC_READ, "H", 0x07, length), buf, True)
    return bytes(buf)


def set_feature(fd, payload):
    return fcntl.ioctl(fd, _ioc(_IOC_WRITE | _IOC_READ, "H", 0x06, len(payload)),
                       bytearray(payload), True)


def dump(fd, label):
    data = get_feature(fd, 0x50, SETUP_SIZE)
    print(f"\n--- {label} (report id {data[0]:#04x}) ---")
    for i, name in enumerate(ZONE_NAMES):
        row = data[1 + i * 16: 17 + i * 16]
        want = bytes(SETUP[1 + i * 16: 17 + i * 16])
        flag = "" if row == want else "   <== differs from reference"
        print(f"  {name:<10} {row.hex(' ')}{flag}")
    return data


def build_perled(hdr1, rgb, count):
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
    mode = sys.argv[1] if len(sys.argv) > 1 else "dump"
    fd = open(HIDRAW, "rb+", buffering=0)

    dump(fd, "BEFORE")
    if mode == "dump":
        return

    n = set_feature(fd, SETUP)
    print(f"\nPushed config array: {n} bytes")
    time.sleep(0.2)
    dump(fd, "AFTER")

    if mode != "run":
        return

    seconds = int(sys.argv[2]) if len(sys.argv) > 2 else 45
    print(f"\nHolding JARGB1=RED JARGB2=GREEN JARGB3=BLUE for {seconds}s")
    targets = ((0, (0xFF, 0, 0)), (1, (0, 0xFF, 0)), (2, (0, 0, 0xFF)))
    end = time.time() + seconds
    while time.time() < end:
        for hdr1, rgb in targets:
            set_feature(fd, build_perled(hdr1, rgb, 240))
        time.sleep(0.1)
    print("done")


if __name__ == "__main__":
    main()
