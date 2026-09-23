"""Generate 128px WebP thumbs for pal grid / UI (keeps full assets for OG)."""
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "assets" / "pals"
DST = SRC / "thumbs"


def main() -> None:
    DST.mkdir(exist_ok=True)
    files = [p for p in SRC.glob("*.webp") if p.parent == SRC]
    before = after = 0
    for i, path in enumerate(files, 1):
        before += path.stat().st_size
        im = Image.open(path).convert("RGBA")
        im.thumbnail((128, 128), Image.Resampling.LANCZOS)
        out = DST / path.name
        im.save(out, "WEBP", quality=70, method=4)
        after += out.stat().st_size
        if i % 40 == 0 or i == len(files):
            print(f"{i}/{len(files)}")
    print(
        f"files={len(files)} before_kb={before/1024:.0f} "
        f"after_kb={after/1024:.0f} saved_kb={(before-after)/1024:.0f}"
    )


if __name__ == "__main__":
    main()
