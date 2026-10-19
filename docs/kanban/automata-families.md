# Automata families — executable scope

This item connects every existing classical and dynamical family to the native
workbench and adds Conway Life from the roadmap. Existing Lenia qualification
remains separate. Undefined custom rules are not substituted with invented ones.

| Group | Executable examples | State view |
| --- | --- | --- |
| Regular | Rule 30, Rule 110, NFA for a*b | Scrolling space-time grid; active states and input head |
| Context-free | a^n b^n, balanced parentheses, Koch L-system | Stack/input; generated word and turtle geometry |
| Context-sensitive | Linear-bounded a^n b^n c^n, n > 0 | Bounded tape, marking head and acceptance |
| Turing | Replace a with X, halt on blank, reject other symbols | Extensible tape, head and machine state |
| Cellular dynamics | Conway B3/S23, HPP lattice gas | Finite XY grid |
| Swarms | Boids, ant foraging | Bounded agents, headings, food, home and pheromone trails |
| Continuous dynamics | Previously qualified quantized Lenia | GPU density voxels and growth controls |

## Contracts

- Each CPU instance has independent state, ticks, pause, single-step and reset.
- Sessions spawn paused. Automatic execution uses ten transitions per second.
- Reset restores the deterministic seed or edited symbolic input and pauses.
- Clear and main-menu return release all CPU sessions. Up to eight coexist.
- Elementary cellular rules use nearest left/self/right cells and a centered
  single-cell seed. The most recent 64 generations remain visible.
- Life and HPP use 64×64 finite grids with dead/open boundaries. Life counts the
  eight XY neighbors. HPP transports direction bits before head-on scattering;
  particles can leave through the boundary.
- NFA states include epsilon closure. Pushdown languages reject invalid symbols
  and ordering. Empty a^n b^n and parentheses are accepted; LBA requires n > 0.
- The LBA never grows its input tape. The sample Turing transition table is not
  presented as a universal-machine implementation.
- Symbolic input is capped at 256 characters, Koch expansion at 4096 symbols,
  and symbolic execution at 10,000 transitions. Limits are distinct from rejection.
- Boids use a synchronous velocity snapshot, speed cap and wrapped arena edges.
  Ants use seeded randomness, clamped arena edges, persistent food locations,
  evaporating trails and explicit delivery counts. These are bounded examples,
  not claims of ecological calibration or fluid-dynamics accuracy.
- Lenia remains on its qualified GPU path. The new views run model updates on
  the CPU and draw through egui; no new GPU simulation backend is claimed.

## Acceptance evidence

The [qualification record](../verification/automata/README.md) tracks native
spawn/step/reset/pause/clear captures, model oracles, command routing tests,
strict workspace checks and structural/commit-cadence audits.

Dean's multi-energy rule remains undefined: the migration reference names
possible fading/energy behavior without an executable transition contract.
Its birth, survival, energy-transfer/decay rules and defaults are needed before
it can be implemented faithfully. Broader isotropic Lenia and a published
Orbium organism likewise retain their separate roadmap status.
