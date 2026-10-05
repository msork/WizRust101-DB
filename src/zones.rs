use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Source {
    CompassPoi,
    ZoneHeader,
    HousingHeader,
    WizardZone,
    SharedMap,
}

impl Source {
    pub fn label(self) -> &'static str {
        match self {
            Self::CompassPoi => "CompassPoi",
            Self::ZoneHeader => "ZoneHeader",
            Self::HousingHeader => "HousingHeader",
            Self::WizardZone => "WizardZone",
            Self::SharedMap => "SharedMap",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub source: Source,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Zone {
    pub path: String,
    pub world_lookup_key: String,
    pub world_name: Option<String>,
    pub candidates: Vec<Candidate>,
    pub selected: Option<Candidate>,
    pub confidence: Confidence,
    pub header_field_value: Option<String>,
    pub header_localized_value: Option<String>,
    pub header_wizard_zone: Option<String>,
    pub selection_provenance: String,
    pub ambiguity: Option<String>,
    pub conflicts: Vec<String>,
    pub unresolved: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confidence {
    Verified,
    UnverifiedFallback,
    Unknown,
}

impl Confidence {
    pub fn label(self) -> &'static str {
        match self {
            Self::Verified => "verified",
            Self::UnverifiedFallback => "unverified_fallback",
            Self::Unknown => "unknown",
        }
    }
}

struct RawZone {
    directory: PathBuf,
    data: Vec<u8>,
}

pub fn discover(root: &Path) -> Result<Vec<Zone>, Box<dyn std::error::Error>> {
    if !root.is_dir() {
        return Err(format!("input root is not a directory: {}", root.display()).into());
    }
    let mut dirs = Vec::new();
    collect(root, &mut dirs)?;
    dirs.sort();

    let mut raw_zones: BTreeMap<String, RawZone> = BTreeMap::new();
    for dir in dirs {
        let file = dir.join("gamedata.bin");
        if !file.is_file() {
            continue;
        }
        let data = fs::read(&file)?;
        if let Some(path) = embedded_path(&data) {
            if let Some(previous) = raw_zones.get(&path) {
                if previous.data != data {
                    return Err(format!(
                        "duplicate canonical zone path has conflicting data: {path}"
                    )
                    .into());
                }
            } else {
                raw_zones.insert(
                    path,
                    RawZone {
                        directory: dir,
                        data,
                    },
                );
            }
        }
    }
    let paths: BTreeSet<String> = raw_zones.keys().cloned().collect();
    let zone_lang = parse_lang(&fs::read(root.join("misc/Root/Locale/en-US/Zone.lang"))?)?;
    let housing_lang = parse_lang(&fs::read(root.join("misc/Root/Locale/en-US/Housing.lang"))?)?;
    let wizard_zone_lang = parse_lang(&fs::read(
        root.join("misc/Root/Locale/en-US/WizardZone.lang"),
    )?)?;
    let world_names = parse_lang(&fs::read(
        root.join("misc/Root/Locale/en-US/WorldNames.lang"),
    )?)?;
    let compass_lang = parse_lang(&fs::read(
        root.join("misc/Root/Locale/en-US/WizardCompassLocs.lang"),
    )?)?;
    let map_refs =
        parse_map_associations(&fs::read(root.join("misc/Root/DoodleMapMap.xml"))?, &paths);
    let map_names = resolve_map_names(root, &map_refs, &zone_lang)?;
    let compass_names = resolve_compass_children(&raw_zones, &map_refs, &compass_lang);

    let mut zones = Vec::with_capacity(raw_zones.len());
    for (path, raw) in raw_zones {
        let map_candidates = map_names.get(&path).cloned().unwrap_or_default();
        let usable_map_candidates: BTreeSet<String> = map_candidates
            .iter()
            .filter(|name| valid_map_title(name))
            .cloned()
            .collect();
        let header_field_value = zone_header_field_value(&raw.data, &path);
        let header_localized_value = header_field_value.as_deref().and_then(|value| {
            localization_for_header(value, &zone_lang, &housing_lang, &wizard_zone_lang).cloned()
        });
        let zone_header_candidates = candidate_from_header(
            header_field_value.as_deref(),
            &zone_lang,
            &housing_lang,
            &wizard_zone_lang,
        )
        .filter(|candidate| candidate.source == Source::ZoneHeader)
        .into_iter()
        .map(|candidate| candidate.name)
        .collect::<BTreeSet<_>>();
        let housing_header_candidates = candidate_from_header(
            header_field_value.as_deref(),
            &zone_lang,
            &housing_lang,
            &wizard_zone_lang,
        )
        .filter(|candidate| candidate.source == Source::HousingHeader)
        .into_iter()
        .map(|candidate| candidate.name)
        .collect::<BTreeSet<_>>();
        let wizard_candidates = candidate_from_header(
            header_field_value.as_deref(),
            &zone_lang,
            &housing_lang,
            &wizard_zone_lang,
        )
        .filter(|candidate| candidate.source == Source::WizardZone)
        .into_iter()
        .map(|candidate| candidate.name)
        .collect::<BTreeSet<_>>();
        let poi_candidates = compass_names.get(&path).cloned().unwrap_or_default();
        let mut candidates = Vec::new();
        candidates.extend(poi_candidates.iter().cloned().map(|name| Candidate {
            source: Source::CompassPoi,
            name,
        }));
        candidates.extend(
            zone_header_candidates
                .iter()
                .cloned()
                .map(|name| Candidate {
                    source: Source::ZoneHeader,
                    name,
                }),
        );
        candidates.extend(
            housing_header_candidates
                .iter()
                .cloned()
                .map(|name| Candidate {
                    source: Source::HousingHeader,
                    name,
                }),
        );
        candidates.extend(wizard_candidates.iter().cloned().map(|name| Candidate {
            source: Source::WizardZone,
            name,
        }));
        candidates.extend(map_candidates.iter().cloned().map(|name| Candidate {
            source: Source::SharedMap,
            name,
        }));
        candidates.sort_by(|a, b| (a.source, &a.name).cmp(&(b.source, &b.name)));

        let (selected, confidence, mut ambiguity, conflicts) = select_candidate(
            &poi_candidates,
            &zone_header_candidates,
            &housing_header_candidates,
            &wizard_candidates,
            &usable_map_candidates,
        );
        let rejected: Vec<_> = map_candidates
            .difference(&usable_map_candidates)
            .cloned()
            .collect();
        if !rejected.is_empty() {
            let reason = format!(
                "rejected malformed shared-map names: {}",
                rejected.join(" | ")
            );
            ambiguity = Some(match ambiguity {
                Some(existing) => format!("{existing}; {reason}"),
                None => reason,
            });
        }
        let selection_provenance = selected.as_ref().map_or_else(
            || "no usable localized candidate; emitted Unknown".to_owned(),
            |candidate| match candidate.source {
                Source::CompassPoi => {
                    "verified parent POI/Compass localization and matching child transition chain"
                        .to_owned()
                }
                Source::ZoneHeader => format!(
                    "exact zone-header {:?} -> Zone.lang",
                    header_field_value.as_deref().unwrap_or_default()
                ),
                Source::HousingHeader => format!(
                    "exact zone-header {:?} -> Housing.lang",
                    header_field_value.as_deref().unwrap_or_default()
                ),
                Source::SharedMap => {
                    "verified DoodleMapMap association -> map resource Zone key -> Zone.lang"
                        .to_owned()
                }
                Source::WizardZone => format!(
                    "unverified zone-header WizardZone field {:?} -> WizardZone.lang; this channel can name a parent region or phase",
                    header_field_value.as_deref().unwrap_or_default()
                ),
            },
        );
        let header_wizard_zone = header_field_value
            .as_deref()
            .filter(|value| value.starts_with("WizardZone_"))
            .map(str::to_owned);
        let unresolved = confidence == Confidence::Unknown;
        let world_lookup_key = path.split('/').next().unwrap_or_default().to_owned();
        let world_name = world_names.get(&world_lookup_key).cloned();
        zones.push(Zone {
            path,
            world_lookup_key,
            world_name,
            candidates,
            selected,
            confidence,
            header_field_value,
            header_localized_value,
            header_wizard_zone,
            selection_provenance,
            ambiguity,
            conflicts,
            unresolved,
        });
    }
    Ok(zones)
}

fn select_candidate(
    poi: &BTreeSet<String>,
    zone_header: &BTreeSet<String>,
    housing_header: &BTreeSet<String>,
    wizard: &BTreeSet<String>,
    map: &BTreeSet<String>,
) -> (Option<Candidate>, Confidence, Option<String>, Vec<String>) {
    let conflicts = candidate_conflicts(poi, zone_header, housing_header, wizard, map);
    if poi.len() == 1 {
        let name = poi.first().expect("one POI candidate");
        return (
            Some(Candidate {
                source: Source::CompassPoi,
                name: name.clone(),
            }),
            Confidence::Verified,
            None,
            conflicts,
        );
    }

    if zone_header.len() == 1 {
        return (
            Some(Candidate {
                source: Source::ZoneHeader,
                name: zone_header
                    .first()
                    .expect("one Zone header candidate")
                    .clone(),
            }),
            Confidence::Verified,
            None,
            conflicts,
        );
    }

    if housing_header.len() == 1 {
        return (
            Some(Candidate {
                source: Source::HousingHeader,
                name: housing_header
                    .first()
                    .expect("one Housing header candidate")
                    .clone(),
            }),
            Confidence::Verified,
            None,
            conflicts,
        );
    }

    let candidate_conflict = !poi.is_empty()
        || zone_header.len() > 1
        || housing_header.len() > 1
        || map.len() > 1
        || (map.len() == 1 && wizard.len() == 1 && wizard.first() != map.first());
    if poi.is_empty() && map.len() == 1 && (wizard.is_empty() || wizard == map) {
        let name = map.first().expect("one map candidate");
        return (
            Some(Candidate {
                source: Source::SharedMap,
                name: name.clone(),
            }),
            Confidence::Verified,
            None,
            conflicts,
        );
    }

    let ambiguity = if poi.len() > 1 {
        Some(format!(
            "conflicting proven Compass POI names: {}",
            join(poi)
        ))
    } else if map.len() > 1 {
        Some(format!("conflicting shared-map names: {}", join(map)))
    } else if candidate_conflict {
        Some(format!(
            "candidate conflict; WizardZone={} SharedMap={} CompassPoi={}",
            join(wizard),
            join(map),
            join(poi)
        ))
    } else if wizard.len() == 1 {
        Some("WizardZone-only candidate is an unverified fallback".to_owned())
    } else {
        None
    };
    if wizard.len() == 1 {
        return (
            Some(Candidate {
                source: Source::WizardZone,
                name: wizard.first().expect("one WizardZone candidate").clone(),
            }),
            Confidence::UnverifiedFallback,
            ambiguity,
            conflicts,
        );
    }
    (None, Confidence::Unknown, ambiguity, conflicts)
}

fn valid_map_title(name: &str) -> bool {
    let trimmed = name.trim();
    !trimmed.is_empty()
        && trimmed == name
        && !name.contains('|')
        && !name.chars().any(char::is_control)
        && !name.contains("  ")
}

fn valid_localized_name(name: &str) -> bool {
    !name.trim().is_empty()
        && !name.chars().any(char::is_control)
        && !name.contains('<')
        && !name.contains('>')
}

fn valid_direct_header_name(name: &str) -> bool {
    valid_localized_name(name) && !name.trim().to_ascii_uppercase().starts_with("TEST-")
}

fn candidate_conflicts(
    poi: &BTreeSet<String>,
    zone_header: &BTreeSet<String>,
    housing_header: &BTreeSet<String>,
    wizard: &BTreeSet<String>,
    map: &BTreeSet<String>,
) -> Vec<String> {
    let candidates = [
        (Source::CompassPoi, poi),
        (Source::ZoneHeader, zone_header),
        (Source::HousingHeader, housing_header),
        (Source::WizardZone, wizard),
        (Source::SharedMap, map),
    ];
    let mut conflicts = BTreeSet::new();
    for (i, (source, names)) in candidates.iter().enumerate() {
        for (other_source, other_names) in candidates.iter().skip(i + 1) {
            for name in names.iter() {
                for other in other_names.iter() {
                    if name != other {
                        conflicts.insert(format!(
                            "{}={name:?} conflicts with {}={other:?}",
                            source.label(),
                            other_source.label()
                        ));
                    }
                }
            }
        }
    }
    conflicts.into_iter().collect()
}

fn join(names: &BTreeSet<String>) -> String {
    names.iter().cloned().collect::<Vec<_>>().join(" | ")
}

fn collect(dir: &Path, dirs: &mut Vec<PathBuf>) -> io::Result<()> {
    dirs.push(dir.to_path_buf());
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            collect(&entry.path(), dirs)?;
        }
    }
    Ok(())
}

fn embedded_path(bytes: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(bytes);
    text.split(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '/' | '_' | '-')))
        .find(|value| {
            value.contains('/')
                && value.split('/').all(|part| !part.is_empty())
                && value
                    .split('/')
                    .next()
                    .is_some_and(|part| part.chars().next().is_some_and(char::is_uppercase))
        })
        .map(str::to_owned)
}

fn parse_lang(bytes: &[u8]) -> Result<BTreeMap<String, String>, Box<dyn std::error::Error>> {
    if !bytes.len().is_multiple_of(2) {
        return Err("localization table has odd byte length".into());
    }
    let (pairs, _) = bytes.as_chunks::<2>();
    let units: Vec<u16> = pairs.iter().map(|pair| u16::from_le_bytes(*pair)).collect();
    let text = String::from_utf16(units.strip_prefix(&[0xfeff]).unwrap_or(&units))?;
    let lines: Vec<&str> = text.lines().collect();
    let mut values = BTreeMap::new();
    for (i, line) in lines.iter().enumerate() {
        let key = line.trim();
        if !is_lang_key(key) {
            continue;
        }
        let Some(second) = lines.get(i + 1) else {
            continue;
        };
        if second.is_empty() {
            if let Some(value) = lines.get(i + 2).filter(|value| !value.is_empty()) {
                values.insert(key.to_owned(), (*value).to_owned());
            }
        } else if !key.bytes().all(|byte| byte.is_ascii_digit()) && is_symbolic_description(second)
        {
            if let Some(value) = lines.get(i + 2).filter(|value| !value.is_empty()) {
                values.insert(key.to_owned(), (*value).to_owned());
            }
        }
    }
    Ok(values)
}

fn is_lang_key(key: &str) -> bool {
    !key.is_empty()
        && key
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b':')
}

fn is_symbolic_description(value: &str) -> bool {
    let description = value.trim().to_ascii_lowercase();
    description.starts_with("zone name for ") || description == "name of commons zone"
}

fn localization_for_header<'a>(
    header: &str,
    zone_lang: &'a BTreeMap<String, String>,
    housing_lang: &'a BTreeMap<String, String>,
    wizard_zone_lang: &'a BTreeMap<String, String>,
) -> Option<&'a String> {
    let (source, key) = if let Some(key) = header.strip_prefix("Zone_") {
        (zone_lang, key)
    } else if let Some(key) = header.strip_prefix("Housing_") {
        (housing_lang, key)
    } else {
        (wizard_zone_lang, header.strip_prefix("WizardZone_")?)
    };
    source.get(key)
}

fn candidate_from_header(
    header: Option<&str>,
    zone_lang: &BTreeMap<String, String>,
    housing_lang: &BTreeMap<String, String>,
    wizard_zone_lang: &BTreeMap<String, String>,
) -> Option<Candidate> {
    let header = header?;
    let (source, value) = if header.starts_with("Zone_") {
        (
            Source::ZoneHeader,
            localization_for_header(header, zone_lang, housing_lang, wizard_zone_lang)?,
        )
    } else if header.starts_with("Housing_") {
        (
            Source::HousingHeader,
            localization_for_header(header, zone_lang, housing_lang, wizard_zone_lang)?,
        )
    } else if header.starts_with("WizardZone_") {
        (
            Source::WizardZone,
            localization_for_header(header, zone_lang, housing_lang, wizard_zone_lang)?,
        )
    } else {
        return None;
    };
    let valid = if matches!(source, Source::ZoneHeader | Source::HousingHeader) {
        valid_direct_header_name(value)
    } else {
        valid_localized_name(value)
    };
    valid.then(|| Candidate {
        source,
        name: value.clone(),
    })
}

fn zone_header_field_value(bytes: &[u8], path: &str) -> Option<String> {
    const PATH_MARKER: &[u8] = b"\xf8\x63\x69\x81";
    const LOCALIZATION_MARKER: &[u8] = b"\x6e\xec\xf6\x74";
    let mut cursor = 0;
    while let Some(marker) = find_from(bytes, PATH_MARKER, cursor) {
        let length_start = marker + PATH_MARKER.len();
        let length_bytes = bytes.get(length_start..length_start + 2)?;
        let path_length = u16::from_le_bytes([length_bytes[0], length_bytes[1]]) as usize;
        let path_start = length_start + 2;
        let path_end = path_start.checked_add(path_length)?;
        if bytes.get(path_start..path_end) == Some(path.as_bytes()) {
            let field_marker = path_end.checked_add(4)?;
            if bytes.get(field_marker..field_marker + LOCALIZATION_MARKER.len())
                != Some(LOCALIZATION_MARKER)
            {
                return None;
            }
            let value_length_start = field_marker + LOCALIZATION_MARKER.len();
            let value_length_bytes = bytes.get(value_length_start..value_length_start + 2)?;
            let value_length =
                u16::from_le_bytes([value_length_bytes[0], value_length_bytes[1]]) as usize;
            let value_start = value_length_start + 2;
            let value =
                std::str::from_utf8(bytes.get(value_start..value_start + value_length)?).ok()?;
            return value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
                .then(|| value.to_owned());
        }
        cursor = path_end;
    }
    None
}

fn resource_keys(bytes: &[u8], prefix: &[u8]) -> BTreeSet<String> {
    let mut keys = BTreeSet::new();
    let mut start = 0;
    while let Some(index) = find_from(bytes, prefix, start) {
        let digits_start = index + prefix.len();
        if bytes
            .get(digits_start..digits_start + 8)
            .is_some_and(|digits| digits.iter().all(u8::is_ascii_digit))
        {
            keys.insert(String::from_utf8_lossy(&bytes[digits_start..digits_start + 8]).into());
        }
        start = digits_start;
    }
    keys
}

fn first_zone_key(bytes: &[u8]) -> Option<String> {
    first_resource_key(bytes, b"Zone_")
}

fn first_resource_key(bytes: &[u8], prefix: &[u8]) -> Option<String> {
    let mut cursor = 0;
    while let Some(index) = find_from(bytes, prefix, cursor) {
        let start = index + prefix.len();
        if bytes
            .get(start..start + 8)
            .is_some_and(|digits| digits.iter().all(u8::is_ascii_digit))
        {
            return Some(String::from_utf8_lossy(&bytes[start..start + 8]).into_owned());
        }
        cursor = start;
    }
    None
}

fn resolve_map_names(
    root: &Path,
    refs: &BTreeMap<String, BTreeSet<String>>,
    loc: &BTreeMap<String, String>,
) -> Result<BTreeMap<String, BTreeSet<String>>, Box<dyn std::error::Error>> {
    let mut map_cache = BTreeMap::new();
    for map in refs.values().flatten() {
        if map_cache.contains_key(map) {
            continue;
        }
        let path = root.join("misc/GUI-WorldData/Maps").join(map);
        let title = if path.is_file() {
            first_zone_key(&fs::read(path)?)
        } else {
            None
        };
        map_cache.insert(map.clone(), title.and_then(|key| loc.get(&key).cloned()));
    }
    let mut names = BTreeMap::new();
    for (zone, maps) in refs {
        let values = maps
            .iter()
            .filter_map(|map| map_cache.get(map).and_then(Clone::clone))
            .collect();
        names.insert(zone.clone(), values);
    }
    Ok(names)
}

fn parse_map_associations(
    blob: &[u8],
    paths: &BTreeSet<String>,
) -> BTreeMap<String, BTreeSet<String>> {
    let marker = b"|GUI|WorldData|Maps/";
    let mut maprefs = Vec::new();
    let mut pos = 0;
    while let Some(index) = find_from(blob, marker, pos) {
        let start = index + marker.len();
        if let Some(end_relative) = blob[start..]
            .windows(4)
            .position(|window| window == b".xml")
        {
            let end = start + end_relative + 4;
            maprefs.push((end, String::from_utf8_lossy(&blob[start..end]).into_owned()));
            pos = end;
        } else {
            pos = start;
        }
    }
    let mut associations: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for path in paths {
        let mut scan = 0;
        while let Some(index) = find_from(blob, path.as_bytes(), scan) {
            let preceding = maprefs.partition_point(|(end, _)| *end <= index);
            if preceding > 0 {
                let (end, map) = &maprefs[preceding - 1];
                if index - *end <= 64 {
                    associations
                        .entry(path.clone())
                        .or_default()
                        .insert(map.clone());
                }
            }
            scan = index + path.len();
        }
    }
    associations
}

fn resolve_compass_children(
    zones: &BTreeMap<String, RawZone>,
    map_refs: &BTreeMap<String, BTreeSet<String>>,
    compass_lang: &BTreeMap<String, String>,
) -> BTreeMap<String, BTreeSet<String>> {
    let mut candidates: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut children_by_exit: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (child_path, child) in zones {
        let triggers = fs::read(child.directory.join("triggers.xml")).unwrap_or_default();
        let volumes = fs::read(child.directory.join("volumes.xml")).unwrap_or_default();
        for token in exit_tokens(&triggers)
            .into_iter()
            .chain(exit_tokens(&volumes))
        {
            children_by_exit
                .entry(token)
                .or_default()
                .push(child_path.clone());
        }
    }
    for (parent_path, parent) in zones {
        let triggers_path = parent.directory.join("triggers.xml");
        let volumes_path = parent.directory.join("volumes.xml");
        let (Ok(triggers), Ok(volumes)) = (fs::read(triggers_path), fs::read(volumes_path)) else {
            continue;
        };
        let parent_maps = map_refs.get(parent_path).cloned().unwrap_or_default();
        if parent_maps.is_empty() {
            continue;
        }
        for (poi_token, loc_key) in poi_trigger_links(&triggers, &volumes) {
            let parent_target = format!("Teleport-{poi_token}");
            if !contains_exact_token(&volumes, parent_target.as_bytes())
                && !contains_exact_token(&triggers, parent_target.as_bytes())
            {
                continue;
            }
            let Some(display_name) = compass_lang.get(&loc_key) else {
                continue;
            };
            for child_path in children_by_exit.get(&poi_token).into_iter().flatten() {
                if child_path == parent_path
                    || !map_refs
                        .get(child_path)
                        .is_some_and(|maps| !maps.is_disjoint(&parent_maps))
                {
                    continue;
                }
                candidates
                    .entry(child_path.clone())
                    .or_default()
                    .insert(display_name.clone());
            }
        }
    }
    candidates
}

fn poi_trigger_links(triggers: &[u8], volumes: &[u8]) -> BTreeSet<(String, String)> {
    let trigger_marker = b"Trigger-POI-";
    let next_trigger = b"Trigger-";
    let mut links = BTreeSet::new();
    let mut cursor = 0;
    while let Some(start) = find_from(triggers, trigger_marker, cursor) {
        let token_start = start + trigger_marker.len();
        let Some(token) = read_token(triggers, token_start) else {
            cursor = token_start;
            continue;
        };
        let record_start = token_start + token.len();
        let record_end = find_from(triggers, next_trigger, record_start).unwrap_or(triggers.len());
        let record = &triggers[record_start..record_end];
        let expected_volume = format!("Enter_POI Volume-{}", String::from_utf8_lossy(token));
        if !contains_exact_token(record, expected_volume.as_bytes()) {
            cursor = record_end;
            continue;
        }
        let volume_name = format!("Volume-{}", String::from_utf8_lossy(token));
        if !contains_exact_token(volumes, volume_name.as_bytes()) {
            cursor = record_end;
            continue;
        }
        let keys = resource_keys(record, b"WizardCompassLocs_");
        if keys.len() == 1 {
            links.insert((
                String::from_utf8_lossy(token).into_owned(),
                keys.into_iter().next().unwrap(),
            ));
        }
        cursor = record_end;
    }
    links
}

fn read_token(bytes: &[u8], start: usize) -> Option<&[u8]> {
    let mut end = bytes[start..]
        .iter()
        .position(|byte| !byte.is_ascii_alphanumeric() && *byte != b'_' && *byte != b'-')?
        + start;
    if end > start && bytes[end - 1] == b'H' && bytes.get(end) == Some(&0) {
        end -= 1;
    }
    (end > start).then_some(&bytes[start..end])
}

fn exit_tokens(bytes: &[u8]) -> BTreeSet<String> {
    let prefix = b"Teleport-Exit";
    let mut tokens = BTreeSet::new();
    let mut cursor = 0;
    while let Some(start) = find_from(bytes, prefix, cursor) {
        let token_start = start + prefix.len();
        if let Some(token) = read_token(bytes, token_start) {
            tokens.insert(String::from_utf8_lossy(token).into_owned());
            cursor = token_start + token.len();
        } else {
            cursor = token_start;
        }
    }
    tokens
}

fn contains_exact_token(haystack: &[u8], token: &[u8]) -> bool {
    let mut cursor = 0;
    while let Some(start) = find_from(haystack, token, cursor) {
        let end = start + token.len();
        let boundary = haystack
            .get(end)
            .is_none_or(|byte| !byte.is_ascii_alphanumeric() && *byte != b'_' && *byte != b'-')
            || (haystack.get(end) == Some(&b'H') && haystack.get(end + 1) == Some(&0));
        if boundary {
            return true;
        }
        cursor = end;
    }
    false
}

fn find_from(haystack: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    if needle.is_empty() || from > haystack.len() {
        return None;
    }
    haystack[from..]
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|index| index + from)
}

pub fn write_json(zones: &[Zone], output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let parent = output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;
    let data: BTreeMap<&str, serde_json::Value> = zones
        .iter()
        .map(|zone| {
            (
                zone.path.as_str(),
                serde_json::json!({
                    "world": zone.world_name.as_deref().unwrap_or("Unknown"),
                    "zone": zone.selected.as_ref().map_or("Unknown", |candidate| candidate.name.as_str()),
                }),
            )
        })
        .collect();
    let mut bytes = serde_json::to_vec_pretty(&data)?;
    bytes.push(b'\n');
    fs::write(output, bytes)?;
    Ok(())
}

pub fn write_diagnostics(zones: &[Zone], output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let parent = output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;
    let entries: Vec<_> = zones
        .iter()
        .map(|zone| {
            let candidates: Vec<_> = zone
                .candidates
                .iter()
                .map(|candidate| {
                    serde_json::json!({
                        "source": candidate.source.label(),
                        "name": candidate.name,
                        "provenance": candidate_provenance(candidate.source),
                    })
                })
                .collect();
            let selected = zone.selected.as_ref().map(|candidate| {
                serde_json::json!({
                    "source": candidate.source.label(),
                    "name": candidate.name,
                    "confidence": zone.confidence.label(),
                    "provenance": zone.selection_provenance,
                })
            });
            serde_json::json!({
                "path": zone.path,
                "world": {
                    "lookup_key": zone.world_lookup_key,
                    "name": zone.world_name.as_deref().unwrap_or("Unknown"),
                    "source": "WorldNames",
                    "confidence": if zone.world_name.is_some() { "verified" } else { "unknown" },
                    "provenance": world_provenance(zone.world_name.is_some()),
                },
                "header_field_value": zone.header_field_value,
                "header_localized_value": zone.header_localized_value,
                "header_wizard_zone": zone.header_wizard_zone,
                "candidates": candidates,
                "selected": selected,
                "selection_provenance": zone.selection_provenance,
                "confidence": zone.confidence.label(),
                "ambiguity": zone.ambiguity,
                "conflicts": zone.conflicts,
                "unresolved": zone.unresolved,
            })
        })
        .collect();
    let mut bytes = serde_json::to_vec_pretty(&entries)?;
    bytes.push(b'\n');
    let sidecar = diagnostics_path(output);
    fs::write(sidecar, bytes)?;
    Ok(())
}

fn world_provenance(resolved: bool) -> &'static str {
    if resolved {
        "exact canonical path first component resolved through WorldNames.lang"
    } else {
        "exact canonical path first component has no WorldNames.lang localization; world left Unknown"
    }
}

pub fn diagnostics_path(output: &Path) -> PathBuf {
    let stem = output.file_stem().unwrap_or_default().to_string_lossy();
    output.with_file_name(format!("{stem}.diagnostics.json"))
}

fn candidate_provenance(source: Source) -> &'static str {
    match source {
        Source::CompassPoi => "proven parent POI localization and matching child transition chain",
        Source::ZoneHeader => {
            "exact current-zone header localization key resolved through Zone.lang"
        }
        Source::HousingHeader => {
            "exact current-zone header localization key resolved through Housing.lang"
        }
        Source::WizardZone => "exact zone-header WizardZone field resolved through WizardZone.lang",
        Source::SharedMap => "DoodleMapMap association to map resource and Zone.lang key",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header(path: &str, value: &str) -> Vec<u8> {
        let mut bytes = b"\xf8\x63\x69\x81".to_vec();
        bytes.extend((path.len() as u16).to_le_bytes());
        bytes.extend(path.as_bytes());
        bytes.extend([0xe8, 0, 0, 0]);
        bytes.extend(b"\x6e\xec\xf6\x74");
        bytes.extend((value.len() as u16).to_le_bytes());
        bytes.extend(value.as_bytes());
        bytes
    }

    fn one(name: &str) -> BTreeSet<String> {
        [name.to_owned()].into_iter().collect()
    }

    #[test]
    fn selects_proven_poi_before_other_candidates() {
        let poi = one("Pit of the Noxii");
        let wizard = one("Mount Olympus");
        let map = one("Mount Olympus");
        let (selected, confidence, ambiguity, conflicts) =
            select_candidate(&poi, &BTreeSet::new(), &BTreeSet::new(), &wizard, &map);
        assert_eq!(selected.unwrap().name, "Pit of the Noxii");
        assert_eq!(confidence, Confidence::Verified);
        assert_eq!(ambiguity, None);
        assert_eq!(conflicts.len(), 2);
    }

    #[test]
    fn wizard_zone_only_is_explicit_fallback() {
        let wizard = one("Stonegaze’s Antichamber");
        let (selected, confidence, ambiguity, _) = select_candidate(
            &BTreeSet::new(),
            &BTreeSet::new(),
            &BTreeSet::new(),
            &wizard,
            &BTreeSet::new(),
        );
        assert_eq!(selected.unwrap().name, "Stonegaze’s Antichamber");
        assert_eq!(confidence, Confidence::UnverifiedFallback);
        assert!(ambiguity.unwrap().contains("unverified fallback"));
    }

    #[test]
    fn conflicting_wizard_zone_and_map_select_fallback_and_report_conflict() {
        let wizard = one("Infirmary");
        let map = one("Arcanum");
        let (selected, confidence, ambiguity, conflicts) = select_candidate(
            &BTreeSet::new(),
            &BTreeSet::new(),
            &BTreeSet::new(),
            &wizard,
            &map,
        );
        assert_eq!(selected.unwrap().name, "Infirmary");
        assert_eq!(confidence, Confidence::UnverifiedFallback);
        assert!(ambiguity.unwrap().contains("candidate conflict"));
        assert!(conflicts
            .iter()
            .any(|conflict| conflict.contains("WizardZone")));
    }

    #[test]
    fn named_fallback_regressions_preserve_header_localizations() {
        let cases = [
            ("Gorgon Cave", "Aquila", "Garden of Hesperides"),
            ("Vestrilund", "Vestrilund", "Cenote"),
            ("Zeus Exalted Duel", "Zeus Exalted Duel", "Mount Olympus"),
            ("Triton", "Triton Avenue", "Crab Alley"),
        ];
        for (expected_specific_name, wizard_name, map_name) in cases {
            let wizard = one(wizard_name);
            let map = one(map_name);
            let (selected, confidence, ambiguity, _) = select_candidate(
                &BTreeSet::new(),
                &BTreeSet::new(),
                &BTreeSet::new(),
                &wizard,
                &map,
            );
            assert_eq!(
                selected.unwrap().name,
                wizard_name,
                "{expected_specific_name}"
            );
            assert_eq!(confidence, Confidence::UnverifiedFallback);
            assert!(ambiguity.is_some(), "{expected_specific_name}");
        }
    }

    #[test]
    fn no_candidate_is_unknown_and_malformed_map_is_rejected() {
        assert!(!valid_map_title("Wizard City|Firecat Alley"));
        let (selected, confidence, ambiguity, _) = select_candidate(
            &BTreeSet::new(),
            &BTreeSet::new(),
            &BTreeSet::new(),
            &BTreeSet::new(),
            &BTreeSet::new(),
        );
        assert!(selected.is_none());
        assert_eq!(confidence, Confidence::Unknown);
        assert!(ambiguity.is_none());
    }

    #[test]
    fn multiple_map_names_use_header_fallback_and_stay_ambiguous() {
        let wizard = one("Triton Avenue");
        let map = ["Crab Alley".to_owned(), "Deep Warrens".to_owned()]
            .into_iter()
            .collect();
        let (selected, confidence, ambiguity, _) = select_candidate(
            &BTreeSet::new(),
            &BTreeSet::new(),
            &BTreeSet::new(),
            &wizard,
            &map,
        );
        assert_eq!(selected.unwrap().name, "Triton Avenue");
        assert_eq!(confidence, Confidence::UnverifiedFallback);
        assert!(ambiguity.unwrap().contains("conflicting shared-map"));
    }

    #[test]
    fn poi_requires_trigger_volume_key_and_exact_token() {
        let trigger = b"Trigger-POI-PitOfTheNoxii\0Enter_POI Volume-PitOfTheNoxii\0WizardCompassLocs_00000135\0Trigger-Other";
        let volumes = b"Volume-PitOfTheNoxii\0TeleportVol_Teleport-PitOfTheNoxii";
        assert_eq!(
            poi_trigger_links(trigger, volumes),
            [("PitOfTheNoxii".into(), "00000135".into())]
                .into_iter()
                .collect()
        );
        assert!(poi_trigger_links(trigger, b"Volume-PitOfTheNoxiiOther").is_empty());
    }

    #[test]
    fn map_title_key_is_extracted_from_binary_strings() {
        assert_eq!(
            first_zone_key(b"Zone_00001123\0Zone_00001124"),
            Some("00001123".into())
        );
    }

    #[test]
    fn wizard_zone_reads_only_immediate_numeric_or_symbolic_header_field() {
        let path = "WizardCity/WC_Streets/WC_Triton";
        let mut bytes = header(path, "WizardZone_TritonAvenue");
        bytes.extend(b"later\0WizardZone_00001603");
        assert_eq!(
            zone_header_field_value(&bytes, path),
            Some("WizardZone_TritonAvenue".into())
        );
        assert_eq!(
            zone_header_field_value(&header(path, "WizardZone_00001603"), path),
            Some("WizardZone_00001603".into())
        );
        assert_eq!(
            zone_header_field_value(b"path\0WizardZone_00001603", path),
            None
        );
    }

    #[test]
    fn direct_zone_and_housing_header_candidates_are_verified() {
        let zone = one("Hall of the Ice Forge");
        let map = one("Hall of the Ice Forge");
        let (selected, confidence, ambiguity, _) = select_candidate(
            &BTreeSet::new(),
            &zone,
            &BTreeSet::new(),
            &BTreeSet::new(),
            &map,
        );
        assert_eq!(selected.unwrap().source, Source::ZoneHeader);
        assert_eq!(confidence, Confidence::Verified);
        assert_eq!(ambiguity, None);

        let housing = one("Meadows at Dusk");
        let (selected, confidence, _, _) = select_candidate(
            &BTreeSet::new(),
            &BTreeSet::new(),
            &housing,
            &BTreeSet::new(),
            &BTreeSet::new(),
        );
        assert_eq!(selected.unwrap().name, "Meadows at Dusk");
        assert_eq!(confidence, Confidence::Verified);
    }

    #[test]
    fn symbolic_wizard_zone_entries_with_explicit_descriptions_resolve() {
        let lang = "1:WizardZone\nTheCommons\nName of commons zone\nThe Commons\nRavenwood\nzone name for Ravenwood\nRavenwood\nUnicornWay\nZone name for Unicorn Way\nUnicorn Way\n";
        let parsed = parse_lang(&utf16(lang)).unwrap();
        assert_eq!(
            parsed.get("TheCommons").map(String::as_str),
            Some("The Commons")
        );
        assert_eq!(
            parsed.get("Ravenwood").map(String::as_str),
            Some("Ravenwood")
        );
        assert_eq!(
            parsed.get("UnicornWay").map(String::as_str),
            Some("Unicorn Way")
        );
    }

    #[test]
    fn test_placeholder_zone_header_is_not_a_display_candidate() {
        assert!(!valid_direct_header_name("TEST-Interior"));
        let candidate = candidate_from_header(
            Some("Zone_00001518"),
            &[("00001518".to_owned(), "TEST-Interior".to_owned())]
                .into_iter()
                .collect(),
            &BTreeMap::new(),
            &BTreeMap::new(),
        );
        assert!(candidate.is_none());
    }

    fn utf16(text: &str) -> Vec<u8> {
        let mut bytes = vec![0xff, 0xfe];
        for unit in text.encode_utf16() {
            bytes.extend(unit.to_le_bytes());
        }
        bytes
    }
}
