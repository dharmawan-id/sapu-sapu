"""Generate the README banner: a typographic neo-brutalist timber banner.
Run: python scripts/make_banner.py  (needs Pillow)
"""
import os
from PIL import Image, ImageDraw, ImageFont

W, H = 1280, 600
PAPER = (239, 231, 214)
INK = (11, 12, 13)
TIMBER = (156, 107, 53)
SOFT = (58, 54, 47)

img = Image.new("RGB", (W, H), PAPER)
d = ImageDraw.Draw(img)


def font(size, black=True):
    paths = ["C:/Windows/Fonts/ariblk.ttf"] if black else ["C:/Windows/Fonts/arialbd.ttf"]
    for p in paths:
        if os.path.exists(p):
            return ImageFont.truetype(p, size)
    return ImageFont.load_default()


def text_w(s, f):
    b = d.textbbox((0, 0), s, font=f)
    return b[2] - b[0]


# outer ink frame
d.rectangle([22, 22, W - 22, H - 22], outline=INK, width=6)

# hard-shadow timber tile, top-right decoration
d.rectangle([W - 230, 70, W - 70, 230], fill=INK)
d.rectangle([W - 250, 50, W - 90, 210], fill=TIMBER, outline=INK, width=6)
sf_big = font(120)
sw = text_w("S", sf_big)
d.text((W - 170 - sw / 2, 78), "S", font=sf_big, fill=PAPER)

# eyebrow
eb = font(24, False)
d.rectangle([72, 104, 72 + text_w("WINDOWS DISK CLEANER", eb) + 28, 104 + 42], fill=TIMBER)
d.text((86, 112), "WINDOWS DISK CLEANER", font=eb, fill=PAPER)

# title
tf = font(120)
d.text((68, 180), "SAPU SAPU", font=tf, fill=INK)
tw = text_w("SAPU SAPU", tf)
# timber underline bar
d.rectangle([74, 330, 74 + tw, 348], fill=TIMBER)

# subtitle + meta
sub = font(30, False)
d.text((74, 420), "An honest cleaner. Scan C: and D:, preview, clean the safe caches.", font=sub, fill=SOFT)
meta = font(24, False)
d.text((74, 486), "TAURI  /  RUST  /  2.8 MB NATIVE  /  MIT", font=meta, fill=INK)

out = os.path.join(os.path.dirname(__file__), "..", "assets", "banner.png")
os.makedirs(os.path.dirname(out), exist_ok=True)
img.save(out)
print("wrote", out)
