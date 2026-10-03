# WizRust101-DB specifications

> WizRust101-DB is fully vibe coded.

## Purpose and observed data

The CLI creates a database of player-visible Wizard101 location names from extracted data in `raw/`. Canonical zone identifiers are not display names: names may differ entirely, and several interiors intentionally resolve through a map assigned to a parent area.

The verified global association resource is `raw/misc/Root/DoodleMapMap.xml`. Its serialized records contain a map resource path such as `|GUI|WorldData|Maps/AQ_Z01_Palace_of_Immortals.xml` followed by one or more canonical zone paths such as `Aquila/Interiors/AQ_Z01_Apollo_Room`. Each referenced map XML contains a localization key such as `Zone_00001124`. `raw/misc/Root/Locale/en-US/Zone.lang` maps that numeric key to the map title `Mount Olympus`. This chain was verified against Aquila, Azteca, and reference examples, including exceptional interior identifiers.

Candidate sources retain provenance. The WizardZone value must come only from the header field immediately following the canonical path field in `gamedata.bin`: path property marker `f8 63 69 81`, its `u16` byte length and path bytes, the four-byte header separator, then WizardZone property marker `6e ec f6 74`, its `u16` byte length, and the complete `WizardZone_...` value. Do not scan farther into the file for another WizardZone reference. Numeric suffixes resolve by their exact key in `raw/misc/Root/Locale/en-US/WizardZone.lang`; symbolic suffixes resolve by the same exact-key lookup. The locale table has both numeric and symbolic keys. An unlocalized symbolic suffix is retained in diagnostics but never prettified or emitted as a name. WizardZone values can identify a parent region, an instance, or an event phase, so a localized header value is a fallback rather than verified display evidence. A shared map candidate comes from the explicit DoodleMapMap-to-map-resource-to-`Zone_########`-to-`Zone.lang` chain and may likewise be broader or stale.

A Compass POI title can be assigned to a child zone only with the complete chain: parent POI trigger and matching POI volume -> localized `WizardCompassLocs_########` key -> named parent teleport/portal -> matching explicit child exit identifier. The child relationship must also be supported by raw zone/resource association; matching strings alone are insufficient. A unique proven Compass POI name wins. Generic POI and entry-text labels are supporting evidence only and never candidates for display names.

Selection follows this priority: (1) one proven Compass POI/child name; (2) another independently proven child-specific name when such a source is established; (3) one trustworthy shared-map name when no other usable name conflicts; (4) the exact localized value from the zone-header WizardZone field as an explicitly unverified fallback; (5) `Unknown`. This implementation currently has no separate proven child-specific source beyond the Compass POI chain. If WizardZone and shared-map candidates conflict without evidence deciding them, choose the header WizardZone only as `unverified_fallback`, and retain the conflict in diagnostics. Multiple conflicting proven POI candidates are not a verified name; use a usable header fallback if available, otherwise `Unknown`. Reject clearly malformed or unrelated map localization, including a label that identifies a different world/zone. Never prettify an identifier or use the reference as a name source. `ZoneLocName.lang` remains unsupported for zone display-name resolution absent a proven zone-specific relationship.

The comparison oracle defaults to `tmp/zones.json` and can be overridden with `--reference`. It is validation input only and never supplies generated names.

## Discovery and association

- Input defaults to `./raw`, overridable with `--input`.
- Recursively discover directories with a direct child `gamedata.bin`; do not follow symlinks. Sort discovered directories before processing.
- Extract the canonical key from the first readable slash-delimited path in `gamedata.bin`. Preserve every component, punctuation, and case.
- Parse `DoodleMapMap.xml` as a binary serialized resource. In the inspected serialization, a canonical path follows a `|GUI|WorldData|Maps/<asset>.xml` reference; associate the path with the nearest preceding map reference only when the byte gap is at most 64 bytes. This is a format-derived association heuristic, not a general XML parser. Do not infer relationships from similar filenames or `_ZNN` prefixes.
- Resolve each referenced map path below `raw/misc/GUI-WorldData/Maps/`. Use the first `Zone_########` key in the referenced map resource. In inspected multi-key maps, this leading key is the map title; later keys can identify other map content and do not override it.
- Parse the English `Zone.lang` UTF-16LE localization table into numeric-key/display-text pairs. Use the exact localized string, without prettifying or correcting it.
- Extract the canonical path from its header field, then read exactly the immediately following `6e ec f6 74` WizardZone property field. Require a complete `WizardZone_...` value inside that field's declared length. Accept numeric and symbolic suffixes as exact `WizardZone.lang` keys; never search later serialized fields when the header value is symbolic, malformed, or missing. Preserve the complete raw field value in diagnostics. Resolve only through the English `WizardZone.lang` table; an unmapped key is not a name.
- Keep WizardZone, shared-map, and proven Compass names as independent candidates. Select a unique proven Compass POI child name first. Otherwise select a unique uncontested usable shared-map name as verified. If candidates conflict, prefer a localized value from the actual zone-header WizardZone field only as `unverified_fallback`. Also use this fallback when no usable map name exists. If no localized usable candidate exists, emit `Unknown`. Never prettify symbolic keys or canonical paths.
- Derive Compass POI candidates from parent `triggers.xml`, parent `volumes.xml`, and child `triggers.xml`/`volumes.xml`. Require a POI trigger, its same-token POI volume, one localized `WizardCompassLocs_########` key in the trigger record, a parent teleport target with the exact same token, a child `Teleport-Exit<token>` identifier, and a shared explicit DoodleMap map association between parent and child. Select a unique proven POI child name ahead of map and WizardZone candidates. If proven POI names conflict, mark the zone ambiguous and omit it.
- A zone may have more than one explicitly associated map resource. If they resolve to different titles, retain the candidate set and mark it ambiguous unless raw relationships establish which title is current. A proven unique POI candidate still wins. Reject malformed/unrelated map labels; do not select a remaining WizardZone candidate merely because the map title was rejected or missing.
- Explicit links in `DoodleMapMap.xml` support parent-map inheritance; `_ZNN` similarity does not. Preserve extracted names even when they differ from the human-curated reference.

## Output and deterministic behavior

`out/zones.json` is a UTF-8, two-space-indented JSON object mapping every discovered canonical path to an extracted display string or the literal `Unknown` when no usable localized name exists:

```json
{
  "Aquila/AQ_Z00_Hub": "Garden of Hesperides"
}
```

Sort keys lexicographically and terminate with a newline. Create the output directory when missing. Never write beneath or modify `raw/`. No identifier-derived fallback is permitted.

Write a separate deterministic diagnostics sidecar beside the output as `<output-stem>.diagnostics.json` (for example, `out/zones.diagnostics.json`). For every discovered path it records candidates with source and provenance, the selected value/source/confidence/provenance, the exact raw WizardZone header value when present, candidate conflicts, and ambiguity/unresolved reasons. Confidence is `verified` for proven Compass or uncontested shared-map selections, `unverified_fallback` for header WizardZone selections, and `unknown` for `Unknown` output values. A validation reference never changes this record or its selection.

## Comparison and conflicts

`compare --reference PATH` writes both generated JSON and its diagnostics sidecar and reports candidate counts, selected counts by confidence, exact reference matches, mismatches, `Unknown`/unresolved reference zones, ambiguous candidates, and generated-only zones. `Unknown` counts as unresolved rather than as a name match or mismatch. Per-zone diagnostics identify each available source with provenance, selected source/confidence, candidate disagreements, and reference conflicts. A reference string is validation-only and never supplies a candidate or changes selection. The default reference is `tmp/zones.json`.

Malformed CLI arguments, unreadable required metadata, duplicate canonical keys with conflicting evidence, or output failures are fatal. Individual zones with incomplete or conflicting mapping evidence are reported in the diagnostics sidecar and still appear in the RPC output with a localized unverified fallback or `Unknown`.

## Tests and validation

Unit and fixture integration tests cover proven direct/child resolution, the Pit of the Noxii POI chain, numeric and symbolic header keys, ignoring later WizardZone references, WizardZone fallback provenance, conflicts, `Unknown`, invalid map labels, special interiors (Gorgon Cave, Vestrilund, Zeus Exalted Duel, Altar of Kings, Triton, Stonegaze's Antichamber), localization parsing, reference comparisons, and deterministic JSON. Run `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test`. Generate with `cargo run -- generate --input raw --output out/zones.json`; this also writes `out/zones.diagnostics.json`. Validate with `cargo run -- compare --input raw --reference tmp/zones.json`.

## Known limitations

- `DoodleMapMap.xml` and its map assets only resolve paths represented in that resource; all other canonical keys remain unresolved.
- WizardZone and Compass POI resources are not guaranteed to exist for every zone. A header WizardZone localization may be broad or phase-specific; fallback confidence is recorded explicitly and is not a claim that the value is the in-game display name.
- `tmp/zones.json` is a mutable validation fixture, not a source of names. Its contents and count can change independently of extracted game data.
- Extracted English is authoritative for this output; localized strings in other supported client languages are not selected automatically.
