# English control names

`english.rs` is an offline snapshot of **1,659 nonempty English localization
entries** from War Thunder's `controls.csv` and `inf.csv`. It retains direct
`hotkeys/<identifier>` and `controls/<identifier>` keys, excluding nested help
paths and empty translations. Not every localization entry is a bindable action.

## Source and attribution

The strings originate from **Gaijin Entertainment's War Thunder** assets, exposed
by the community [War-Thunder-Datamine repository](https://github.com/gszabi99/War-Thunder-Datamine).
They are game data, not original translations by this project's contributors;
the project's MIT license does not grant rights to Gaijin's content. This viewer
is not affiliated with or endorsed by Gaijin Entertainment.

Pinned upstream revision: `aa326f10c6ba7d523647572368ead2e4d51ffb4d`.

| Source path in that revision | SHA-256 |
| --- | --- |
| `lang.vromfs.bin_u/lang/controls.csv` | `96ef5cb23fdf3c34b7f720ff2016326f9f6327374546eacd229aa7a8011c9f18` |
| `lang.vromfs.bin_u/lang/inf.csv` | `d14a8b1afd6282089af68e62e1e48c1ccb661f6572e5b2e89499765eda1cb37b` |

## Lookup rules and limitations

- Hotkey entries use the exact `hotkeys/` key; axis entries use `controls/`.
  These labels are not interchangeable: `gm_zoom` is “Enable ground vehicle zoom
  axis” as a hotkey and “Zoom axis (Ground)” as an axis.
- No case folding, guessed aliases, or inferred vehicle categories are used.
- Missing or blank translations use the viewer's identifier-based fallback,
  explicitly identified as such by the UI. This is a versioned snapshot, not a
  guarantee that all past or future exported identifiers have a game label.
- The bundled export has labels for **81 of 97 entries**, including all five
  axes. The other 16 entries use ten distinct `human_uav_*_rangeMin/rangeMax`
  identifiers absent from these source catalogues. Their labels are not invented
  by borrowing other vehicles' controls.
- The parser, original action IDs, bindings, and source files are never modified.
  No network or Python runtime is needed to build or run the viewer.

## Reproduce the snapshot

From the repository root, with `curl` and Python 3.10+ installed:

```sh
curl --fail --location 'https://raw.githubusercontent.com/gszabi99/War-Thunder-Datamine/aa326f10c6ba7d523647572368ead2e4d51ffb4d/lang.vromfs.bin_u/lang/controls.csv' -o target/wt-controls.csv
curl --fail --location 'https://raw.githubusercontent.com/gszabi99/War-Thunder-Datamine/aa326f10c6ba7d523647572368ead2e4d51ffb4d/lang.vromfs.bin_u/lang/inf.csv' -o target/wt-inf.csv
python3 scripts/generate_control_names.py target/wt-controls.csv target/wt-inf.csv
cargo fmt --all -- --check
cargo test --locked
```

Create `target/` first if the project has never been built. The generator uses
Python's standard CSV reader (semicolon separator, quoted fields, embedded
newlines), locates the English column by its header, checks source hashes, rejects
duplicate keys, and sorts the Rust table for binary search. It performs no network
requests itself. Regenerating from the pinned inputs is byte-for-byte deterministic.

For an intentional update, choose a new upstream revision, review the source
diffs, update the revision and hashes in the generator and this document, then
regenerate and recheck coverage. Do not silently download `master` during a build.
