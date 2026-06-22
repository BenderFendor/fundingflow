# Check

Run the fast local checks before handoff:

```bash
cargo fmt --all --check
cargo check --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
pnpm --dir apps/web lint
pnpm --dir apps/web typecheck
```
