# WizRust101-DB specifications

> WizRust101-DB is fully vibe coded.

## Purpose and observed data

The CLI creates a database of player-visible Wizard101 location names from extracted data in `raw/`. Canonical zone identifiers are not display names: names may differ entirely, and several interiors intentionally resolve through a map assigned to a parent area.

The verified global association resource is `raw/misc/Root/DoodleMapMap.xml`. Its serialized records contain a map resource path such as `|GUI|WorldData|Maps/AQ_Z01_Palace_of_Immortals.xml` followed by one or more canonical zone paths such as `Aquila/Interiors/AQ_Z01_Apollo_Room`. Each referenced map XML contains a localization key such as `Zone_00001124`. `raw/misc/Root/Locale/en-US/Zone.lang` maps that numeric key to the map title `Mount Olympus`. This chain was verified against Aquila, Azteca, and reference examples, including exceptional interior identifiers.

Three independent candidate channels are retained with source provenance. A zone's own `gamedata.bin` header places its primary `WizardZone_########` reference directly after the canonical path; later WizardZone references elsewhere in serialized content can belong to other objects and are ignored. Resolve the header key through `raw/misc/Root/Locale/en-US/WizardZone.lang`. This is only a candidate: values can identify a parent region, an instance, or an event phase. A shared map candidate comes from the explicit DoodleMapMap-to-map-resource-to-`Zone_########`-to-`Zone.lang` chain and may likewise be broader or stale. A difference between these two candidates does not establish which describes the current location.

A Compass POI title can be assigned to a child zone only with the complete chain: parent POI trigger and matching POI volume -> localized `WizardCompassLocs_########` key -> named parent teleport/portal -> matching explicit child exit identifier. The child relationship must also be supported by raw zone/resource association; matching strings alone are insufficient. A unique proven Compass POI name wins. Generic POI and entry-text labels are supporting evidence only and never candidates for display names.

Selection is conservative. If there is no proven Compass POI chain, select a name only when raw relationships establish that candidate as the specific current/child location. Do not automatically select WizardZone when no map title resolves. Do not automatically select either WizardZone or shared map when they conflict. Reject clearly malformed or unrelated map localization, including a label that identifies a different world/zone. If available raw relationships do not disambiguate candidates, report the zone as unresolved or ambiguous and omit it from `zones.json`. Never prettify an identifier or use the reference as a name source. `ZoneLocName.lang` remains unsupported for zone display-name resolution absent a proven zone-specific relationship.

The comparison oracle defaults to `tmp/zones.json` and can be overridden with `--reference`. It is validation input only and never supplies generated names.

## Discovery and association

- Input defaults to `./raw`, overridable with `--input`.
- Recursively discover directories with a direct child `gamedata.bin`; do not follow symlinks. Sort discovered directories before processing.
- Extract the canonical key from the first readable slash-delimited path in `gamedata.bin`. Preserve every component, punctuation, and case.
- Parse `DoodleMapMap.xml` as a binary serialized resource. In the inspected serialization, a canonical path follows a `|GUI|WorldData|Maps/<asset>.xml` reference; associate the path with the nearest preceding map reference only when the byte gap is at most 64 bytes. This is a format-derived association heuristic, not a general XML parser. Do not infer relationships from similar filenames or `_ZNN` prefixes.
- Resolve each referenced map path below `raw/misc/GUI-WorldData/Maps/`. Use the first `Zone_########` key in the referenced map resource. In inspected multi-key maps, this leading key is the map title; later keys can identify other map content and do not override it.
- Parse the English `Zone.lang` UTF-16LE localization table into numeric-key/display-text pairs. Use the exact localized string, without prettifying or correcting it.
- Extract only the first `WizardZone_########` key following the canonical zone path in each `gamedata.bin` header; ignore later keys, which can belong to other serialized objects. Resolve it through English `WizardZone.lang` and retain it as an independent candidate. A WizardZone-only result is unsupported and stays unresolved. If one usable map candidate and one WizardZone candidate disagree, keep both in diagnostics and mark the zone unresolved unless other raw relationships establish which candidate describes the current/child location. A unique explicitly associated map candidate may be selected when there is no contradictory candidate; exact cross-source agreement is also safe to select.
- Derive Compass POI candidates from parent `triggers.xml`, parent `volumes.xml`, and child `triggers.xml`/`volumes.xml`. Require a POI trigger, its same-token POI volume, one localized `WizardCompassLocs_########` key in the trigger record, a parent teleport target with the exact same token, a child `Teleport-Exit<token>` identifier, and a shared explicit DoodleMap map association between parent and child. Select a unique proven POI child name ahead of map and WizardZone candidates. If proven POI names conflict, mark the zone ambiguous and omit it.
- A zone may have more than one explicitly associated map resource. If they resolve to different titles, retain the candidate set and mark it ambiguous unless raw relationships establish which title is current. A proven unique POI candidate still wins. Reject malformed/unrelated map labels; do not select a remaining WizardZone candidate merely because the map title was rejected or missing.
- Explicit links in `DoodleMapMap.xml` support parent-map inheritance; `_ZNN` similarity does not. Preserve extracted names even when they differ from the human-curated reference.

## Output and deterministic behavior

`out/zones.json` is a UTF-8, two-space-indented JSON object mapping only confidently resolved canonical paths to exact extracted display strings:

```json
{
  "Aquila/AQ_Z00_Hub": "Garden of Hesperides"
}
```

Sort keys lexicographically and terminate with a newline. Create the output directory when missing. Never write beneath or modify `raw/`. Unresolved and ambiguous keys are excluded and reported to stderr; no identifier-derived fallback is permitted.

## Comparison and conflicts

`compare --reference PATH` writes the generated JSON and reports candidate counts and selections by source, exact reference matches, mismatches, unresolved/reference-only zones, ambiguous candidates, and generated-only zones. Per-zone diagnostics identify each available source with provenance, selected source, candidate disagreements, and reference conflicts. A reference string is validation-only and never supplies a candidate or changes selection. The default reference is `tmp/zones.json`.

Malformed CLI arguments, unreadable required metadata, duplicate canonical keys with conflicting evidence, or output failures are fatal. Individual zones with incomplete or conflicting mapping evidence are reported and omitted rather than making the entire scan fail.

## Tests and validation

Unit and fixture integration tests cover proven direct/child resolution, the Pit of the Noxii POI chain, unsupported WizardZone-only candidates, conflicting WizardZone/map candidates, invalid unrelated map labels, special interiors (Gorgon Cave, Vestrilund, Zeus Exalted Duel, Altar of Kings, Triton), ambiguous mappings, missing mappings, localization parsing, reference conflict reporting, and deterministic JSON. Run `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test`. Generate with `cargo run -- generate --input raw --output out/zones.json`. Validate with `cargo run -- compare --input raw --reference tmp/zones.json`.

## Known limitations

- `DoodleMapMap.xml` and its map assets only resolve paths represented in that resource; all other canonical keys remain unresolved.
- WizardZone and Compass POI resources are not guaranteed to exist for every zone. WizardZone and map disagreements need a relationship that identifies the current/child location; neither source wins based on availability or source type alone.
- `tmp/zones.json` is a mutable validation fixture, not a source of names. Its contents and count can change independently of extracted game data.
- Extracted English is authoritative for this output; localized strings in other supported client languages are not selected automatically.
