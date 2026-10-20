"""Independent FFT oracle for the pinned published polynomial Orbium model.

Run from any directory; emits numerical fixtures and a 1000-tick phenotype ledger.
Requires NumPy. Source/model provenance is in assets/specimens/README.md.
"""
from pathlib import Path
import hashlib
import json
import numpy as np

ROOT = Path(__file__).resolve().parents[1]


def generate():
    cells = np.array(json.loads((ROOT / "assets/specimens/orbium.json").read_text()))
    world = np.zeros((64, 64))
    world[22:42, 22:42] = cells
    coords = (np.arange(64) - 32) / 13
    radius = np.sqrt(coords[:, None] ** 2 + coords[None, :] ** 2)
    kernel = (radius < 1) * (4 * np.minimum(radius, 1) * (1 - np.minimum(radius, 1))) ** 4
    kernel /= kernel.sum()
    spectrum = np.fft.fft2(np.fft.ifftshift(kernel))
    fixtures = {}
    ledger = []
    for tick in range(1001):
        if tick in (0, 1, 10, 100):
            fixtures[str(tick)] = world.ravel().tolist()
        if tick % 100 == 0:
            # Circular centroid follows a moving organism through the periodic seam.
            phase = np.exp(2j * np.pi * np.arange(64) / 64)
            centroid = [float(np.angle(np.sum(world.sum(axis=a) * phase)) * 64 / (2*np.pi)) for a in (0, 1)]
            ledger.append(dict(tick=tick, mass=float(world.sum()), centroid=centroid))
        potential = np.fft.ifft2(np.fft.fft2(world) * spectrum).real
        growth = np.maximum(0, 1 - (potential - .15)**2 / (.014**2 * 9))**4 * 2 - 1
        world = np.clip(world + .1 * growth, 0, 1)
    assert all(20 < row["mass"] < 150 for row in ledger), ledger
    output = ROOT / "assets/specimens/orbium-reference.json"
    output.write_text(json.dumps(dict(states=fixtures, phenotype=ledger), separators=(",", ":")) + "\n")
    print(json.dumps(dict(fixture_sha256=hashlib.sha256(output.read_bytes()).hexdigest(), phenotype=ledger), indent=2))


if __name__ == "__main__":
    generate()
