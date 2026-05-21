from pathlib import Path
import math
import zlib
import struct

OUTPUT_DIRS = [Path("src-tauri/icons"), Path("public")]
BASE_SIZE = 512

# Utility: save RGBA array as PNG

def write_png(path: Path, width: int, height: int, rgba: bytes) -> None:
    def png_chunk(chunk_type: bytes, data: bytes) -> bytes:
        return struct.pack(
            ">I", len(data)
        ) + chunk_type + data + struct.pack(
            ">I", zlib.crc32(chunk_type + data) & 0xFFFFFFFF
        )

    raw_data = b""
    for y in range(height):
        start = y * width * 4
        row = rgba[start:start + width * 4]
        raw_data += b"\x00" + row

    chunks = [
        png_chunk(b"IHDR", struct.pack(
            ">IIBBBBB",
            width,
            height,
            8,
            6,
            0,
            0,
            0,
        )),
        png_chunk(b"IDAT", zlib.compress(raw_data, level=9)),
        png_chunk(b"IEND", b"")
    ]
    path.write_bytes(b"\x89PNG\r\n\x1a\n" + b"".join(chunks))


def clamp(val: int) -> int:
    return max(0, min(255, val))


def mix(a: int, b: int, t: float) -> int:
    return clamp(int(a * (1 - t) + b * t))


def draw_icon(size: int) -> bytes:
    width = height = size
    rgba = bytearray(width * height * 4)
    cx = width // 2
    cy = int(height * 0.52)
    body_radius = int(width * 0.4)
    highlight_center = (int(width * 0.53), int(height * 0.3))
    highlight_radius = int(width * 0.15)

    for y in range(height):
        for x in range(width):
            idx = (y * width + x) * 4
            dx = x - cx
            dy = y - cy
            dist = math.hypot(dx, dy)
            a = 0
            r = g = b = 0
            if dist <= body_radius:
                t = dist / body_radius
                r = mix(255, 190, t * 0.8)
                g = mix(70, 30, t * 0.8)
                b = mix(50, 30, t * 0.8)
                a = 255
                # green stem area on top
                stem_dx = x - int(width * 0.5)
                stem_dy = y - int(height * 0.16)
                if 0 <= stem_dx <= width * 0.15 and -width * 0.05 <= stem_dy <= width * 0.23:
                    if stem_dy > 0 and stem_dx < (width * 0.15) * (1 - stem_dy / (width * 0.22)):
                        r = 45
                        g = 145
                        b = 55
                # highlight
                hx = x - highlight_center[0]
                hy = y - highlight_center[1]
                if math.hypot(hx, hy) <= highlight_radius:
                    k = 1 - math.hypot(hx, hy) / highlight_radius
                    r = mix(r, 255, 0.65 * k)
                    g = mix(g, 255, 0.65 * k)
                    b = mix(b, 255, 0.65 * k)
            else:
                rgba[idx:idx+4] = (0, 0, 0, 0)
                continue

            # clock face
            clock_radius = int(width * 0.22)
            clock_cx = cx
            clock_cy = int(height * 0.6)
            cdx = x - clock_cx
            cdy = y - clock_cy
            cdist = math.hypot(cdx, cdy)
            if cdist <= clock_radius:
                r = 245
                g = 245
                b = 245
                if cdist >= clock_radius - max(1, width // 128):
                    r = g = b = 220
                if cdist <= clock_radius * 0.2:
                    r = g = b = 220
                # clock tick marks and hands
                if cdist <= clock_radius and cdist >= clock_radius * 0.65:
                    angle = math.atan2(cdy, cdx)
                    deg = math.degrees(angle) % 360
                    if abs(deg - 0) < 7.5 or abs(deg - 60) < 7.5 or abs(deg - 120) < 7.5 or abs(deg - 180) < 7.5 or abs(deg - 240) < 7.5 or abs(deg - 300) < 7.5:
                        r = g = b = 110
                # hour hand
                if cdist <= clock_radius * 0.5:
                    theta = math.atan2(-1, 0)
                    hand_x = int(clock_cx + math.cos(theta) * cdist)
                    hand_y = int(clock_cy + math.sin(theta) * cdist)
                    if abs(x - hand_x) < 3 and abs(y - hand_y) < 6:
                        r = g = b = 60
                # minute hand
                theta2 = math.atan2(0, 1)
                hand2_x = int(clock_cx + math.cos(theta2) * min(cdist, clock_radius * 0.4))
                hand2_y = int(clock_cy + math.sin(theta2) * min(cdist, clock_radius * 0.4))
                if abs(x - hand2_x) < 3 and abs(y - hand2_y) < 6 and cdist <= clock_radius * 0.4:
                    r = g = b = 60

            rgba[idx] = clamp(r)
            rgba[idx + 1] = clamp(g)
            rgba[idx + 2] = clamp(b)
            rgba[idx + 3] = clamp(a)

    return bytes(rgba)


def nearest_scale(src: bytes, src_w: int, src_h: int, tgt_w: int, tgt_h: int) -> bytes:
    tgt = bytearray(tgt_w * tgt_h * 4)
    for y in range(tgt_h):
        sy = int(y * src_h / tgt_h)
        for x in range(tgt_w):
            sx = int(x * src_w / tgt_w)
            src_idx = (sy * src_w + sx) * 4
            tgt_idx = (y * tgt_w + x) * 4
            tgt[tgt_idx:tgt_idx+4] = src[src_idx:src_idx+4]
    return bytes(tgt)


def make_ico(path: Path, png_datas: dict[int, bytes]) -> None:
    entries = []
    data_blocks = []
    offset = 6 + 16 * len(png_datas)
    for size, png_data in png_datas.items():
        width = size if size < 256 else 0
        height = size if size < 256 else 0
        entry = struct.pack(
            "<BBBBHHII",
            width,
            height,
            0,
            0,
            1,
            32,
            len(png_data),
            offset,
        )
        entries.append(entry)
        data_blocks.append(png_data)
        offset += len(png_data)
    path.write_bytes(b"\x00\x00\x01\x00" + struct.pack("<H", len(png_datas)) + b"".join(entries) + b"".join(data_blocks))


def make_icns(path: Path, png_datas: dict[str, bytes]) -> None:
    blocks = []
    total_length = 8
    for icon_type, data in png_datas.items():
        length = 8 + len(data)
        blocks.append(icon_type.encode("ascii") + struct.pack(">I", length) + data)
        total_length += length
    path.write_bytes(b"icns" + struct.pack(">I", total_length) + b"".join(blocks))


def main() -> None:
    base = draw_icon(BASE_SIZE)
    outputs = [32, 64, 128, 256, 512]
    png_datas = {}
    for size in outputs:
        if size == BASE_SIZE:
            data = base
        else:
            data = nearest_scale(base, BASE_SIZE, BASE_SIZE, size, size)
        for out_dir in OUTPUT_DIRS:
            out_dir.mkdir(parents=True, exist_ok=True)
            write_png(out_dir / f"{size}x{size}.png", size, size, data)
            if size == 256:
                write_png(out_dir / "128x128@2x.png", size, size, data)
        if size in (32, 64, 128, 256):
            write_png(Path("src-tauri/icons") / f"{size}x{size}.png", size, size, data)
            if size == 256:
                write_png(Path("src-tauri/icons") / "128x128@2x.png", size, size, data)
            png_datas[size] = data_to_png_bytes(size, size, data)
        if size == 512:
            write_png(Path("src-tauri/icons") / "icon.png", size, size, data)
            write_png(Path("public") / "icon.png", size, size, data)

    png_datas_for_ico = {size: data_to_png_bytes(size, size, nearest_scale(base, BASE_SIZE, BASE_SIZE, size, size)) for size in [32, 64, 128, 256]}
    make_ico(Path("src-tauri/icons/icon.ico"), png_datas_for_ico)

    png_datas_for_icns = {
        "ic08": data_to_png_bytes(128, 128, nearest_scale(base, BASE_SIZE, BASE_SIZE, 128, 128)),
        "ic09": data_to_png_bytes(256, 256, nearest_scale(base, BASE_SIZE, BASE_SIZE, 256, 256)),
        "ic10": data_to_png_bytes(512, 512, base),
    }
    make_icns(Path("src-tauri/icons/icon.icns"), png_datas_for_icns)

    print("Generated tomato icon assets.")


def data_to_png_bytes(width: int, height: int, rgba: bytes) -> bytes:
    def png_chunk(chunk_type: bytes, data: bytes) -> bytes:
        return struct.pack(
            ">I", len(data)
        ) + chunk_type + data + struct.pack(
            ">I", zlib.crc32(chunk_type + data) & 0xFFFFFFFF
        )
    raw_data = b""
    for y in range(height):
        row = rgba[y * width * 4:(y + 1) * width * 4]
        raw_data += b"\x00" + row
    chunks = [
        png_chunk(b"IHDR", struct.pack(
            ">IIBBBBB",
            width,
            height,
            8,
            6,
            0,
            0,
            0,
        )),
        png_chunk(b"IDAT", zlib.compress(raw_data, level=9)),
        png_chunk(b"IEND", b"")
    ]
    return b"\x89PNG\r\n\x1a\n" + b"".join(chunks)


if __name__ == "__main__":
    main()
