# AGENTS.md

## Coding Standards

- Keep source files between **300 and 500 lines max** whenever practical.
- Split large features into separate modules/files before a file grows beyond this range.
- Organize code by feature and responsibility, not by dumping unrelated logic together.
- Keep functions focused and short; each function should do one clear thing.
- Avoid excessive nesting (`if`, `match`, loops inside loops). Prefer early returns and simple control flow.
- Use a **C+-style** approach:
  - procedural-first design,
  - explicit data flow,
  - minimal hidden magic,
  - low abstraction unless it clearly improves readability or performance.
- Prefer straightforward, debuggable code over clever code.

## Design Expectations

- Separate simulation, rendering, input, and benchmarking concerns.
- Keep performance-sensitive paths easy to inspect and profile.
- Keep public APIs small and intentional.

