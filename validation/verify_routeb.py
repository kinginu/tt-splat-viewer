#!/usr/bin/env python3
"""Verify the viewer's WSR rendering matches tt-splat's own render() for a route-B model exported via
tt-splat's plyio.save_ply — and demonstrate the white-background data issue.

Builds a scene = a colored "object" + many low-opacity WHITE haze gaussians (what white-background
training produces in empty space), exports it with spike/plyio, then renders arm A with BLACK vs WHITE
background. The Rust offscreen WSR render (BLACK, the viewer default) is PSNR-checked against the BLACK
oracle: a match proves the method is correct and the white look is purely the background convention.

    ../tt-splat/.venv/bin/python validation/verify_routeb.py            # write ply + oracle pngs + view.json
    cargo run --bin offscreen -- validation/routeb.ply validation/routeb_rust.png validation/view_routeb.json
    ../tt-splat/.venv/bin/python validation/verify_routeb.py --psnr     # report PSNR(rust, oracle_black)
"""
import json
import math
import os
import sys
import types

import numpy as np
import torch
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
TT_SPLAT = os.path.abspath(os.path.join(HERE, "..", "..", "tt-splat"))
sys.path.insert(0, TT_SPLAT)
from spike import arms, camera as cam_mod, forward, geometry, plyio  # noqa: E402

K, BLUR_EPS, NEAR = 4.0, 0.3, 0.2
W = H = 384
FOVY = math.radians(55.0)
C0 = 0.28209479177387814


def dc_for(color):  # invert color_from_dc so rendered color ≈ `color`
    return (np.asarray(color, np.float32) - 0.5) / C0


def build_model():
    rng = np.random.default_rng(0)
    means, dc, op, scale = [], [], [], []
    # Object: a handful of opaque green/brown blobs near the origin.
    for _ in range(40):
        means.append(rng.normal(0, 0.25, 3))
        dc.append(dc_for([rng.uniform(0.1, 0.3), rng.uniform(0.4, 0.8), rng.uniform(0.1, 0.3)]))
        op.append(5.0)                      # sigmoid≈0.99 opaque
        scale.append([math.log(0.06)] * 3)
    # White haze: many low-opacity WHITE gaussians filling space (what white-bg training puts in
    # "empty" regions). Invisible on white, a white cloud on black.
    for _ in range(1500):
        means.append(rng.normal(0, 0.6, 3))
        dc.append(dc_for([1.0, 1.0, 1.0]))
        op.append(-1.5)                     # sigmoid≈0.18 faint
        scale.append([math.log(0.05)] * 3)
    n = len(means)
    m = types.SimpleNamespace(
        means3d=torch.tensor(np.array(means), dtype=torch.float32),
        color_dc=torch.tensor(np.array(dc), dtype=torch.float32),
        opacity_raw=torch.tensor(np.array(op), dtype=torch.float32),
        log_scales=torch.tensor(np.array(scale), dtype=torch.float32),
        quats=torch.tensor(np.tile([1.0, 0, 0, 0], (n, 1)), dtype=torch.float32),
    )
    return m


def render_arm_a(m, r_v, t_v, focal, w_b, c_b):
    R = geometry.quat_to_rotmat(m.quats)
    cov = geometry.cov3d(torch.exp(m.log_scales), R)
    mu2d, conic, depth, keep = geometry.project_ewa(
        m.means3d, cov, r_v, t_v, focal, focal, W / 2.0, H / 2.0, BLUR_EPS, NEAR)
    px, py = cam_mod.pixel_grid(H, W)
    Q = forward.quad_form(px, py, mu2d, conic)
    w_geo = forward.poly_splat_wgeo(Q, K, keep)
    color = forward.color_from_dc(m.color_dc)
    img = arms.blend_A(w_geo, m.opacity_raw, color, torch.tensor(w_b),
                       torch.tensor(c_b, dtype=torch.float32)).reshape(H, W, 3)
    return (img.clamp(0, 1) * 255).round().to(torch.uint8).cpu().numpy()


def camera():
    r_v, t_v = cam_mod.look_at_opencv(eye=[0.0, 0.0, -3.0], target=[0, 0, 0], up=[0, 1, 0], dtype=torch.float32)
    focal = (H * 0.5) / math.tan(FOVY / 2)
    return r_v, t_v, focal  # torch tensors (project_ewa needs tensors)


def psnr(a, b):
    a = np.asarray(Image.open(a).convert("RGB"), np.float64)
    b = np.asarray(Image.open(b).convert("RGB"), np.float64)
    mse = np.mean((a - b) ** 2)
    return float("inf") if mse == 0 else 10 * np.log10(255.0 ** 2 / mse), mse


def main():
    r_v, t_v, focal = camera()
    if "--psnr" in sys.argv:
        p, mse = psnr(os.path.join(HERE, "routeb_oracle_black.png"), os.path.join(HERE, "routeb_rust.png"))
        print(f"PSNR(rust vs oracle, BLACK bg) = {p:.2f} dB (MSE={mse:.4f})")
        print("PASS — WSR method matches tt-splat render()" if p > 45 else "FAIL — method mismatch")
        return

    m = build_model()
    ply = os.path.join(HERE, "routeb.ply")
    n = plyio.save_ply(ply, m)
    print(f"wrote {ply} ({n} gaussians) via tt-splat spike/plyio.save_ply")

    # tt-splat's trained background is white, w_b = softplus(-3); the viewer default is black, 0.02.
    w_b_white = float(torch.nn.functional.softplus(torch.tensor(-3.0)))
    Image.fromarray(render_arm_a(m, r_v, t_v, focal, 0.02, [0, 0, 0]), "RGB").save(
        os.path.join(HERE, "routeb_oracle_black.png"))
    Image.fromarray(render_arm_a(m, r_v, t_v, focal, w_b_white, [1, 1, 1]), "RGB").save(
        os.path.join(HERE, "routeb_oracle_white.png"))
    print(f"oracle: BLACK bg (viewer default) and WHITE bg (tt-splat trained, w_b={w_b_white:.4f})")

    with open(os.path.join(HERE, "view_routeb.json"), "w") as f:
        json.dump(dict(width=W, height=H,
                       camera=dict(r_v=r_v.reshape(-1).tolist(), t_v=t_v.tolist(),
                                   fx=float(focal), fy=float(focal), cx=W / 2.0, cy=H / 2.0),
                       background=dict(w_b=0.02, c_b=[0, 0, 0])), f, indent=2)
    print("wrote view_routeb.json (black bg, matches viewer default)")


if __name__ == "__main__":
    main()
