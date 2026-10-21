# Authored rules, composition and energy

Run `cargo run --all-features -- --rules-demo` for three paused examples:
a directional copy-west rule, copy-west composed with Life, and the composed
rule gated by two energy pools. The window opens on monitor 1. These simulations
run on the CPU. The normal library's Life channel supports the same controls.

## Author a rule

Select a Life channel and open **parameters → Authored rules and layers**.
**Load current draft** converts the current Life birth/survival masks into an
explicit definition; **Copy-west example** gives a compact directional example.
Edit the JSON, choose **Validate and prepare rule**, then **Apply live** (keep
current occupancy) or **Apply and reset** (return to the configured seed).
Invalid text leaves the prepared/live rule unchanged. **Use birth/survival masks**
prepares a return to the built-in rule; press Apply to activate that change.

Each rule has `version: 1`, a name, `default_alive`, and ordered `transitions`.
A transition contains a nine-bit `mask`, a `pattern` contained in that mask,
and an `alive` result. The first transition whose masked neighborhood equals
its pattern wins. Otherwise the default wins. A zero mask matches everything.

Bits are numbered by coordinate, independently of screen orientation:

| y \\ x | -1 | 0 | +1 |
|---|---:|---:|---:|
| -1 | 0 | 1 | 2 |
| 0 | 3 | 4 (self) | 5 |
| +1 | 6 | 7 | 8 |

For example, this program copies the previous west neighbor, moving occupancy
east by one cell per tick:

```json
{
  "primary": {
    "version": 1,
    "name": "Copy west",
    "default_alive": false,
    "transitions": [{"mask": 8, "pattern": 8, "alive": true}]
  },
  "secondary": null
}
```

There are 512 possible binary neighborhoods. Exact masks of 511 can express any
binary radius-one XY rule; wildcard masks shorten equivalent cases. Definitions
are limited to 512 transitions per rule, 64-byte names, and 128 KiB per file.
Unknown fields, versions and invalid masks are rejected. Rule definitions do not
contain scripts, paths or executable code.

## Compose two rules

With valid text loaded, **Add copy-west layer** creates an editable second rule.
Edit its JSON as needed, select **Union**, **Intersection**, or **XOR**, validate,
and Apply. Both rules read the same prior occupancy. The combined output becomes
the next shared grid. Both layers share dimensions, boundary, seed and clock.
Use the existing comparison drawer for two independently evolving channels.

## Use energy

Open **Two-pool energy**, enable it, adjust either pool and **Apply and reset**.
Each cell owns two integer pools even while dead. Each tick diffuses energy,
subtracts decay, adds recharge, caps the pools, then gates the rule's proposed
birth/survival. Both costs must be affordable. Only successful cells pay them.
The readout shows separate A/B totals; stage images and comparison measure binary
occupancy. [Exact arithmetic and defaults](../kanban/rules-composition.md) define
this editable starting model. It is not a historical rule reproduction.

Energy changes require reset. Painting creates a new seed and resets both pools
to their initial values. Undo/redo restores the previous pools as well as occupancy.
Project/specimen save-load retains exact current pools, rules and clock state.

## Share and preserve

Enter a path and **Export new file** to share the JSON rule program. Existing
files are never replaced. **Import rule** loads validated text; validate/prepare
and Apply to activate it. Rule files contain definitions only; energy settings,
world dimensions, seed and current state travel in projects and **My specimens**.
Save a paused specimen to share an exact reproducible experiment. PNG/GIF exports
include the full checkpoint alongside occupancy images. Comparison CSV provenance
includes the authored program and energy settings.

The optional sprite renderer is deferred. Existing cellular analysis views and
symbolic voxel entities continue to provide the workbench's presentation.
