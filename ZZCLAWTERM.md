# ZzClawTerm fork notes

This branch carries ZzClawTerm's local changes to `gpui-kit` on the `nyaterm` branch.

- Fork: <https://github.com/jackfahdin/gpui-kit>
- Upstream: <https://github.com/nyakang/gpui-kit>
- Branch: `nyaterm`
- Merged upstream revision: `9a0429b92d80f0dd9618fdcd721f69008d2ede7f`
- Coherent Zed revision: `468bc1eb8e31d8f802af766a4d2d74d2d08ea46a`

## Local changes

- All Zed-derived workspace dependencies use `jackfahdin/zed` at the coherent
  revision above. This prevents duplicate GPUI source identities and type
  mismatches between the application, Kit, Base and Component.
- Existing ZzClawTerm popup appearance and notification presentation patches
  are retained. Upstream menu behavior and tests are otherwise preserved.
- `NYATERM.md` remains as the historical patch record from the original fork.

## Validation on Windows

```sh
cargo check -p gpui-kit -p gpui-component
cargo test -p gpui-component menu::popup_menu --lib
```

The component checks passed and all six selected popup-menu tests passed.
`Cargo.lock` contains exactly one `gpui`, `gpui_platform` and `collections`
package, all sourced from the same `jackfahdin/zed` revision.

`cargo fmt --all -- --check` reports the existing upstream formatting difference
in `crates/base/examples/showcase/components/text_view.rs`; this merge does not
change that file. `bun script/check-gpui-pin.ts` was not run because Bun is not
installed on the validation host.
