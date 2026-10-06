# World-root fallback audit

Audit of the 86 roots that used `raw_root_fallback` before this policy revision, using extracted `raw/` data and en-US localization. `tmp/zones.json` was not used as evidence. Counts below are number of canonical zone entries under each root.

## 1. Defensible player-facing mappings

| Root | Entries | World | Evidence |
|---|---:|---|---|
| `Housing_AR_Dormroom` | 1 | Arcanum | Its current-zone `Zone.lang` title is “Arcanum Apartment”. |
| `Housing_AV_BAC` | 4 | Avalon | Its current-zone `Zone.lang` title is “Avalon Castle Plot”. |
| `Housing_AvalonTier1` | 4 | Avalon | Contains the complete `Avalon` key/display spelling in `WorldNames.lang`. |
| `Housing_AztecaTier1` | 4 | Azteca | Contains the complete `Azteca` key/display spelling in `WorldNames.lang`. |
| `Housing_Darkmoor` | 4 | Darkmoor | Contains the complete `Darkmoor` key/display spelling in `WorldNames.lang`. |
| `Housing_KT_Apartment` | 2 | Krokotopia | Its current-zone `Zone.lang` title is “Krokotopia Apartment”. |
| `Housing_Mirage_Tents` | 4 | Mirage | Contains the complete `Mirage` key/display spelling in `WorldNames.lang`. |
| `Housing_Novus` | 4 | Novus | Contains the complete `Novus` key/display spelling in `WorldNames.lang`. |
| `Housing_Polaris_Ship` | 4 | Polaris | Contains the complete `Polaris` key/display spelling in `WorldNames.lang`. |
| `Housing_Villa_Gardens` | 4 | Wysteria | Its current-zone `Zone.lang` title is “Wysteria Villa”. |
| `Housing_Wallaru_Ranch` | 4 | Wallaru | Contains the complete `Wallaru` key/display spelling in `WorldNames.lang`. |

These mappings are exact-root rules with provenance in the resolver. Similar abbreviations and merely thematic zone titles are not generalized. For example, `Housing_AV_Gauntlet`, `Housing_AZ_BAC`, `Housing_KaramelVillage`, and `Housing_EM_Airship` do not qualify based on their roots/titles alone.

## 2. Recognizable player-facing roots preserved unchanged

| Root | Entries | Reason |
|---|---:|---|
| `PetDerby` | 18 | The extracted `PetDerby.lang` and `Arena.lang` identify the player-facing Pet Derby activity. No world mapping is implied. |
| `Raids` | 1 | The extracted raid resources establish a player-facing raid feature, but do not identify a world for its canonical root. No world mapping is implied. |

## 3. Roots without a defensible world mapping

At the time of this evidence audit, the following 73 roots had no defensible player-facing world mapping. The current output applies the requested final fallback to these roots: the 66 `Housing*` roots become `House`, while the other seven roots are preserved unchanged. This fallback behavior does not assert that these technical/group roots are canonical world names; it is applied because the zone database must carry a non-Unknown value for every non-empty root.

| Root | Entries | Root | Entries |
|---|---:|---|---:|
| `DD_DS_01` | 30 | `DD_PA_01` | 10 |
| `Heroic_Dungeons_01` | 22 | `Holiday` | 15 |
| `Housing` | 78 | `Housing_Acropolis` | 4 |
| `Housing_AeroVillage` | 2 | `Housing_AV_Gauntlet` | 25 |
| `Housing_AV_OutlawHouse` | 4 | `Housing_AZ_BAC` | 4 |
| `Housing_BeeHouse` | 4 | `Housing_BirdNest` | 1 |
| `Housing_BotanicalGardens` | 4 | `Housing_BuildACastleProto` | 12 |
| `Housing_CastleToursApt` | 4 | `Housing_CL` | 8 |
| `Housing_CreepyFairgrounds` | 4 | `Housing_CrystalMine` | 1 |
| `Housing_DM_GraveholmInn` | 4 | `Housing_EM_Airship` | 4 |
| `Housing_FairytaleFarm` | 4 | `Housing_FantasyPalace` | 2 |
| `Housing_FarmHouse` | 4 | `Housing_FishBowl` | 1 |
| `Housing_FishingRetreat` | 1 | `Housing_FlotsamCantina` | 4 |
| `Housing_Gauntlet` | 40 | `Housing_Gauntlet_Forbidden_Library` | 1 |
| `Housing_Gauntlet_Highlands` | 3 | `Housing_Gauntlet_Professor` | 4 |
| `Housing_Gauntlet_Sinbad` | 3 | `Housing_Gauntlet_SIT` | 2 |
| `Housing_Gauntlet_Swamp` | 4 | `Housing_Gauntlet_Theater` | 24 |
| `Housing_Gauntlet_Train` | 6 | `Housing_Gauntlet_Voyage` | 25 |
| `Housing_GH_BlacksmithFjord` | 4 | `Housing_GHGauntlet` | 30 |
| `Housing_GreatBelow` | 4 | `Housing_Guild` | 4 |
| `Housing_KaramelVillage` | 4 | `Housing_KR_PeppergrassGlen` | 2 |
| `Housing_KrampusMagicArena` | 1 | `Housing_KT_RiverPalace` | 4 |
| `Housing_LightkeepersEstate` | 6 | `Housing_LM_StreetHouse` | 4 |
| `Housing_MonsterMagicArena` | 1 | `Housing_MR_Dormroom` | 12 |
| `Housing_NinjaDojo` | 1 | `Housing_PA_Gauntlet` | 30 |
| `Housing_Polarian_Fort` | 4 | `Housing_PyramidOfTheLost` | 4 |
| `Housing_Sandbox` | 1 | `Housing_SeaVillage` | 4 |
| `Housing_SkyCastle` | 4 | `Housing_SP` | 4 |
| `Housing_SP_CottageCore` | 4 | `Housing_SP_FlatIsland` | 4 |
| `Housing_SP_TomeFallArena` | 1 | `Housing_SP_TomeSummerArena` | 1 |
| `Housing_SteamSkyCastle` | 4 | `Housing_SunkP` | 4 |
| `Housing_SunPal` | 4 | `Housing_VA_Gauntlet` | 30 |
| `Housing_Watch` | 4 | `Housing_WaterMoleResort` | 4 |
| `Housing_Winter_Wind_Tower` | 4 | `Housing_WizardCommonsApt` | 4 |
| `Housing_WizardKeep` | 4 | `Housing_ZF_HouseBoat` | 4 |
| `MonthlyEvents` | 29 | `Test` | 3 |
| `ThePhantomZoneWorld` | 29 |  |  |

`DD_DS_01` and `DD_PA_01` have activity transitions (“BattleBands” and “Tanglewood”) tied to technical packages; those labels do not establish world names. `Test` is a test root. `ThePhantomZoneWorld` contains minigame/technical content, not a localized world-title association. `Holiday`, `MonthlyEvents`, and the `Housing_*` entries are package/group labels whose contents span activities or properties, so the current final fallback uses their root (or `House`) without claiming it is an evidence-backed world mapping.

## Sources and policy

- `raw/misc/Root/Locale/en-US/WorldNames.lang`: canonical localized world keys and display spellings.
- `raw/misc/Root/Locale/en-US/Zone.lang`: direct localized zone titles cited for the four title-based mappings.
- `raw/misc/Root/Locale/en-US/PetDerby.lang` and `Arena.lang`: evidence that Pet Derby is a player-facing activity, not evidence of a world.
- Extracted raid resources: evidence that raids are a player-facing feature, not evidence of a world.
- World transition and hub/group metadata was inspected for the technical/event roots. A transition can name an activity and a hub grouping can aggregate unrelated zones; neither is generalized into a world mapping.

The validation file `tmp/zones.json` is not a source of world names. In the 3,346-entry snapshot, 2,643 entries use localization, 51 use canonical aliases, 19 retain recognized raw roots, 138 use `root_fallback`, 495 use `house_fallback`, and none remain `Unknown`. The fallback sources apply only to the entries that previously emitted `Unknown`; they do not change the localized or alias results. Counts can change with a different extraction.
