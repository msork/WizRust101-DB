# WizRust101-DB

WizRust101-DB is fully vibe coded.

A Rust CLI that builds `out/zones.json` from extracted Wizard101 data in `raw/`. Read [the specification](docs/specification.md) for discovery, naming confidence, schema, errors, and known uncertainties.

```sh
cargo run -- generate --input raw --output out/zones.json
cargo run -- compare --input raw --reference ~/Downloads/zones.json
```

The generator reports independent `WizardZone`, shared map-title, and verified Compass POI candidates. A proven POI-to-child portal chain wins; otherwise a unique per-zone `WizardZone` title is used only when no shared map title resolves. Candidate disagreements and reference conflicts are reported, and the reference file never supplies names.

Validation commands:

```sh
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```
