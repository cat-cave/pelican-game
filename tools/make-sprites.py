#!/usr/bin/env python3
"""make-sprites — chroma-key the captured rider/wheel sprites to transparency.

Pipeline (tools/make-sprites.sh): the palette-faithful rider art is drawn by
the same canvas code as the design reference (12-color pixd palette) on a
magenta background, captured headlessly by chromium, then this script keys
the magenta out and trims to content, recording sprite origin offsets in
assets/sprites/sprites.json for the Bevy loader.
"""
import json
import os
import struct
import sys
import zlib

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
OUT = os.path.join(ROOT, 'assets', 'sprites')


def read_png(path):
    data = open(path, 'rb').read()
    assert data[:8] == b'\x89PNG\r\n\x1a\n', 'not a png'
    pos = 8
    w = h = bd = ct = None
    idat = b''
    while pos < len(data):
        ln = struct.unpack('>I', data[pos:pos + 4])[0]
        typ = data[pos + 4:pos + 8]
        chunk = data[pos + 8:pos + 8 + ln]
        if typ == b'IHDR':
            w, h, bd, ct = struct.unpack('>IIBB', chunk[:10])
        elif typ == b'IDAT':
            idat += chunk
        pos += 12 + ln
    assert bd == 8 and ct in (6, 2), f'unsupported png bd={bd} ct={ct}'
    has_alpha = ct == 6
    bpp = 4 if has_alpha else 3
    raw = zlib.decompress(idat)
    stride = w * bpp
    out = bytearray()
    prev = bytearray(stride)
    i = 0
    for y in range(h):
        f = raw[i]
        i += 1
        line = bytearray(raw[i:i + stride])
        i += stride
        if f == 1:
            for x in range(bpp, stride):
                line[x] = (line[x] + line[x - bpp]) & 255
        elif f == 2:
            for x in range(stride):
                line[x] = (line[x] + prev[x]) & 255
        elif f == 3:
            for x in range(stride):
                a = line[x - bpp] if x >= bpp else 0
                line[x] = (line[x] + ((a + prev[x]) >> 1)) & 255
        elif f == 4:
            for x in range(stride):
                a = line[x - bpp] if x >= bpp else 0
                c = prev[x - bpp] if x >= bpp else 0
                p = a + prev[x] - c
                pa, pb, pc = abs(p - a), abs(p - prev[x]), abs(p - c)
                pred = a if (pa <= pb and pa <= pc) else (prev[x] if pb <= pc else c)
                line[x] = (line[x] + pred) & 255
        out += line
        prev = line
    return w, h, bpp, bytes(out)


def paeth(a, b, c):
    p = a + b - c
    pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
    return a if (pa <= pb and pa <= pc) else (b if pb <= pc else c)


def write_png(path, w, h, px):
    raw = bytearray()
    stride = w * 4
    prev = bytearray(stride)
    for y in range(h):
        raw.append(0)  # filter none
        line = px[y * stride:(y + 1) * stride]
        raw += line
        prev = line

    def chunk(typ, data):
        c = struct.pack('>I', len(data)) + typ + data
        return c + struct.pack('>I', zlib.crc32(typ + data) & 0xffffffff)

    ihdr = struct.pack('>IIBBBBB', w, h, 8, 6, 0, 0, 0)
    png = b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', ihdr) + chunk(b'IDAT', zlib.compress(bytes(raw), 9)) + chunk(b'IEND', b'')
    open(path, 'wb').write(png)


def key_and_trim(src, dst, origin):
    w, h, bpp, raw = read_png(src)
    px = bytearray(w * h * 4)
    for y in range(h):
        for x in range(w):
            o = (y * w + x) * bpp
            r, g, b = raw[o], raw[o + 1], raw[o + 2]
            a = raw[o + 3] if bpp == 4 else 255
            # chroma key: near-magenta -> transparent (feathered)
            if r > 200 and b > 200 and g < 90:
                a = 0
            d = (y * w + x) * 4
            px[d], px[d + 1], px[d + 2], px[d + 3] = r, g, b, a
    # trim to content
    minx, miny, maxx, maxy = w, h, -1, -1
    for y in range(h):
        for x in range(w):
            if px[(y * w + x) * 4 + 3] > 0:
                if x < minx: minx = x
                if x > maxx: maxx = x
                if y < miny: miny = y
                if y > maxy: maxy = y
    assert maxx >= 0, 'empty sprite'
    tw, th = maxx - minx + 1, maxy - miny + 1
    trimmed = bytearray(tw * th * 4)
    for y in range(th):
        trimmed[y * tw * 4:(y + 1) * tw * 4] = px[((miny + y) * w + minx) * 4:((miny + y) * w + minx + tw) * 4]
    write_png(dst, tw, th, trimmed)
    # origin (chassis point) in trimmed coordinates
    ox = origin[0] - minx
    oy = origin[1] - miny
    return {'file': os.path.relpath(dst, ROOT), 'w': tw, 'h': th, 'origin': [ox, oy]}


def main():
    os.makedirs(OUT, exist_ok=True)
    meta = {}
    meta['wheel'] = key_and_trim('/tmp/spriteauth/wheel.png', os.path.join(OUT, 'wheel.png'), (64, 64))
    meta['rider'] = key_and_trim('/tmp/spriteauth/rider.png', os.path.join(OUT, 'rider.png'), (256, 220))
    meta['cloud'] = key_and_trim('/tmp/spriteauth/cloud.png', os.path.join(OUT, 'cloud.png'), (0, 0))
    with open(os.path.join(OUT, 'sprites.json'), 'w') as fh:
        json.dump(meta, fh, indent=2)
    print(json.dumps(meta, indent=2))


if __name__ == '__main__':
    main()
