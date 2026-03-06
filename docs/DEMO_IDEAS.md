# Demo Ideas Backlog

This is a shortlist of future sim demos that fit this project style (fast visual prototypes with measurable performance).

## 1) Galactic Collision (N-body)
- Goal: two rotating galaxies with central massive bodies and particle stars.
- Core model: Newtonian gravity, softening term, optional Barnes-Hut / uniform-grid approximation.
- Why: visually strong and a clear stress test for compute + draw at high particle counts.
- Metrics:
  - particles simulated
  - ms/step
  - FPS
  - energy drift over time

## 2) Black Hole Accretion Disk
- Goal: central attractor with orbiting particles and infall.
- Core model: inverse-square gravity + drag + optional relativistic-ish visual tweaks.
- Why: simpler than full galaxy collision but still high impact visually.
- Metrics:
  - stable orbit count
  - average radial infall speed
  - particles/sec

## 3) SPH Fluid (2D)
- Goal: fluid blobs, splashes, incompressibility behavior.
- Core model: Smoothed Particle Hydrodynamics (density/pressure/viscosity kernels).
- Why: direct path to "water sim" and richer interaction.
- Metrics:
  - neighbor search cost
  - step ms with/without spatial hash
  - pressure solve stability

## 4) Grid Fluid (Eulerian)
- Goal: smoke/wind flow with diffusion/advection.
- Core model: staggered grid, semi-Lagrangian advection, projection solve.
- Why: complements particle SPH and enables "wind field" effects.
- Metrics:
  - grid resolution
  - pressure iterations/frame
  - frame time split by pass

## 5) Falling Sand / Cellular Automata
- Goal: powder, liquid, gas materials with interactions and wind.
- Core model: CA update rules on a 2D lattice, chunked updates.
- Why: very good "rules + emergent behavior" demo.
- Metrics:
  - grid cells updated/frame
  - material rule cost
  - bandwidth utilization

## 6) Boids / Swarm + Obstacles
- Goal: flocking with avoidance, cohesion, separation, plus attractors.
- Core model: local-neighbor steering with uniform grid.
- Why: good benchmark for neighbor search and interaction rules.
- Metrics:
  - neighbors queried/agent
  - step ms vs agent count
  - collision/obstacle rule cost

## 7) Reaction-Diffusion (Gray-Scott)
- Goal: procedural pattern growth and morphing.
- Core model: PDE on a 2D grid, multi-pass compute.
- Why: low complexity, visually strong, predictable workload.
- Metrics:
  - resolution
  - passes per frame
  - update ms

## Suggested Build Order
1. Galactic collision (high visual payoff, reuses current particle path)
2. Falling sand CA (different data model, broadens architecture)
3. SPH fluid (harder, but unlocks water-style demos)
4. Grid fluid / smoke (field-based path)

## Common Infra To Add Once
- Unified benchmark mode for GPU demos (`--frames`, CSV output)
- Runtime toggles for draw/fade/present mode
- Shared camera controls (pan/zoom/grid) for all GPU demos
- Stats overlay breakdown:
  - compute ms
  - render ms
  - total frame ms
