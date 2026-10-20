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
