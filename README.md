# WizRust101-DB

WizRust101-DB is fully vibe coded.

A Rust CLI that builds `out/zones.json` from extracted Wizard101 data in `raw/`. Read [the specification](docs/specification.md) for discovery, naming confidence, schema, errors, and known uncertainties.

## Clean checkout

From a clean checkout, build and run the fixture tests with the locked Rust dependencies:

```sh
git clone https://github.com/msork/WizRust101-DB.git
cd WizRust101-DB
cargo build --locked
cargo test --locked
```

The extracted game files are not committed. Put the extracted tree in `raw/` before generating the full database.

## Generate and consume

```sh
cargo run --locked -- generate --input raw --output out/zones.json
cargo run --locked -- compare --input raw --reference tmp/zones.json
```

Generation is deterministic and emits every discovered canonical path, including paths whose value is `Unknown`. `out/zones.json` is a stable path -> string JSON object that WizRust101-RPC can consume directly as its zone-name database. Verified names use proven raw relationships, including a complete Compass POI-to-child chain, direct localized zone/housing header fields, or an uncontested shared-map association. An `unverified_fallback` uses the localized WizardZone value from the exact zone-header field; it may name a broader area or phase. `Unknown` means no usable localized name was found.

The generator also writes `out/zones.diagnostics.json`, which records candidates, selected source and confidence, raw header values, provenance, and conflicts. This sidecar does not change the RPC JSON schema or its selected names. The `generate` command does not read `tmp/zones.json`; only `compare` reads an oracle. `tmp/zones.json` is validation-only and never supplies names.

Validation commands:

```sh
cargo fmt --check
cargo clippy --locked -- -D warnings
cargo test --locked
```
