# ZzClawTerm fork notes

This branch carries ZzClawTerm's local source and CI changes to `gpui-kit` on
the `nyaterm` branch.

- Fork: <https://github.com/jackfahdin/gpui-kit>
- Upstream: <https://github.com/nyakang/gpui-kit>
- Branch: `nyaterm`
- Previous upstream revision: `9a0429b92d80f0dd9618fdcd721f69008d2ede7f`
- Merged upstream revision: `a4c92a92dfe87c194a2f2859468f6d80b936c2ae`
- Coherent Zed revision: `474dcb1d85d34c30aceef1900b24e4d27665eee8`

## Local changes

- All six Zed-derived workspace dependencies use `jackfahdin/zed` at the
  coherent revision above. This prevents duplicate GPUI source identities and
  type mismatches between the application, Kit, Base and Component.
- `script/check-gpui-pin.ts` requires those dependencies to use the owned fork,
  matching the source-of-truth policy.
- The native facade test command in `.github/workflows/nyaterm.yml` quotes
  `exports::` so the workflow is valid YAML.
- Upstream now coalesces rounded-quad fragments in the block-surface tests, so
  the earlier local test correction is no longer carried separately. Menu,
  notification and native-overlay implementations come from the upstream
  `nyaterm` branch without a second copy.
- `NYATERM.md` remains as the historical patch record from the original fork.

## Validation on Windows

```sh
cargo check -p gpui-kit -p gpui-component
cargo test -p gpui-base --test block_surface --features test-support
cargo test -p gpui-component menu::popup_menu --lib
```

The Kit and Component checks passed. All ten block-surface tests and all seven
selected popup-menu tests passed. The lockfile uses one coherent
`jackfahdin/zed` source. Bun is not installed on the validation host, so the pin
script is left for the branch workflow, where Bun is installed explicitly.
