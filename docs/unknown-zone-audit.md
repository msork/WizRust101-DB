# Unknown-zone audit for df4c243

> WizRust101-DB is fully vibe coded.

This audit started with all 53 paths emitted as `Unknown` by commit `df4c243`. The validation file `tmp/zones.json` was used only after examining raw evidence; it did not supply any names.

## Results

Two raw-data patterns safely recover 27 of the 53 paths:

- **18 direct Zone/Housing headers:** the property immediately following the canonical path contains `Zone_########` or `Housing_########`, and the exact key resolves through `Zone.lang` or `Housing.lang`. The raw header ties the localization key to the current canonical path.
- **9 symbolic WizardZone headers:** the immediate header field contains `WizardZone_TheCommons`, `WizardZone_Ravenwood`, or `WizardZone_UnicornWay`. The corresponding `WizardZone.lang` records include a description row between the symbolic key and localized value. Supporting this documented row shape recovers the existing `The Commons`, `Ravenwood`, and `Unicorn Way` localized values as unverified fallbacks.

The remaining 26 paths stay `Unknown`. Their breakdown is:

| Cause | Count | Outcome |
| --- | ---: | --- |
| No usable current-zone localization reference or supported map/POI relationship | 22 | Keep `Unknown` |
| Localized field belongs to another semantic channel (sigil or quest) | 2 | Keep `Unknown` |
| Numeric WizardZone key resolves to whitespace | 1 | Keep `Unknown` |
| Development/test title (`TEST-Interior`) | 1 | Reject and keep `Unknown` |
| Missing canonical raw zone resource | 0 | — |
| Ambiguous candidate evidence | 0 | — |

## Evidence and limits

The immediate header property uses the serialized property marker `6e ec f6 74`, the same layout as the WizardZone field: canonical path marker `f8 63 69 81`, path length and bytes, four-byte separator, then the property marker, value length and value. Raw `Grizzleheim/Interiors/Hub_CraftingRoom` contains `Zone_00000026`, which resolves to `Hall of the Ice Forge`; the independently associated shared map says `Northguard`. The direct current-zone header is the specific room name, while the map is the broader area. This corroborates that `Zone_` in this exact header position is a usable current-zone title.

The housing headers provide further direct examples: `Zone_00001374` resolves to `Avalon Castle Plot`, `Zone_00001524` to `Castle Tours Apartment`, `Zone_00001519` to `Olde Town Apartment`, and `Zone_00001188` to the extracted `Midday  Estate` (including its original double space). The two Dusk build-a-castle paths use `Housing_00000832`, which resolves in `Housing.lang` to `Meadows at Dusk`. These strings are preserved exactly; no spelling or whitespace correction is applied.

The symbolic WizardZone records were checked in `raw/misc/Root/Locale/en-US/WizardZone.lang`. Their relevant forms are `TheCommons` / `Name of commons zone` / `The Commons`, `Ravenwood` / `zone name for Ravenwood` / `Ravenwood`, and `UnicornWay` / `Zone name for Unicorn Way` / `Unicorn Way`. These remain unverified fallback candidates because WizardZone can label a parent area or phase.

Some nearby localized fields were rejected as names. `Marleybone/Interiors/MB_Gauntlet/MB_Gauntlet1/MB_Gauntlet1_01` has the current header field `SigilText_00000011`, whose `SigilText.lang` value is `Spiral Geographic Society Archives`; the field explicitly describes sigil text, not the zone title. `Marleybone/MB_Station/Interiors/MB_Ironworks_T7` has `WizQst9B90_00000019`; the linked `WizQst9B90.lang` contains no `00000019` entry, and quest text is not a zone title. Neither is selected.

The Phantom Zone minigame resources contain gameplay start objects and global minigame title/instruction strings. The title strings are attached to the minigame UI, but the inspected zone records do not contain a typed reference from the canonical zone path to those title keys. Matching actor or path tokens to localization names alone is insufficient. Krokotopia tomb and Marleybone Big Ben subzones contain transition/target labels and internal WizardZone references in later records, but no usable immediate current-zone localization key or supported map/POI child-name chain. Those strings do not establish a displayed location name.

Every audited path has a raw `gamedata.bin` resource. “No localization reference” below means no usable direct current-zone name key and no proven map/POI name relationship; it does not claim that the resource contains no unrelated localized quest, UI, or gameplay text.

## Per-path classification

The categories describe why each path was `Unknown` in `df4c243`; `resolved` marks the paths recovered by this pass.

| Canonical path | Classification at audit | Finding / outcome |
| --- | --- | --- |
| `Housing_AV_BAC/Exterior` | Other: unsupported direct Zone header | `Zone_00001374` -> `Zone.lang` -> `Avalon Castle Plot`; resolved, verified |
| `Housing_AV_BAC/Exterior_Preview` | Other: unsupported direct Zone header | `Zone_00001374` -> `Zone.lang` -> `Avalon Castle Plot`; resolved, verified |
| `Housing_AV_BAC/Interior` | Other: unsupported direct Zone header | `Zone_00001374` -> `Zone.lang` -> `Avalon Castle Plot`; resolved, verified |
| `Housing_AV_BAC/Interior_Preview` | Other: unsupported direct Zone header | `Zone_00001374` -> `Zone.lang` -> `Avalon Castle Plot`; resolved, verified |
| `Housing_BuildACastleProto/BAC_Dusk_Ext_Preview` | Other: unsupported direct Housing header | `Housing_00000832` -> `Housing.lang` -> `Meadows at Dusk`; resolved, verified |
| `Housing_BuildACastleProto/BAC_Midday_Ext_Preview` | Other: unsupported direct Zone header | `Zone_00001188` -> `Zone.lang` -> `Midday  Estate`; resolved, verified |
| `Housing_BuildACastleProto/BAC_Midday_Int_Preview` | Other: unsupported direct Zone header | `Zone_00001188` -> `Zone.lang` -> `Midday  Estate`; resolved, verified |
| `Housing_BuildACastleProto/Exterior` | Other: unsupported direct Zone header | `Zone_00001188` -> `Zone.lang` -> `Midday  Estate`; resolved, verified |
| `Housing_BuildACastleProto/Exterior_Dusk` | Other: unsupported direct Housing header | `Housing_00000832` -> `Housing.lang` -> `Meadows at Dusk`; resolved, verified |
| `Housing_BuildACastleProto/Interior` | Other: unsupported direct Zone header | `Zone_00001188` -> `Zone.lang` -> `Midday  Estate`; resolved, verified |
| `Housing_CastleToursApt/Housing_CastleToursApt_Ext` | Other: unsupported direct Zone header | `Zone_00001524` -> `Zone.lang` -> `Castle Tours Apartment`; resolved, verified |
| `Housing_CastleToursApt/Housing_CastleToursApt_Ext_Preview` | Other: unsupported direct Zone header | `Zone_00001524` -> `Zone.lang` -> `Castle Tours Apartment`; resolved, verified |
| `Housing_CastleToursApt/Housing_CastleToursApt_Int` | Other: unsupported direct Zone header | `Zone_00001524` -> `Zone.lang` -> `Castle Tours Apartment`; resolved, verified |
| `Housing_CastleToursApt/Housing_CastleToursApt_Int_Preview` | Other: unsupported direct Zone header | `Zone_00001524` -> `Zone.lang` -> `Castle Tours Apartment`; resolved, verified |
| `Housing_WizardCommonsApt/House_WizardCommonsApt_Ext` | Other: unsupported direct Zone header | `Zone_00001519` -> `Zone.lang` -> `Olde Town Apartment`; resolved, verified |
| `Housing_WizardCommonsApt/House_WizardCommonsApt_Ext_Pre` | Other: unsupported direct Zone header | `Zone_00001519` -> `Zone.lang` -> `Olde Town Apartment`; resolved, verified |
| `Housing_WizardCommonsApt/House_WizardCommonsApt_Int` | Other: unsupported direct Zone header | `Zone_00001519` -> `Zone.lang` -> `Olde Town Apartment`; resolved, verified |
| `Housing_WizardCommonsApt/House_WizardCommonsApt_Int_Pre` | Other: unsupported direct Zone header | `Zone_00001519` -> `Zone.lang` -> `Olde Town Apartment`; resolved, verified |
| `Krokotopia/KT_Tomb/Interiors/KT_TombOfSutekh8` | No localization reference | Tomb transition resources have no usable direct zone title or proven child-name relationship; remains `Unknown` |
| `Krokotopia/KT_Tomb/Interiors/KT_TombOfSutekh_028` | No localization reference | Variant tomb transition resources have no usable direct zone title or proven child-name relationship; remains `Unknown` |
| `Krokotopia/KT_Tomb/KT_DjeseritTomb8` | No localization reference | Tomb resources have no usable direct zone title or proven map/POI child-name relationship; remains `Unknown` |
| `Marleybone/Interiors/MB_Gauntlet/MB_Gauntlet1/MB_Gauntlet1_01` | Other: non-zone SigilText field | `SigilText_00000011` resolves to `Spiral Geographic Society Archives`, a sigil label; remains `Unknown` |
| `Marleybone/MB_BigBen/MB_CounterweightEast0` | No localization reference | Big Ben level targets identify transitions only; remains `Unknown` |
| `Marleybone/MB_BigBen/MB_CounterweightWest0` | No localization reference | Big Ben level targets identify transitions only; remains `Unknown` |
| `Marleybone/MB_ScotlandYard/MB_Roof8` | No localization reference | Later WizardZone reference is not the current-zone header field; roof transitions do not prove a display title; remains `Unknown` |
| `Marleybone/MB_Station/Interiors/MB_Ironworks_T7` | Other: non-zone quest field | `WizQst9B90_00000019` is a quest-resource reference with no matching entry; remains `Unknown` |
| `Test/Court_Test` | Dev/test/variant zone | `Zone_00001518` -> `Zone.lang` -> `TEST-Interior`; rejected as a test label, remains `Unknown` |
| `Test/Sound_Test` | Unresolved symbolic key format | `WizardZone_TheCommons` -> described symbolic `WizardZone.lang` row -> `The Commons`; resolved as unverified fallback |
| `ThePhantomZoneWorld/ChooChooZooP` | No localization reference | Minigame title/instruction text has no typed path-to-title localization link; remains `Unknown` |
| `ThePhantomZoneWorld/DoodleDougPhantomZoneP` | No localization reference | Minigame title/instruction text has no typed path-to-title localization link; remains `Unknown` |
| `ThePhantomZoneWorld/Dueling_DiegoP` | No localization reference | Minigame title/instruction text has no typed path-to-title localization link; remains `Unknown` |
| `ThePhantomZoneWorld/HotShotsP` | No localization reference | Minigame title/instruction text has no typed path-to-title localization link; remains `Unknown` |
| `ThePhantomZoneWorld/PetGameDanceP` | No localization reference | Pet-game title is present, but no typed path-to-title localization link exists; remains `Unknown` |
| `ThePhantomZoneWorld/PetGameMorphP` | No localization reference | Parent hatchery has Pet Morph gameplay objects, but no localized current-zone title chain; remains `Unknown` |
| `ThePhantomZoneWorld/PotionMotionP` | No localization reference | Minigame title/instruction text has no typed path-to-title localization link; remains `Unknown` |
| `ThePhantomZoneWorld/ShockalockP` | No localization reference | Minigame title/instruction text has no typed path-to-title localization link; remains `Unknown` |
| `ThePhantomZoneWorld/SkullRidersPhantomZoneP` | No localization reference | Minigame title text has no typed path-to-title localization link; remains `Unknown` |
| `ThePhantomZoneWorld/SoblocksPhantomZoneP` | No localization reference | Minigame title text has no typed path-to-title localization link; remains `Unknown` |
| `ThePhantomZoneWorld/ThePhantomZoneP` | No localization reference | Generic Phantom Zone/UI location strings are not tied to this current path; remains `Unknown` |
| `ThePhantomZoneWorld/WebGameGrubGuardianP` | No localization reference | Pet-game data has no typed path-to-title localization link; remains `Unknown` |
| `ThePhantomZoneWorld/WebGameLobbyP` | No localization reference | Lobby resources contain no usable direct zone title; remains `Unknown` |
| `ThePhantomZoneWorld/concentrationP` | No localization reference | Minigame title/instruction and quest text do not establish a current-zone title; remains `Unknown` |
| `WizardCity/QA_SpawnRate` | Unresolved numeric localization key | `WizardZone_00000577` -> `WizardZone.lang` -> one space; unusable, remains `Unknown` |
| `WizardCity/Tutorial_InteriorP` | No localization reference | Tutorial encounter resources contain no usable direct zone title or proven map/POI link; remains `Unknown` |
| `WizardCity/WC_Hub` | Unresolved symbolic key format | `WizardZone_TheCommons` -> described symbolic `WizardZone.lang` row -> `The Commons`; resolved as unverified fallback |
| `WizardCity/WC_Ravenwood` | Unresolved symbolic key format | `WizardZone_Ravenwood` -> described symbolic `WizardZone.lang` row -> `Ravenwood`; resolved as unverified fallback |
| `WizardCity/WC_Ravenwood_Graduation` | Unresolved symbolic key format | `WizardZone_Ravenwood` -> described symbolic `WizardZone.lang` row -> `Ravenwood`; resolved as unverified fallback |
| `WizardCity/WC_Ravenwood_Lite` | Unresolved symbolic key format | `WizardZone_Ravenwood` -> described symbolic `WizardZone.lang` row -> `Ravenwood`; resolved as unverified fallback |
| `WizardCity/WC_Ravenwood_TeleporterH` | No localization reference | Local teleport targets and an unassociated WizardZone string do not establish the current display name; remains `Unknown` |
| `WizardCity/WC_Streets/Interiors/WC_Unicorn_H4` | Unresolved symbolic key format | `WizardZone_UnicornWay` -> described symbolic `WizardZone.lang` row -> `Unicorn Way`; resolved as unverified fallback |
| `WizardCity/WC_Streets/Interiors/WC_Unicorn_HedgeMaze` | Unresolved symbolic key format | `WizardZone_UnicornWay` -> described symbolic `WizardZone.lang` row -> `Unicorn Way`; resolved as unverified fallback |
| `WizardCity/WC_Streets/Interiors/WC_Unicorn_T1` | Unresolved symbolic key format | `WizardZone_UnicornWay` -> described symbolic `WizardZone.lang` row -> `Unicorn Way`; resolved as unverified fallback |
| `WizardCity/WC_Streets/WC_Unicorn` | Unresolved symbolic key format | `WizardZone_UnicornWay` -> described symbolic `WizardZone.lang` row -> `Unicorn Way`; resolved as unverified fallback |

## Validation and limitations

After adding the two deterministic extraction rules, the generator reports 1,240 verified names, 2,080 unverified WizardZone fallbacks, and 26 `Unknown` values across 3,346 paths. Against `tmp/zones.json`, it reports 1,573 exact matches, 1,062 mismatches, 36 unresolved reference paths, and 685 named generated-only paths. These numbers validate extracted output only; the reference contributes no names.

The remaining Unknowns are intentionally unresolved. The global minigame UI catalogs, sigil text, quest localization, target labels, and similarly named resources can describe gameplay rather than the current player-visible location. A future resolver needs a typed raw link from a canonical zone to the specific UI title before using those channels.
