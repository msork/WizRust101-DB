use std::{fs, process::Command};

fn utf16(text: &str) -> Vec<u8> {
    let mut bytes = vec![0xff, 0xfe];
    for unit in text.encode_utf16() {
        bytes.extend(unit.to_le_bytes());
    }
    bytes
}

fn make_zone(root: &std::path::Path, folder: &str, path: &str, wizard_key: Option<&str>) {
    let directory = root.join(folder);
    fs::create_dir_all(&directory).unwrap();
    let key = wizard_key
        .map(|key| format!("WizardZone_{key}\0"))
        .unwrap_or_default();
    fs::write(
        directory.join("gamedata.bin"),
        format!("zone metadata\0{path}\0{key}"),
    )
    .unwrap();
}

fn map_record(map: &str, path: &str) -> Vec<u8> {
    format!("|GUI|WorldData|Maps/{map}\0{path}\0").into_bytes()
}

#[test]
fn resolves_poi_and_wizard_candidates_reports_conflicts_and_omits_unknowns() {
    let temp = std::env::temp_dir().join(format!("wizrust-evidence-{}", std::process::id()));
    let raw = temp.join("raw");
    let maps = raw.join("misc/GUI-WorldData/Maps");
    fs::create_dir_all(&maps).unwrap();

    let parent = "Aquila/AQ_Z01_MountOlympus";
    let pit = "Aquila/Interiors/AQ_Z01_PitOfNoxii";
    let hall = "Aquila/Interiors/AQ_SkelKey_Hall_01";
    let conflicted = "Arcanum/Interiors/AR_Z01_Infirmary";
    let unresolved = "NoWorld/NoMap";
    make_zone(&raw, "parent", parent, Some("00001029"));
    make_zone(&raw, "pit", pit, Some("00001029"));
    make_zone(&raw, "hall", hall, Some("00001603"));
    make_zone(&raw, "infirmary", conflicted, Some("00001387"));
    make_zone(&raw, "unknown", unresolved, None);

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
        utf16("1:Zone\n00000001\n\nMount Olympus\n00000003\n\nArcanum\n"),
    )
    .unwrap();
    fs::write(
        locale.join("WizardZone.lang"),
        utf16("1:WizardZone\n00001029\n\nMount Olympus\n00001603\n\nStonegaze’s Antichamber\n00001387\n\nInfirmary\n"),
    )
    .unwrap();
    fs::write(
        locale.join("WizardCompassLocs.lang"),
        utf16("1:WizardCompassLocs\n00000135\n\nPit of the Noxii\n"),
    )
    .unwrap();
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
    assert_eq!(json[pit], "Pit of the Noxii");
    assert_eq!(json[hall], "Stonegaze’s Antichamber");
    assert_eq!(json[conflicted], "Arcanum");
    assert!(json.get(unresolved).is_none());
    let stderr = String::from_utf8(result.stderr).unwrap();
    let stdout = String::from_utf8(result.stdout).unwrap();
    assert!(stderr.contains("CompassPoi=\"Pit of the Noxii\""));
    assert!(stderr.contains("WizardZone=\"Stonegaze’s Antichamber\""));
    assert!(stderr.contains("SharedMap=\"Arcanum\""));
    assert!(stderr.contains("reference conflict"));
    assert!(stderr
        .contains("conflicts=[WizardZone=\"Infirmary\" conflicts with SharedMap=\"Arcanum\"]"));
    assert!(stdout.contains("mismatches: 2"));
    let _ = fs::remove_dir_all(temp);
}
