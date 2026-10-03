# WizRust101-DB

WizRust101-DB is fully vibe coded.

A Rust CLI that builds `out/zones.json` from extracted Wizard101 data in `raw/`. Read [the specification](docs/specification.md) for discovery, naming confidence, schema, errors, and known uncertainties.

```sh
cargo run -- generate --input raw --output out/zones.json
cargo run -- compare --input raw --reference tmp/zones.json
```

`out/zones.json` is a simple canonical-path -> best-available-name object for RPC consumers. Verified names use proven raw relationships, including a complete Compass POI-to-child chain, direct localized zone/housing header fields, or an uncontested shared-map association. An `unverified_fallback` uses the localized WizardZone value from the exact zone-header field; it may name a broader area or phase. `Unknown` means no usable localized name was found.

`out/zones.diagnostics.json` records candidates, selected source and confidence, raw header values, provenance, and conflicts. `tmp/zones.json` is validation-only: it reports exact matches, mismatches, unresolved paths, and new paths, and never supplies names.

Validation commands:

```sh
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```
