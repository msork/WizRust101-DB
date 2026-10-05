mod zones;

use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::PathBuf,
    process,
};
use zones::{Candidate, Source, Zone};

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "generate".into());
    let mut input = PathBuf::from("raw");
    let mut output = PathBuf::from("out/zones.json");
    let mut reference = PathBuf::from("tmp/zones.json");
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--input" => input = args.next().ok_or("--input needs a path")?.into(),
            "--output" => output = args.next().ok_or("--output needs a path")?.into(),
            "--reference" => reference = args.next().ok_or("--reference needs a path")?.into(),
            "-h" | "--help" => {
                println!("Usage: wizrust101-db [generate|compare] [--input DIR] [--output FILE] [--reference FILE]");
                return Ok(());
            }
            _ => return Err(format!("unknown argument: {argument}").into()),
        }
    }
    if command != "generate" && command != "compare" {
        return Err(format!("unknown command: {command}").into());
    }

    let discovered = zones::discover(&input)?;
    reject_output_under_raw(&input, &output)?;
    zones::write_json(&discovered, &output)?;
    zones::write_diagnostics(&discovered, &output)?;
    report_generation(&discovered, &output);
    if command == "compare" {
        compare(&discovered, &output, &reference)?;
    }
    Ok(())
}

fn reject_output_under_raw(
    input: &std::path::Path,
    output: &std::path::Path,
) -> std::io::Result<()> {
    let raw_root = fs::canonicalize(input)?;
    for candidate in [output.to_path_buf(), zones::diagnostics_path(output)] {
        let resolved = resolve_destination(&candidate)?;
        if resolved.starts_with(&raw_root) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "refusing to write output beneath raw input root {}: {}",
                    raw_root.display(),
                    resolved.display()
                ),
            ));
        }
    }
    Ok(())
}

fn resolve_destination(path: &std::path::Path) -> std::io::Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()?.join(path)
    };
    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }

    let mut existing = normalized.clone();
    let mut suffix = Vec::new();
    while !existing.exists() {
        let name = existing.file_name().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("no existing parent for output path {}", path.display()),
            )
        })?;
        suffix.push(name.to_os_string());
        existing.pop();
    }
    let mut resolved = fs::canonicalize(existing)?;
    for name in suffix.into_iter().rev() {
        resolved.push(name);
    }
    Ok(resolved)
}

fn report_generation(zones: &[Zone], output: &std::path::Path) {
    let verified = zones
        .iter()
        .filter(|zone| zone.confidence == zones::Confidence::Verified)
        .count();
    let fallback = zones
        .iter()
        .filter(|zone| zone.confidence == zones::Confidence::UnverifiedFallback)
        .count();
    let unknown = zones
        .iter()
        .filter(|zone| zone.confidence == zones::Confidence::Unknown)
        .count();
    let resolved_worlds: BTreeSet<&str> = zones
        .iter()
        .filter_map(|zone| zone.world_name.as_deref())
        .collect();
    let resolved_world_count = zones
        .iter()
        .filter(|zone| zone.world_name.is_some())
        .count();
    eprintln!(
        "wrote {} zones to {} (zone verified={verified}, unverified_fallback={fallback}, Unknown={unknown}; world resolved={resolved_world_count}, unknown={}, distinct={}); diagnostics={}",
        zones.len(), output.display(), zones.len() - resolved_world_count,
        resolved_worlds.len(), zones::diagnostics_path(output).display()
    );
    eprintln!(
        "world names: {}",
        resolved_worlds.into_iter().collect::<Vec<_>>().join(", ")
    );
    report_source_summary(zones);
    for zone in zones {
        if zone.confidence == zones::Confidence::Unknown
            || zone.ambiguity.is_some()
            || !zone.conflicts.is_empty()
        {
            eprintln!("{}", trace(zone));
        }
    }
}

fn report_source_summary(zones: &[Zone]) {
    let mut candidates = BTreeMap::<Source, usize>::new();
    let mut selected = BTreeMap::<Source, usize>::new();
    let mut ambiguous = 0;
    let mut conflicts = 0;
    for zone in zones {
        let present: BTreeSet<Source> = zone
            .candidates
            .iter()
            .map(|candidate| candidate.source)
            .collect();
        for source in present {
            *candidates.entry(source).or_default() += 1;
        }
        if let Some(candidate) = &zone.selected {
            *selected.entry(candidate.source).or_default() += 1;
        }
        ambiguous += usize::from(zone.ambiguity.is_some());
        conflicts += usize::from(!zone.conflicts.is_empty());
    }
    eprintln!(
        "candidate zones: CompassPoi={}, ZoneHeader={}, HousingHeader={}, WizardZone={}, SharedMap={}; selected: CompassPoi={}, ZoneHeader={}, HousingHeader={}, WizardZone={}, SharedMap={}; candidate ambiguities={ambiguous}; conflicting candidates={conflicts}",
        candidates.get(&Source::CompassPoi).copied().unwrap_or_default(),
        candidates.get(&Source::ZoneHeader).copied().unwrap_or_default(),
        candidates.get(&Source::HousingHeader).copied().unwrap_or_default(),
        candidates.get(&Source::WizardZone).copied().unwrap_or_default(),
        candidates.get(&Source::SharedMap).copied().unwrap_or_default(),
        selected.get(&Source::CompassPoi).copied().unwrap_or_default(),
        selected.get(&Source::ZoneHeader).copied().unwrap_or_default(),
        selected.get(&Source::HousingHeader).copied().unwrap_or_default(),
        selected.get(&Source::WizardZone).copied().unwrap_or_default(),
        selected.get(&Source::SharedMap).copied().unwrap_or_default(),
    );
}

fn trace(zone: &Zone) -> String {
    let candidates = zone
        .candidates
        .iter()
        .map(|candidate| format!("{}={:?}", candidate.source.label(), candidate.name))
        .collect::<Vec<_>>()
        .join(", ");
    let selected = zone
        .selected
        .as_ref()
        .map(format_selected)
        .unwrap_or_else(|| "Unknown".into());
    let mut details = vec![
        format!("candidates=[{candidates}]"),
        format!("selected={selected}"),
        format!("confidence={}", zone.confidence.label()),
        format!("provenance={:?}", zone.selection_provenance),
    ];
    if let Some(header) = &zone.header_field_value {
        details.push(format!("header_field_value={header:?}"));
    }
    if let Some(localized) = &zone.header_localized_value {
        details.push(format!("header_localized_value={localized:?}"));
    }
    if let Some(ambiguity) = &zone.ambiguity {
        details.push(format!("ambiguity={ambiguity:?}"));
    }
    if zone.unresolved {
        details.push("unresolved=true".into());
    }
    if !zone.conflicts.is_empty() {
        details.push(format!("conflicts=[{}]", zone.conflicts.join("; ")));
    }
    format!("zone {}: {}", zone.path, details.join(" "))
}

fn format_selected(candidate: &Candidate) -> String {
    format!("{}={:?}", candidate.source.label(), candidate.name)
}

fn compare(
    zones: &[Zone],
    output: &std::path::Path,
    reference: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let json: serde_json::Value = serde_json::from_slice(&fs::read(reference)?)?;
    let expected: BTreeMap<String, String> = json
        .as_object()
        .ok_or("reference must be a JSON object")?
        .iter()
        .filter_map(|(key, value)| {
            let zone = value
                .as_str()
                .or_else(|| value.get("zone").and_then(serde_json::Value::as_str))?;
            Some((key.clone(), zone.to_owned()))
        })
        .filter(|(key, _)| key.contains('/'))
        .collect();
    let generated: BTreeMap<String, serde_json::Value> =
        serde_json::from_slice(&fs::read(output)?)?;
    let by_path: BTreeMap<&str, &Zone> = zones
        .iter()
        .map(|zone| (zone.path.as_str(), zone))
        .collect();
    let (mut exact, mut mismatch, mut missing, mut new) = (0, 0, 0, 0);
    for (key, reference_name) in &expected {
        match generated
            .get(key)
            .and_then(|entry| entry.get("zone"))
            .and_then(serde_json::Value::as_str)
        {
            Some("Unknown") => {
                missing += 1;
                if let Some(zone) = by_path.get(key.as_str()) {
                    eprintln!("unresolved reference zone: {}", trace(zone));
                } else {
                    eprintln!("reference zone absent from raw: {key}");
                }
            }
            None => {
                missing += 1;
                if let Some(zone) = by_path.get(key.as_str()) {
                    eprintln!("unresolved reference zone: {}", trace(zone));
                } else {
                    eprintln!("reference zone absent from raw: {key}");
                }
            }
            Some(actual) if actual == reference_name => exact += 1,
            Some(actual) => {
                mismatch += 1;
                eprintln!(
                    "reference conflict {key}: reference={reference_name:?}, selected={actual:?}"
                );
                if let Some(zone) = by_path.get(key.as_str()) {
                    eprintln!("{}", trace(zone));
                }
            }
        }
    }
    for (key, value) in &generated {
        if !expected.contains_key(key)
            && value.get("zone").and_then(serde_json::Value::as_str) != Some("Unknown")
        {
            new += 1;
            if let Some(zone) = by_path.get(key.as_str()) {
                eprintln!("generated-only zone: {}", trace(zone));
            }
        }
    }
    println!(
        "reference zone keys: {}; resolved/reference: {}/{}; exact matches: {exact}; mismatches: {mismatch}; unresolved/missing: {missing}; new zones: {new}",
        expected.len(),
        expected.len() - missing,
        expected.len()
    );
    Ok(())
}
