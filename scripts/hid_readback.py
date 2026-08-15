#!/usr/bin/env python3
"""Decisive board-vs-cable probe: write a colour, then read the feature report back.

If the device echoes our RGB bytes, the board accepted the write and is driving the
header -- meaning the fault is downstream (hub still inline, chain direction, seating).
If the readback is zeros/garbage or the write is rejected, the fault is on our side
(report length, missing config push, header disabled).
"""
import fcntl
import struct
import sys

HIDRAW = "/dev/hidraw2"
REPORT_ID = 0x51

_IOC_WRITE, _IOC_READ = 1, 2


def _ioc(d, t, nr, size):
    return (d << 30) | (size << 16) | (ord(t) << 8) | nr


def get_feature(fd, report_id, length):
    buf = bytearray(length)
    buf[0] = report_id
    # HIDIOCGFEATURE(len) = _IOC(_IOC_WRITE|_IOC_READ, 'H', 0x07, len)
    req = _ioc(_IOC_WRITE | _IOC_READ, "H", 0x07, length)
    n = fcntl.ioctl(fd, req, buf, True)
    return n, bytes(buf)


def set_feature(fd, payload):
    # HIDIOCSFEATURE(len) = _IOC(_IOC_WRITE|_IOC_READ, 'H', 0x06, len)
    req = _ioc(_IOC_WRITE | _IOC_READ, "H", 0x06, len(payload))
    return fcntl.ioctl(fd, req, bytearray(payload), True)


def build(header, rgb, count, report_len):
    buf = bytearray(report_len)
    buf[0] = REPORT_ID
    buf[1] = 0x09
    buf[2] = 0x04       # ARGB per-LED
    buf[3] = header     # 0/1/2 = JARGB 1/2/3
    buf[6] = count
    for i in range(count):
        buf[7 + i * 3: 10 + i * 3] = bytes(rgb)
    return bytes(buf)


def main():
    report_len = int(sys.argv[1]) if len(sys.argv) > 1 else 727
    fd = open(HIDRAW, "rb+", buffering=0)

    for length in (727, 761):
        try:
            n, data = get_feature(fd, REPORT_ID, length)
            nz = [i for i, b in enumerate(data) if b]
            print(f"GET len={length}: ioctl returned {n}, "
                  f"header={data[:8].hex(' ')}, nonzero bytes={len(nz)}")
            if nz:
                print(f"   first nonzero offsets: {nz[:12]}")
        except OSError as e:
            print(f"GET len={length}: FAILED ({e})")

    print()
    for hdr, name, rgb in ((0, "JARGB1", (0xFF, 0x00, 0x00)),
                           (1, "JARGB2", (0x00, 0xFF, 0x00)),
                           (2, "JARGB3", (0x00, 0x00, 0xFF))):
        pkt = build(hdr, rgb, 240, report_len)
        try:
            n = set_feature(fd, pkt)
            print(f"SET {name} len={report_len} count=240: wrote {n} bytes")
        except OSError as e:
            print(f"SET {name} len={report_len}: FAILED ({e})")
            continue
        try:
            n, data = get_feature(fd, REPORT_ID, report_len)
            echo = data[7:16].hex(" ")
            match = data[3] == hdr and tuple(data[7:10]) == rgb
            print(f"    readback: hdr_byte={data[3]} count={data[6]} "
                  f"rgb[0:3]={echo}  -> {'ECHOES OUR WRITE' if match else 'does NOT match'}")
        except OSError as e:
            print(f"    readback FAILED ({e})")

    fd.close()


if __name__ == "__main__":
    main()
