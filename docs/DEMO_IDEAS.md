# Demo Ideas Backlog

Future simulation and graphics demos that fit this repo style:
- measurable performance
- high visual payoff
- simple, inspectable implementation

## 1) Splatter / Paint Field (Particle + Grid Hybrid)
- Goal: high-count particles that "splat" pigment into a texture field and then advect.
- Core model:
  - particles carry color + mass
  - splat pass writes to accumulation textures
  - advection/decay pass evolves the painted field
  - optional velocity field for smear/drip effects
- Why:
  - visually distinct from the current white-point demos
  - lets us test compute + render bandwidth hard
  - can be extended to smoke/ink behavior later
- Metrics:
  - particles splatted/frame
  - texture resolution
  - compute ms (splat + advect + decay)
  - render ms

## 2) Falling Sand / Cellular Automata
- Goal: powder, liquid, gas materials with interactions and wind.
- Core model: CA update rules on a 2D lattice, chunked updates.
- Why: great emergent behavior with straightforward rules.
- Metrics:
  - grid cells updated/frame
  - material rule cost
  - bandwidth utilization

## 3) SPH Fluid (2D)
- Goal: fluid blobs, splashes, incompressibility behavior.
- Core model: Smoothed Particle Hydrodynamics with neighbor search.
- Why: direct path to "water sim" behavior.
- Metrics:
  - neighbor search cost
  - step ms with/without spatial hash
  - stability at different smoothing radii

## 4) Grid Fluid (Eulerian / Stable Fluids)
- Goal: smoke/wind velocity field simulation on a grid.
- Core model: advection + diffusion + pressure projection.
- Why: field-based sim complements particle-based demos.
- Metrics:
  - grid resolution
  - pressure iterations/frame
  - frame time split by pass

## 5) Boids / Swarm + Obstacles
- Goal: flocking with separation/cohesion/alignment and obstacle avoidance.
- Core model: local-neighbor steering with spatial partitioning.
- Why: strong benchmark for neighbor query performance.
- Metrics:
  - neighbors queried/agent
  - step ms vs. agent count
  - obstacle rule cost

## 6) Wave Equation / Ripple Field
- Goal: interactive ripple propagation and interference patterns.
- Core model: 2D discrete wave equation on a texture/grid.
- Why: low implementation cost, immediate visual feedback.
- Metrics:
  - resolution
  - updates/sec
  - stability vs. damping parameters

## 7) Cloth / Rope Constraints (Verlet or PBD)
- Goal: cloth sheet and ropes with pin constraints and collisions.
- Core model: positional constraints solved iteratively.
- Why: good testbed for constraint solver scaling.
- Metrics:
  - points/constraints
  - solver iterations/frame
  - ms/step

## 8) Galactic Collision (N-body)
- Goal: two rotating systems with central massive bodies and particle stars.
- Core model: Newtonian gravity + softening term.
- Why: high visual payoff and clear compute stress test.
- Metrics:
  - particles simulated
  - ms/step
  - FPS
  - energy drift over time

## 9) Barnes-Hut N-body Approximation
- Goal: scale particle gravity far beyond O(n^2) interactions.
- Core model: quadtree (2D) or octree (3D) aggregate-force approximation.
- Why: practical path to bigger astrophysical scenes.
- Metrics:
  - build tree ms
  - force eval ms
  - error vs. exact force

## 10) Reaction-Diffusion (Gray-Scott)
- Goal: procedural pattern growth and morphing.
- Core model: PDE update on 2-channel field.
- Why: predictable workload, visually rich output.
- Metrics:
  - resolution
  - passes/frame
  - update ms

## 11) SDF Raymarcher (Fullscreen)
- Goal: render 3D implicit geometry without mesh pipeline complexity.
- Core model: signed distance fields + raymarch in shader.
- Why: major visual upgrade with minimal scene infrastructure.
- Metrics:
  - steps/pixel
  - frame ms at 720p/1080p
  - shading mode cost (normal/AO/shadow)

## 12) Progressive GPU Path Tracer (Spheres First)
- Goal: physically based rendering demo with progressive accumulation.
- Core model: stochastic ray paths + accumulation buffer.
- Why: canonical "compute-heavy graphics" demo.
- Metrics:
  - samples/pixel/sec
  - convergence rate
  - denoise/no-denoise frame cost

## 13) Voxel Raycaster / Raymarcher
- Goal: dense or sparse voxel scene rendering.
- Core model: voxel traversal (DDA) or distance-field raymarch.
- Why: gets voxel visuals without full mesh extraction pipeline.
- Metrics:
  - voxels traversed/ray
  - frame ms vs. scene density
  - memory footprint

## 14) Rigid Body Broadphase + Narrowphase (2D)
- Goal: many-body collision sandbox.
- Core model: uniform grid broadphase + impulse resolution.
- Why: useful core for future game-like simulations.
- Metrics:
  - pair candidates/frame
  - contacts solved/frame
  - ms in broadphase vs. solver

## 15) Terrain Erosion (Hydraulic/Thermal)
- Goal: heightmap evolution with rainfall and sediment transport.
- Core model: iterative erosion/deposition over grid.
- Why: mixes simulation and rendering nicely.
- Metrics:
  - grid resolution
  - iterations/frame
  - erosion pass ms

## 16) Acoustic Wave Transmission Through Materials (2D FDTD)
- Goal: simulate sound traveling through mixed materials and measure received signal changes.
- Core model:
  - finite-difference wave update on pressure/velocity fields
  - per-cell material properties (impedance, absorption, speed)
  - source emitters + receiver probes
- Why:
  - directly matches "sound through objects/materials" behavior
  - gives both visual output and measurable filtering behavior
- Metrics:
  - grid resolution
  - solver steps/frame
  - source-to-probe transfer response (amplitude/frequency)
  - compute ms

## 17) Radio Transmission / Coverage + Multipath
- Goal: approximate RF propagation in 2D with obstacles and reflections.
- Core model:
  - path-loss baseline
  - optional reflection bounces from boundaries/objects
  - receiver map and signal-to-noise proxy
- Why:
  - useful "field simulation" demo with practical interpretation
  - supports real-time exploration by moving TX/RX/obstacles
- Metrics:
  - rays/samples per frame
  - coverage map update ms
  - SNR/received power error vs reference mode

## 18) 2D Light Transport in Waveguides / Light Pipes
- Goal: visualize guided light through channels, bends, and couplers.
- Core model:
  - geometric optics approximation (ray or beam packets)
  - index-of-refraction map and boundary reflection/transmission
  - optional ring resonator recirculation behavior
- Why:
  - high visual payoff and pairs with your light-pipe/ring idea
  - can start simple and scale toward wave models later
- Metrics:
  - rays propagated/frame
  - coupling efficiency at outputs
  - cavity/ring energy vs time

## 19) Mesh Force Propagation / Stress Waves
- Goal: apply impulses to a mesh and observe force/stress transmission.
- Core model options:
  - mass-spring network
  - position-based constraints
  - linear FEM on triangle mesh
- Why:
  - strong bridge between physics and graphics
  - useful for material and structural behavior demos
- Metrics:
  - vertices/elements
  - solver iterations/frame
  - max strain / stress distribution
  - compute ms

## 20) GPU Skeletal Animation Sandbox
- Goal: skin many animated instances using GPU-side bone transforms.
- Core model:
  - bone matrices in storage/uniform buffers
  - GPU skinning pass (vertex or compute)
  - animation blending and instancing
- Why:
  - good benchmark for transform-heavy GPU workflows
  - useful infrastructure for later character/creature demos
- Metrics:
  - bones per rig
  - rigs/instances on screen
  - skinning ms
  - total frame ms

## 21) Fluvial Geomorphology
- Goal: terrain evolution from river network formation and sediment transport.
- Core model:
  - rainfall/runoff routing
  - erosion/deposition coupled to flow
  - drainage basin and channel growth dynamics
- Why:
  - scientifically rich and visually meaningful terrain generation
  - complements existing particle and grid sim tracks
- Metrics:
  - terrain resolution
  - simulated years/iteration scale
  - drainage network statistics
  - erosion/deposition throughput

## 22) Orographic Wind + Water Erosion Terrain Generation
- Goal: generate terrain by coupling mountain wind fields with precipitation and erosion.
- Core model:
  - wind advection over heightmap
  - precipitation model from uplift/cooling
  - hydraulic + thermal erosion passes
- Why:
  - directly matches mountain wind/water terrain generation request
  - produces plausible large-scale terrain structure
- Metrics:
  - grid resolution
  - pass timings (wind/precip/erosion)
  - slope and roughness distributions
  - total generation time

## Suggested Build Order
1. Splatter / paint field (fast visual win, fits existing particle+texture path)
2. Falling sand CA (different data model, high payoff)
3. SPH fluid (heavier but valuable)
4. SDF raymarcher (graphics showcase without full raster pipeline complexity)
5. Progressive path tracer or voxel raycaster
6. Acoustic transmission through materials (signal + physics crossover)
7. Mesh force propagation (structural physics track)
8. Fluvial/orographic terrain generation (long-horizon simulation track)

## Common Infra To Add Once
- Unified benchmark mode for GPU demos (`--frames`, CSV output)
- Runtime toggles for vsync/present mode, fade, trails, and overlays
- Shared camera controls (pan/zoom/grid) where meaningful
- Consistent stats overlay:
  - compute ms
  - render ms
  - total frame ms
  - particles/cells/rays per second
