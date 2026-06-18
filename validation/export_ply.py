#!/usr/bin/env python3
"""Export a tt-splat gaussian set to a standard 3DGS .ply + an oracle-matched view.json, and render
the PyTorch arm-A reference. Drives the viewer's .ply path and validates the loader by PSNR.

    ../tt-splat/.venv/bin/python validation/export_ply.py [G]

Writes validation/{model.ply, view.json, oracle_ply.png}. The .ply uses the INRIA-3DGS property
layout (x y z, nx ny nz, f_dc_0..2, opacity, scale_0..2, rot_0..3) that src/scene.rs::load_ply reads.
"""
import json
import math
import os
import struct
import sys

import numpy as np
import torch
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
TT_SPLAT = os.path.abspath(os.path.join(HERE, "..", "..", "tt-splat"))
sys.path.insert(0, TT_SPLAT)

from spike import arms, camera as cam_mod, forward, geometry, model as model_mod  # noqa: E402

K, BLUR_EPS, NEAR = 4.0, 0.3, 0.2
W = H = 512
FOVY = math.radians(60.0)


def write_ply(path, m):
    means = m.means3d.detach().cpu().numpy().astype(np.float32)
    scales = m.log_scales.detach().cpu().numpy().astype(np.float32)
    quats = m.quats.detach().cpu().numpy().astype(np.float32)
    color = m.color_dc.detach().cpu().numpy().astype(np.float32)
    opac = m.opacity_raw.detach().cpu().numpy().astype(np.float32).reshape(-1, 1)
    n = means.shape[0]
    normals = np.zeros((n, 3), np.float32)

    names = (["x", "y", "z", "nx", "ny", "nz", "f_dc_0", "f_dc_1", "f_dc_2", "opacity"]
             + [f"scale_{i}" for i in range(3)] + [f"rot_{i}" for i in range(4)])
    header = "ply\nformat binary_little_endian 1.0\n"
    header += f"element vertex {n}\n"
    header += "".join(f"property float {nm}\n" for nm in names)
    header += "end_header\n"

    rows = np.concatenate([means, normals, color, opac, scales, quats], axis=1).astype(np.float32)
    with open(path, "wb") as f:
        f.write(header.encode("ascii"))
        f.write(rows.tobytes())
    return n


def main():
    G = int(sys.argv[1]) if len(sys.argv) > 1 else 2000
    m = model_mod.GaussianModel(G, gt=True, seed=0)

    ply_path = os.path.join(HERE, "model.ply")
    n = write_ply(ply_path, m)
    print(f"wrote {ply_path}  ({n} gaussians)")

    # Auto-frame: orbit the centroid at a distance that fits the bounding sphere in the fov.
    means = m.means3d.detach()
    centroid = means.mean(0)
    bound = (means - centroid).norm(dim=1).max().clamp(min=1e-3)
    radius = 1.4 * bound / math.sin(FOVY / 2)
    eye = centroid + torch.tensor([0.0, 0.0, -float(radius)])  # look down +z toward the centroid
    r_v, t_v = cam_mod.look_at_opencv(eye=eye.tolist(), target=centroid.tolist(),
                                      up=[0.0, 1.0, 0.0], dtype=torch.float32)
    focal = (H * 0.5) / math.tan(FOVY / 2)
    cam = dict(r_v=r_v.reshape(-1).tolist(), t_v=t_v.tolist(),
               fx=focal, fy=focal, cx=W / 2.0, cy=H / 2.0)
    bg = dict(w_b=0.02, c_b=[0.0, 0.0, 0.0])
    view = dict(width=W, height=H, camera=cam, background=bg)
    view_path = os.path.join(HERE, "view.json")
    with open(view_path, "w") as f:
        json.dump(view, f, indent=2)
    print(f"wrote {view_path}")

    # Oracle render (arm A) with the same camera + background.
    R = geometry.quat_to_rotmat(m.quats)
    cov = geometry.cov3d(torch.exp(m.log_scales), R)
    mu2d, conic, depth, keep = geometry.project_ewa(
        m.means3d, cov, r_v, t_v, focal, focal, W / 2.0, H / 2.0, BLUR_EPS, NEAR)
    px, py = cam_mod.pixel_grid(H, W)
    Q = forward.quad_form(px, py, mu2d, conic)
    w_geo = forward.poly_splat_wgeo(Q, K, keep)
    color = forward.color_from_dc(m.color_dc)
    img = arms.blend_A(w_geo, m.opacity_raw,
                       color, torch.tensor(bg["w_b"]), torch.tensor(bg["c_b"])).reshape(H, W, 3)

    arr = (img.detach().clamp(0, 1) * 255).round().to(torch.uint8).cpu().numpy()
    out = os.path.join(HERE, "oracle_ply.png")
    Image.fromarray(arr, mode="RGB").save(out)
    print(f"wrote {out}  ({keep.sum().item()}/{G} kept, mean={img.mean().item():.4f})")


if __name__ == "__main__":
    main()
