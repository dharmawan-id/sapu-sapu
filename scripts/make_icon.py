"""Generate Sapu's app icon: a neo-brutalist timber tile with a hard offset
shadow and a bold paper 'S' monogram. Reproducible, committed to the repo.

Run: python scripts/make_icon.py
Needs Pillow (pip install Pillow).
"""
import os
from PIL import Image, ImageDraw, ImageFont

SZ = 1024
INK = (11, 12, 13, 255)
TIMBER = (156, 107, 53, 255)
PAPER = (239, 231, 214, 255)

img = Image.new("RGBA", (SZ, SZ), (0, 0, 0, 0))
d = ImageDraw.Draw(img)

# hard offset shadow (down-right), flat ink, no blur (anti-visual-slop)
d.rectangle([150, 150, 980, 980], fill=INK)

# timber tile with thick ink border, sharp corners
x0, y0, x1, y1 = 44, 44, 884, 884
d.rectangle([x0, y0, x1, y1], fill=TIMBER, outline=INK, width=40)

# paper 'S' monogram, heavy weight
font = None
for path in ("C:/Windows/Fonts/ariblk.ttf", "C:/Windows/Fonts/arialbd.ttf"):
    if os.path.exists(path):
        font = ImageFont.truetype(path, 560)
        break
if font is None:
    font = ImageFont.load_default()

text = "S"
cx, cy = (x0 + x1) // 2, (y0 + y1) // 2
bbox = d.textbbox((0, 0), text, font=font)
tw, th = bbox[2] - bbox[0], bbox[3] - bbox[1]
d.text((cx - tw / 2 - bbox[0], cy - th / 2 - bbox[1]), text, font=font, fill=PAPER)

out_dir = os.path.join(os.path.dirname(__file__), "..", "src-tauri", "icons")
os.makedirs(out_dir, exist_ok=True)
ico = os.path.join(out_dir, "icon.ico")
img.save(ico, sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)])
img.save(os.path.join(out_dir, "icon.png"))
print("wrote", ico)
