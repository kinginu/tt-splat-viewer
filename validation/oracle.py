#!/usr/bin/env python3
"""Oracle side of the PSNR validation harness (CLAUDE.md §5).

Defines a small fixed scene, writes `scene.json` (the SAME inputs the Rust offscreen renderer
loads), and renders it with the tt-splat PyTorch reference (arm A = depth-free WSR), saving a PNG.
Run with the tt-splat venv:  ../tt-splat/.venv/bin/python validation/oracle.py

    quad_form -> poly_splat_wgeo -> blend_A   (mirrors spike/render.py arm "A")
"""
import json
import math
import os
import sys

import numpy as np
import torch
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
TT_SPLAT = os.path.abspath(os.path.join(HERE, "..", "..", "tt-splat"))
sys.path.insert(0, TT_SPLAT)

from spike import arms, camera as cam_mod, forward, geometry  # noqa: E402

K = 4.0
BLUR_EPS = 0.3
NEAR = 0.2

W = H = 128
FOCAL = float(W)  # fx = fy = width  (~53° hfov)


def build_scene():
    """Three overlapping colored gaussians on a gray WSR background. Raw (pre-activation) params."""
    gaussians = [
        dict(mean=[-0.3, 0.0, 4.0], log_scale=[math.log(0.30)] * 3, quat=[1, 0, 0, 0],
             color_dc=[1.2, -0.5, -0.5], opacity_raw=4.0),
        dict(mean=[0.3, 0.0, 4.2], log_scale=[math.log(0.30)] * 3, quat=[1, 0, 0, 0],
             color_dc=[-0.5, 1.2, -0.5], opacity_raw=4.0),
        dict(mean=[0.0, 0.25, 3.8], log_scale=[math.log(0.25)] * 3, quat=[1, 0, 0, 0],
             color_dc=[-0.5, -0.3, 1.4], opacity_raw=4.0),
    ]
    background = dict(w_b=0.05, c_b=[0.1, 0.1, 0.12])

    # OpenCV camera at the origin looking down +z (identity extrinsics for this scene).
    r_v, t_v = cam_mod.look_at_opencv(
        eye=[0.0, 0.0, 0.0], target=[0.0, 0.0, 1.0], up=[0.0, 1.0, 0.0], dtype=torch.float32)
    camera = dict(
        r_v=r_v.reshape(-1).tolist(),  # row-major 3x3
        t_v=t_v.tolist(),
        fx=FOCAL, fy=FOCAL, cx=W / 2.0, cy=H / 2.0,
    )
    return dict(width=W, height=H, camera=camera, background=background, gaussians=gaussians)


def render(scene):
    g = scene["gaussians"]
    means3d = torch.tensor([x["mean"] for x in g], dtype=torch.float32)
    log_scales = torch.tensor([x["log_scale"] for x in g], dtype=torch.float32)
    quats = torch.tensor([x["quat"] for x in g], dtype=torch.float32)
    color_dc = torch.tensor([x["color_dc"] for x in g], dtype=torch.float32)
    opacity_raw = torch.tensor([x["opacity_raw"] for x in g], dtype=torch.float32)

    c = scene["camera"]
    r_v = torch.tensor(c["r_v"], dtype=torch.float32).reshape(3, 3)
    t_v = torch.tensor(c["t_v"], dtype=torch.float32)
    w_b = torch.tensor(scene["background"]["w_b"], dtype=torch.float32)
    c_b = torch.tensor(scene["background"]["c_b"], dtype=torch.float32)

    R = geometry.quat_to_rotmat(quats)
    cov = geometry.cov3d(torch.exp(log_scales), R)
    mu2d, conic, depth, keep = geometry.project_ewa(
        means3d, cov, r_v, t_v, c["fx"], c["fy"], c["cx"], c["cy"], BLUR_EPS, NEAR)

    px, py = cam_mod.pixel_grid(H, W)
    Q = forward.quad_form(px, py, mu2d, conic)
    w_geo = forward.poly_splat_wgeo(Q, K, keep)
    color = forward.color_from_dc(color_dc)
    img = arms.blend_A(w_geo, opacity_raw, color, w_b, c_b).reshape(H, W, 3)
    return img


def main():
    scene = build_scene()
    scene_path = os.path.join(HERE, "scene.json")
    with open(scene_path, "w") as f:
        json.dump(scene, f, indent=2)
    print(f"wrote {scene_path}")

    img = render(scene)
    arr = (img.clamp(0.0, 1.0) * 255.0).round().to(torch.uint8).cpu().numpy()
    out = os.path.join(HERE, "oracle.png")
    Image.fromarray(arr, mode="RGB").save(out)
    print(f"wrote {out}  ({W}x{H}, mean={img.mean().item():.4f})")


if __name__ == "__main__":
    main()
