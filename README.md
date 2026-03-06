# particle-demos-rs

Rust simulation and graphics demos with CPU and GPU implementations.

## Current demos

| Binary | Backend | Demo | Notes |
|---|---|---|---|
| `grav2mouse_cpu` | CPU (`raylib`, `rayon`) | particles gravitate to mouse | includes benchmark mode and CSV export |
| `grav2mouse_gpu` | GPU (`wgpu`) | particles gravitate to mouse | supports fade toggle and very high particle counts |
| `chain_cpu` | CPU (`raylib`, `rayon`) | each particle follows the next in a ring | fixed-speed chain rule |
| `chain_gpu` | GPU (`wgpu`) | chain-follow simulation on GPU | pan/zoom + background grid |

## Build

```bash
cargo build --release
```

## Run

```bash
cargo run --release --bin grav2mouse_cpu
cargo run --release --bin grav2mouse_gpu
cargo run --release --bin chain_cpu
cargo run --release --bin chain_gpu
```

## Controls

### `grav2mouse_cpu`
- move mouse: gravity target
- escape or close window: exit

### `grav2mouse_gpu`
- move mouse: gravity target (or center target if configured)
- `F`: toggle fade
- escape: exit

### `chain_cpu`
- escape or close window: exit

### `chain_gpu`
- mouse wheel: zoom
- right mouse drag: pan
- escape: exit

## CLI options

### `chain_cpu`
- `--seed <u64>` (alias: `--spawn-seed`)

Example:

```bash
cargo run --release --bin chain_cpu -- --seed 42
```

### `chain_gpu`
- `--seed <u64>` (alias: `--spawn-seed`)

Example:

```bash
cargo run --release --bin chain_gpu -- --seed 42
```

### `grav2mouse_cpu` benchmark mode
- `--benchmark`
- `--benchmark-step-only`
- `--benchmark-draw-only`
- `--frames <usize>`
- `--warmup-frames <usize>`
- `--output <path>`
- `--step-kernel zip|chunked`
- `--chunk-size <usize>`
- `--render-mode rgba|bitset`
- `--fused-step-draw`

Example:

```bash
cargo run --release --bin grav2mouse_cpu -- \
  --benchmark \
  --frames 1200 \
  --warmup-frames 300 \
  --step-kernel chunked \
  --chunk-size 16384 \
  --render-mode bitset \
  --output perf_samples.csv
```

CSV includes per-frame `step_ms`, `draw_ms`, `total_ms`, and ratios.

## Config files

Primary tuning points are compile-time constants in:

- `src/cpu/common/config.rs`
- `src/gpu/particle_sim/config.rs`
- `src/gpu/chain/config.rs`

Examples of tunables:
- particle counts
- gravity strength and fixed speed
- bounds enable/disable
- draw budget, fade amount, point alpha
- workgroup size and present mode preferences

## Layout

```text
src/
  cpu/
    common/
    particle_sim/
    chain/
  gpu/
    common/
    particle_sim/
    chain/
    shaders/
docs/
  DEMO_IDEAS.md
```

## Notes

- The package name in `Cargo.toml` is currently `gravsim-rust`; the repository/project label is `particle-demos-rs`.
- Very high GPU particle counts can hit adapter limits (buffer sizes, dispatch dimensions).
- A demo backlog for future additions is in `docs/DEMO_IDEAS.md`.
