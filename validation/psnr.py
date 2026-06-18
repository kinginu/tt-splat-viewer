#!/usr/bin/env python3
"""Compute PSNR between two PNGs (RGB). Used to gate the Rust render against the oracle (CLAUDE.md §5).

    python3 validation/psnr.py validation/oracle.png validation/rust.png
"""
import sys

import numpy as np
from PIL import Image


def load_rgb(path):
    return np.asarray(Image.open(path).convert("RGB"), dtype=np.float64)


def main():
    a = load_rgb(sys.argv[1])
    b = load_rgb(sys.argv[2])
    if a.shape != b.shape:
        sys.exit(f"shape mismatch: {a.shape} vs {b.shape}")
    mse = np.mean((a - b) ** 2)
    if mse == 0.0:
        print("PSNR = inf dB (bit-identical)")
        return
    psnr = 10.0 * np.log10(255.0 ** 2 / mse)
    max_abs = np.max(np.abs(a - b))
    print(f"PSNR = {psnr:.2f} dB   (MSE={mse:.4f}, max|Δ|={max_abs:.0f}/255)")
    print("PASS (>50 dB)" if psnr > 50.0 else "CHECK conventions (<50 dB) — see CLAUDE.md §5")


if __name__ == "__main__":
    main()
