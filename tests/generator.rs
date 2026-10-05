use std::{fs, io::Write, path::Path, process::Command};

fn utf16(text: &str) -> Vec<u8> {
    let mut bytes = vec![0xff, 0xfe];
    for unit in text.encode_utf16() {
        bytes.extend(unit.to_le_bytes());
    }
    bytes
}

fn make_zone(root: &std::path::Path, folder: &str, path: &str, wizard_key: Option<&str>) {
    make_zone_with_header(
        root,
        folder,
        path,
        wizard_key.map(|key| format!("WizardZone_{key}")),
    );
}

fn make_zone_with_header(
    root: &std::path::Path,
    folder: &str,
    path: &str,
    header_value: Option<String>,
) {
    let directory = root.join(folder);
    fs::create_dir_all(&directory).unwrap();
    let mut bytes = b"\xf8\x63\x69\x81".to_vec();
    bytes.extend((path.len() as u16).to_le_bytes());
    bytes.extend(path.as_bytes());
    bytes.extend([0xe8, 0, 0, 0]);
    if let Some(value) = header_value {
        bytes.extend(b"\x6e\xec\xf6\x74");
        bytes.extend((value.len() as u16).to_le_bytes());
        bytes.extend(value.as_bytes());
    }
    fs::write(directory.join("gamedata.bin"), bytes).unwrap();
}

fn map_record(map: &str, path: &str) -> Vec<u8> {
    format!("|GUI|WorldData|Maps/{map}\0{path}\0").into_bytes()
}

#[test]
fn resolves_candidates_writes_unknowns_and_records_header_provenance() {
    let temp = std::env::temp_dir().join(format!("wizrust-evidence-{}", std::process::id()));
    let raw = temp.join("raw");
    let maps = raw.join("misc/GUI-WorldData/Maps");
    fs::create_dir_all(&maps).unwrap();

    let parent = "Aquila/AQ_Z01_MountOlympus";
    let pit = "Aquila/Interiors/AQ_Z01_PitOfNoxii";
    let hall = "Aquila/Interiors/AQ_SkelKey_Hall_01";
    let conflicted = "Arcanum/Interiors/AR_Z01_Infirmary";
    let unresolved = "NoWorld/NoMap";
    let phantom = "ThePhantomZoneWorld/DoodleDougPhantomZoneP";
    let alias = "DragonSpire/DS_Hub_Cathedral";
    let special = "G14_DM/DM_Z01_CastleDarkmoor";
    let zigazag = "G14_HS/HS_Z01_ZigazagUpper";
    let grizzleheim_lite = "GrizzleheimLite/GH_GrizzleheimHubLite";
    let event_root = "MonthlyEvents/ME_Sinbad/ME_Sinbad_Int01";
    let house = "Housing_AV_BAC/Exterior";
    let dusk = "Housing_BuildACastleProto/Exterior_Dusk";
    make_zone(&raw, "parent", parent, Some("00001029"));
    make_zone(&raw, "pit", pit, Some("00001029"));
    make_zone(&raw, "hall", hall, Some("00001603"));
    make_zone(&raw, "infirmary", conflicted, Some("00001387"));
    make_zone(&raw, "unknown", unresolved, None);
    make_zone(&raw, "phantom", phantom, None);
    make_zone(&raw, "alias", alias, None);
    make_zone(&raw, "special", special, None);
    make_zone(&raw, "zigazag", zigazag, None);
    make_zone(&raw, "grizzleheim-lite", grizzleheim_lite, None);
    make_zone(&raw, "event-root", event_root, None);
    make_zone_with_header(&raw, "house", house, Some("Zone_00001374".to_owned()));
    make_zone_with_header(&raw, "dusk", dusk, Some("Housing_00000832".to_owned()));
    fs::OpenOptions::new()
        .append(true)
        .open(raw.join("unknown/gamedata.bin"))
        .unwrap()
        .write_all(b"\0later WizardZone_00001603")
        .unwrap();

    fs::write(maps.join("olympus.xml"), b"Zone_00000001\0Zone_00000002").unwrap();
    let mut doodle = map_record("olympus.xml", parent);
    doodle.extend(map_record("olympus.xml", pit));
    doodle.extend(map_record("arcanum.xml", conflicted));
    fs::write(maps.join("arcanum.xml"), b"Zone_00000003").unwrap();
    fs::create_dir_all(raw.join("misc/Root")).unwrap();
    fs::write(raw.join("misc/Root/DoodleMapMap.xml"), doodle).unwrap();

    let parent_dir = raw.join("parent");
    fs::write(
        parent_dir.join("triggers.xml"),
        b"Trigger-POI-PitOfTheNoxii\0Enter_POI Volume-PitOfTheNoxii\0WizardCompassLocs_00000135\0Trigger-Other\0",
    )
    .unwrap();
    fs::write(
        parent_dir.join("volumes.xml"),
        b"Volume-PitOfTheNoxii\0TeleportVol_Teleport-PitOfTheNoxii\0",
    )
    .unwrap();
    fs::write(
        raw.join("pit/triggers.xml"),
        b"Trigger-Exit\0Teleport-ExitPitOfTheNoxii\0",
    )
    .unwrap();
    fs::write(raw.join("pit/volumes.xml"), b"Teleport-ExitPitOfTheNoxii\0").unwrap();

    let locale = raw.join("misc/Root/Locale/en-US");
    fs::create_dir_all(&locale).unwrap();
    fs::write(
        locale.join("Zone.lang"),
        utf16("1:Zone\n00000001\n\nMount Olympus\n00000003\n\nArcanum\n00001374\n\nAvalon Castle Plot\n"),
    )
    .unwrap();
    fs::write(
        locale.join("WizardZone.lang"),
        utf16("1:WizardZone\n00001029\n\nMount Olympus\n00001603\n\nStonegaze’s Antichamber\n00001387\n\nInfirmary\nTritonAvenue\n\nTriton Avenue\n"),
    )
    .unwrap();
    fs::write(
        locale.join("Housing.lang"),
        utf16("1:Housing\n00000832\n\nMeadows at Dusk\n"),
    )
    .unwrap();
    fs::write(
        locale.join("WizardCompassLocs.lang"),
        utf16("1:WizardCompassLocs\n00000135\n\nPit of the Noxii\n"),
    )
    .unwrap();
    fs::write(
        locale.join("WorldNames.lang"),
        utf16("1:WorldNames\nAquila\n\nAquila\nDragonSpire\n\nDragonspyre\nG14_DM\n\nCastle Darkmoor\nWizardCity\n\nWizard City\n"),
    )
    .unwrap();
    let triton = "WizardCity/WC_Streets/WC_Triton";
    make_zone(&raw, "triton", triton, Some("TritonAvenue"));
    let reference = temp.join("reference.json");
    fs::write(
        &reference,
        format!(r#"{{"{pit}":"Pit of Noxxi","{conflicted}":"Infirmary"}}"#),
    )
    .unwrap();
    let output = temp.join("nested/zones.json");
    let result = Command::new(env!("CARGO_BIN_EXE_wizrust101-db"))
        .args(["compare", "--input"])
        .arg(&raw)
        .arg("--output")
        .arg(&output)
        .arg("--reference")
        .arg(&reference)
        .output()
        .unwrap();
    assert!(result.status.success());
    let json: serde_json::Value = serde_json::from_slice(&fs::read(&output).unwrap()).unwrap();
    let generated = json.as_object().unwrap();
    assert_eq!(generated.len(), 14);
    assert!(generated.values().all(|entry| {
        entry.is_object() && entry["world"].is_string() && entry["zone"].is_string()
    }));
    assert_eq!(json[pit]["zone"], "Pit of the Noxii");
    assert_eq!(json[pit]["world"], "Aquila");
    assert_eq!(json[hall]["zone"], "Stonegaze’s Antichamber");
    assert_eq!(json[conflicted]["zone"], "Infirmary");
    assert_eq!(json[unresolved]["zone"], "NoMap");
    assert_eq!(json[unresolved]["world"], "Unknown");
    assert_eq!(json[phantom]["zone"], "DoodleDougPhantomZoneP");
    assert_eq!(json[phantom]["world"], "Unknown");
    assert_eq!(json[triton]["zone"], "Triton Avenue");
    assert_eq!(json[house]["zone"], "Avalon Castle Plot");
    assert_eq!(json[house]["world"], "Avalon");
    assert_eq!(json[dusk]["zone"], "Meadows at Dusk");
    assert_eq!(json[alias]["world"], "Dragonspyre");
    assert_eq!(json[special]["world"], "Castle Darkmoor");
    assert_eq!(json[triton]["world"], "Wizard City");
    assert_eq!(json[zigazag]["world"], "Zigazag");
    assert_eq!(json[grizzleheim_lite]["world"], "Grizzleheim");
    assert_eq!(json[event_root]["world"], "Unknown");
    let diagnostics_path = output.with_file_name("zones.diagnostics.json");
    let diagnostics: serde_json::Value =
        serde_json::from_slice(&fs::read(&diagnostics_path).unwrap()).unwrap();
    let triton_diagnostic = diagnostics
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["path"] == triton)
        .unwrap();
    assert_eq!(
        triton_diagnostic["header_wizard_zone"],
        "WizardZone_TritonAvenue"
    );
    assert_eq!(
        triton_diagnostic["selected"]["confidence"],
        "unverified_fallback"
    );
    let unresolved_diagnostic = diagnostics
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["path"] == unresolved)
        .unwrap();
    assert_eq!(unresolved_diagnostic["emitted_zone"]["name"], "NoMap");
    assert_eq!(
        unresolved_diagnostic["emitted_zone"]["source"],
        "canonical_leaf_fallback"
    );
    assert_eq!(
        unresolved_diagnostic["emitted_zone"]["confidence"],
        "unverified_fallback"
    );
    assert_eq!(unresolved_diagnostic["world"]["source"], "unknown");
    let phantom_diagnostic = diagnostics
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["path"] == phantom)
        .unwrap();
    assert_eq!(
        phantom_diagnostic["emitted_zone"]["name"],
        "DoodleDougPhantomZoneP"
    );
    assert_eq!(
        phantom_diagnostic["emitted_zone"]["source"],
        "canonical_leaf_fallback"
    );
    assert_eq!(phantom_diagnostic["world"]["name"], "Unknown");
    let house_diagnostic = diagnostics
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["path"] == house)
        .unwrap();
    assert_eq!(house_diagnostic["selected"]["source"], "ZoneHeader");
    assert_eq!(house_diagnostic["selected"]["confidence"], "verified");
    assert_eq!(house_diagnostic["world"]["name"], "Avalon");
    assert_eq!(
        house_diagnostic["world"]["source"],
        "canonical_alias_fallback"
    );
    assert_eq!(house_diagnostic["world"]["confidence"], "fallback");
    let zigazag_diagnostic = diagnostics
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["path"] == zigazag)
        .unwrap();
    assert_eq!(
        zigazag_diagnostic["world"]["source"],
        "canonical_alias_fallback"
    );
    let event_diagnostic = diagnostics
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["path"] == event_root)
        .unwrap();
    assert_eq!(event_diagnostic["world"]["source"], "unknown");
    let wizard_city_diagnostic = diagnostics
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["path"] == triton)
        .unwrap();
    assert_eq!(wizard_city_diagnostic["world"]["source"], "localized");
    let dusk_diagnostic = diagnostics
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["path"] == dusk)
        .unwrap();
    assert_eq!(dusk_diagnostic["selected"]["source"], "HousingHeader");
    let stderr = String::from_utf8(result.stderr).unwrap();
    let stdout = String::from_utf8(result.stdout).unwrap();
    assert!(stderr.contains("CompassPoi=\"Pit of the Noxii\""));
    assert!(stderr.contains("WizardZone=\"Stonegaze’s Antichamber\""));
    assert!(stderr.contains("SharedMap=\"Arcanum\""));
    assert!(stderr.contains("reference conflict"));
    assert!(stderr
        .contains("conflicts=[WizardZone=\"Infirmary\" conflicts with SharedMap=\"Arcanum\"]"));
    assert!(stderr.contains("unverified_fallback"));
    assert!(stdout.contains("mismatches: 1"));

    let zones_bytes = fs::read(&output).unwrap();
    let diagnostics_bytes = fs::read(&diagnostics_path).unwrap();
    fs::write(&diagnostics_path, b"deliberately corrupted diagnostics").unwrap();
    let second = Command::new(env!("CARGO_BIN_EXE_wizrust101-db"))
        .args(["compare", "--input"])
        .arg(&raw)
        .arg("--output")
        .arg(&output)
        .arg("--reference")
        .arg(&reference)
        .output()
        .unwrap();
    assert!(second.status.success());
    assert_eq!(fs::read(&output).unwrap(), zones_bytes);
    assert_eq!(fs::read(&diagnostics_path).unwrap(), diagnostics_bytes);
    let _ = fs::remove_dir_all(temp);
}

#[test]
fn generate_needs_no_oracle_and_never_writes_under_raw() {
    let temp = std::env::temp_dir().join(format!("wizrust-no-oracle-{}", std::process::id()));
    let raw = temp.join("raw");
    make_zone(&raw, "zone", "Fixture/UnknownZone", None);
    let locale = raw.join("misc/Root/Locale/en-US");
    fs::create_dir_all(&locale).unwrap();
    for name in [
        "Zone.lang",
        "Housing.lang",
        "WizardZone.lang",
        "WizardCompassLocs.lang",
        "WorldNames.lang",
    ] {
        fs::write(locale.join(name), utf16("1:fixture\n")).unwrap();
    }
    fs::write(raw.join("misc/Root/DoodleMapMap.xml"), []).unwrap();
    let before = snapshot(&raw);

    let forbidden_output = raw.join("generated/zones.json");
    let rejected = Command::new(env!("CARGO_BIN_EXE_wizrust101-db"))
        .args(["generate", "--input"])
        .arg(&raw)
        .arg("--output")
        .arg(&forbidden_output)
        .output()
        .unwrap();
    assert!(!rejected.status.success());
    assert!(!raw.join("generated").exists());

    let output = temp.join("out/nested/zones.json");
    let missing_oracle = temp.join("does-not-exist/reference.json");
    let generated = Command::new(env!("CARGO_BIN_EXE_wizrust101-db"))
        .args(["generate", "--input"])
        .arg(&raw)
        .arg("--output")
        .arg(&output)
        .arg("--reference")
        .arg(&missing_oracle)
        .output()
        .unwrap();
    assert!(
        generated.status.success(),
        "{}",
        String::from_utf8_lossy(&generated.stderr)
    );

    let json: serde_json::Value = serde_json::from_slice(&fs::read(&output).unwrap()).unwrap();
    assert_eq!(json.as_object().unwrap().len(), 1);
    assert_eq!(json["Fixture/UnknownZone"]["zone"], "UnknownZone");
    assert_eq!(json["Fixture/UnknownZone"]["world"], "Unknown");
    let diagnostics_path = output.with_file_name("zones.diagnostics.json");
    let diagnostics: serde_json::Value =
        serde_json::from_slice(&fs::read(diagnostics_path).unwrap()).unwrap();
    assert_eq!(diagnostics.as_array().unwrap().len(), 1);
    assert_eq!(diagnostics[0]["confidence"], "unknown");
    assert_eq!(
        diagnostics[0]["emitted_zone"]["source"],
        "canonical_leaf_fallback"
    );
    assert_eq!(
        diagnostics[0]["emitted_zone"]["confidence"],
        "unverified_fallback"
    );
    assert_eq!(snapshot(&raw), before);
    let _ = fs::remove_dir_all(temp);
}

fn snapshot(root: &Path) -> Vec<(std::path::PathBuf, Vec<u8>)> {
    fn collect(root: &Path, current: &Path, files: &mut Vec<std::path::PathBuf>) {
        for entry in fs::read_dir(current).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                collect(root, &path, files);
            } else {
                files.push(path.strip_prefix(root).unwrap().to_path_buf());
            }
        }
    }
    let mut paths = Vec::new();
    collect(root, root, &mut paths);
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            let bytes = fs::read(root.join(&path)).unwrap();
            (path, bytes)
        })
        .collect()
}
