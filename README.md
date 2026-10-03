# WizRust101-DB

WizRust101-DB is fully vibe coded.

A Rust CLI that builds `out/zones.json` from extracted Wizard101 data in `raw/`. Read [the specification](docs/specification.md) for discovery, naming confidence, schema, errors, and known uncertainties.

```sh
cargo run -- generate --input raw --output out/zones.json
cargo run -- compare --input raw --reference ~/Downloads/zones.json
```

The generator follows explicit `DoodleMapMap.xml` links to map resources, then resolves each map's `Zone_########` key through the English `Zone.lang` table. It excludes zones whose names cannot be resolved from this chain. The reference file is validation data only: extracted names stay unchanged when the reference differs.

Validation commands:

```sh
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```
