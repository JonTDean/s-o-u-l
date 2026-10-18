# Repository workflow

- Commit at every 100 changed source lines (additions plus deletions), or sooner
  when a coherent change is ready. Do not accumulate changes past that checkpoint.
- Follow the structural rules in `structural-governance.toml` and `README.md`.
- Run checks appropriate to each change; use `make check` for completed work items.
- Keep application previews windowed on monitor 1; monitor 2 is reserved for the user.
