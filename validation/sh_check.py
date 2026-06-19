#!/usr/bin/env python3
"""Cross-check the viewer's 3DGS SH color eval against an independent (INRIA-convention) implementation.

Writes a single big opaque gaussian with known f_dc + f_rest to a .ply, computes the expected
view-dependent color here, and (after the Rust --gs render) compares the center pixel.

    ../tt-splat/.venv/bin/python validation/sh_check.py            # writes model_sh.ply + view.json, prints expected
    cargo run --bin offscreen -- --gs validation/model_sh.ply validation/sh.png validation/view_sh.json
    ../tt-splat/.venv/bin/python validation/sh_check.py --compare  # reads sh.png, asserts match
"""
import json
import math
import os
import sys

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
TT_SPLAT = os.path.abspath(os.path.join(HERE, "..", "..", "tt-splat"))
sys.path.insert(0, TT_SPLAT)
import torch  # noqa: E402
from spike import camera as cam_mod  # noqa: E402

W = H = 256
FOVY = math.radians(60.0)
EYE = [1.0, 0.6, -3.0]
TARGET = [0.0, 0.0, 0.0]

C0 = 0.28209479177387814
C1 = 0.4886025119029199
C2 = [1.0925484305920792, -1.0925484305920792, 0.31539156525252005, -1.0925484305920792, 0.5462742152960396]
C3 = [-0.5900435899266435, 2.890611442640554, -0.4570457994644658, 0.3731763325901154,
      -0.4570457994644658, 1.445305721320277, -0.5900435899266435]


def eval_sh_color(dc, rest, direction):
    d = np.asarray(direction, float)
    d = d / np.linalg.norm(d)
    x, y, z = d
    xx, yy, zz, xy, yz, xz = x * x, y * y, z * z, x * y, y * z, x * z
    out = []
    for c in range(3):
        s = lambda k: rest[c * 15 + (k - 1)]  # noqa: E731
        r = C0 * dc[c]
        r += -C1 * y * s(1) + C1 * z * s(2) - C1 * x * s(3)
        r += C2[0] * xy * s(4) + C2[1] * yz * s(5) + C2[2] * (2 * zz - xx - yy) * s(6) \
            + C2[3] * xz * s(7) + C2[4] * (xx - yy) * s(8)
        r += C3[0] * y * (3 * xx - yy) * s(9) + C3[1] * xy * z * s(10) \
            + C3[2] * y * (4 * zz - xx - yy) * s(11) + C3[3] * z * (2 * zz - 3 * xx - 3 * yy) * s(12) \
            + C3[4] * x * (4 * zz - xx - yy) * s(13) + C3[5] * z * (xx - yy) * s(14) \
            + C3[6] * x * (xx - 3 * yy) * s(15)
        out.append(max(0.0, r + 0.5))
    return np.array(out)


def scene():
    mean = np.array([0.0, 0.0, 0.0], np.float32)
    f_dc = np.array([0.3, 0.05, -0.25], np.float32)
    rng = np.random.default_rng(0)
    f_rest = (rng.standard_normal(45) * 0.15).astype(np.float32)  # channel-major R×15,G×15,B×15
    focal = (H * 0.5) / math.tan(FOVY / 2)
    return mean, f_dc, f_rest, focal


def write_ply(path, mean, f_dc, f_rest):
    names = (["x", "y", "z", "nx", "ny", "nz", "f_dc_0", "f_dc_1", "f_dc_2"]
             + [f"f_rest_{i}" for i in range(45)] + ["opacity"]
             + [f"scale_{i}" for i in range(3)] + [f"rot_{i}" for i in range(4)])
    header = "ply\nformat binary_little_endian 1.0\n" + f"element vertex 1\n"
    header += "".join(f"property float {n}\n" for n in names) + "end_header\n"
    row = np.concatenate([
        mean, np.zeros(3, np.float32), f_dc, f_rest,
        np.array([10.0], np.float32),                 # opacity_raw -> sigmoid ~1
        np.full(3, math.log(0.8), np.float32),        # big, so the center pixel is ~flat
        np.array([1.0, 0.0, 0.0, 0.0], np.float32),   # identity quat
    ]).astype(np.float32)
    with open(path, "wb") as f:
        f.write(header.encode("ascii"))
        f.write(row.tobytes())


def main():
    mean, f_dc, f_rest, focal = scene()
    r_v, t_v = cam_mod.look_at_opencv(eye=EYE, target=TARGET, up=[0.0, 1.0, 0.0], dtype=torch.float32)
    r_v, t_v = r_v.numpy(), t_v.numpy()
    mu_cam = r_v @ mean + t_v
    z = mu_cam[2]
    col = int(focal * mu_cam[0] / z + W / 2)
    row = int(focal * mu_cam[1] / z + H / 2)
    view_dir = mean - np.asarray(EYE)  # cam center = eye
    expected = eval_sh_color(f_dc, f_rest, view_dir)

    if "--compare" in sys.argv:
        img = np.asarray(Image.open(os.path.join(HERE, "sh.png")).convert("RGB"), float) / 255.0
        got = img[row, col]
        exp = np.clip(expected * 0.99, 0, 1)  # center alpha ≈ 0.99 over black bg
        diff = np.abs(got - exp).max()
        print(f"pixel({row},{col}) got={got.round(3)} expected={exp.round(3)} max|Δ|={diff:.3f}")
        print("PASS" if diff < 0.02 else "FAIL (SH eval mismatch)")
        return

    write_ply(os.path.join(HERE, "model_sh.ply"), mean, f_dc, f_rest)
    view = dict(width=W, height=H,
                camera=dict(r_v=r_v.reshape(-1).tolist(), t_v=t_v.tolist(),
                            fx=focal, fy=focal, cx=W / 2.0, cy=H / 2.0),
                background=dict(w_b=0.0, c_b=[0.0, 0.0, 0.0]))
    with open(os.path.join(HERE, "view_sh.json"), "w") as f:
        json.dump(view, f, indent=2)
    print(f"wrote model_sh.ply + view_sh.json; expected center color = {expected.round(3)} at pixel ({row},{col})")


if __name__ == "__main__":
    main()
