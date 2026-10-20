# Authored rules, composition and energy

## Acceptance

- General binary, radius-one XY Moore rules use portable version-1 JSON. Nine
  bits enumerate y=-1..1 then x=-1..1, including the centre at bit 4. Ordered
  masked transitions use the first matching outcome; the explicit default is
  used otherwise. At most 512 transitions, with bounded names and masks.
  These express all binary 3x3 local rules, including directional/non-totalistic
  rules. They do not execute code. The inspector edits, validates, imports and
  exports definitions; projects and named specimens retain exact definitions.
- Composition evaluates two authored rules from the same complete prior grid.
  Union, intersection and exclusive-or combine their binary outputs, then the
  combined grid feeds both rules on the next tick. Both layers necessarily share
  dimensions, boundaries, seed and clock. This is coupled rule composition,
  not two independently evolving channels or alpha blending. Existing paired
  channel comparison remains available for independent experiments.
- An optional two-pool energy model gates the composed or single-rule result.
  Its initial editable contract is specified separately before implementation.
  All simulation paths are CPU; existing GPU Lenia stays distinct.
- Retire the optional sprite-renderer milestone from v0-0-2: no separate sprite
  backend is planned. Existing grid analysis views remain, while symbolic
  automata continue to use voxel entities. Revisit only for a concrete need.
- Old projects keep their previous behavior. Invalid definitions/state are
  rejected before replacing a running session. Exact save/load continuation,
  reset, paint/undo, native operation, and existing families are verified.

## Delivery

Use the existing governed roots and limits without exceptions. Keep source
checkpoints at or below 100 changed lines. Run full `make check`, build and native
monitor-1 windowed verification before merging to local main. Record actual
results and remaining limitations; prior hosted results do not qualify new code.

## Two-pool starting contract (v1)

This is a new configurable starting model, not a claimed reproduction of an
unprovided historical rule. Each cell, alive or dead, owns two integer energy
pools. Default capacities are [64,32], uniform initial energies [32,16], diffusion
[250,100] parts per thousand, decay [1,1], recharge [3,2], birth costs [8,4] and
survival costs [2,1]. All quantities and capacities are editable; capacity is
1..4096, diffusion 0..1000, other amounts 0..capacity.

Each tick first computes the ordinary binary rule/composition candidate from the
complete previous occupancy. Independently for each pool, sample the four
cardinal XY neighbors from the previous energy field (periodic wrap or zero
outside dead edges). Calculate floor((self*(4000-4*d)+neighbor_sum*d)/4000),
subtract decay saturating at zero, add recharge, then clamp to capacity. A
candidate birth or survival succeeds only if both pools cover its respective
costs; deduct those costs only on success. Otherwise the cell dies. Remaining
energy persists even in dead cells. There is no death refund or cross-pool
conversion. Recharge and decay deliberately make this an open energy system.

Any energy-parameter change requires reset. Reset and painting initialize both
pools uniformly; undo restores exact prior pools. Projects/specimens retain both
fields and definitions; restore rejects mismatched lengths or out-of-capacity
values. Occupancy remains the stage/comparison quantity; inspector reports both
energy totals separately. Tests cover the arithmetic, boundaries, exhaustion,
continuation, configuration refusal and editing history.
