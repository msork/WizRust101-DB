# WizRust101-DB

WizRust101-DB is fully vibe coded.

A Rust command-line tool that discovers zones in extracted Wizard101 data and generates a zone-name database for [WizRust101-RPC](https://github.com/msork/WizRust101-RPC). The project documents its discovery and naming rules in the [specification](docs/specification.md).

## Requirements

- Rust and Cargo
- An extracted Wizard101 data tree in `raw/` to generate the full database

Extracted game files and the reference dataset are not included in a clean checkout. The fixture tests can run without either.

## Build and test

```sh
git clone https://github.com/msork/WizRust101-DB.git
cd WizRust101-DB
cargo build --locked
cargo test --locked
```

## Generate the database

Place the extracted data under `raw/`, then run:

```sh
cargo run --locked -- generate --input raw --output out/zones.json
```

This writes two files:

- `out/zones.json` — a deterministic, alphabetically ordered object keyed by canonical zone paths. Each value has a `world` and `zone` string, for example `"Aquila/AQ_Z00_Hub": { "world": "Aquila", "zone": "Garden of Hesperides" }`. The RPC-facing contract is documented in the [specification](docs/specification.md).
- `out/zones.diagnostics.json` — zone candidates, world lookup key and evidence, selected source and confidence, raw header values, provenance, and conflicts for each discovered zone.

Every discovered path is included. Zone names supported by verified raw-data relationships are selected where available; a documented `unverified_fallback` may refer to a broader area or phase. If no zone candidate is supported, the final canonical path component is emitted unchanged as an explicitly unverified fallback. World names use exact `WorldNames.lang` localization first, then evidence-backed canonical aliases, then recognized content roots. For any remaining non-empty root, the final fallback preserves that root unchanged, except roots beginning with `Housing` become `House`. These rules apply only to entries that would otherwise resolve to `Unknown`; existing localized and alias results are unchanged. Diagnostics record fallback provenance. The generator does not modify `raw/` and does not use the reference dataset to select names.

On the inspected 3,346-zone raw snapshot, 26 zone names use the canonical-leaf fallback. World source counts and the 86-root evidence audit are documented in the [specification](docs/specification.md).

## Compare with a reference

If you have a reference JSON file, compare it with generated results using:

```sh
cargo run --locked -- compare --input raw --reference tmp/zones.json
```

`compare` generates the output and reports matches, mismatches, missing paths, and newly discovered paths. The reference is used only for validation.

## Development checks

```sh
cargo fmt --check
cargo clippy --locked -- -D warnings
cargo test --locked
```

## License

This project is licensed under the [MIT License](LICENSE).
