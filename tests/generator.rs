use std::{fs, process::Command};

fn utf16(s: &str) -> Vec<u8> {
    let mut b = vec![0xff, 0xfe];
    for c in s.encode_utf16() {
        b.extend(c.to_le_bytes())
    }
    b
}
fn zone(root: &std::path::Path, folder: &str, path: &str) {
    let d = root.join(folder);
    fs::create_dir_all(&d).unwrap();
    fs::write(d.join("gamedata.bin"), format!("prefix\0{path}\0")).unwrap();
}
#[test]
fn resolves_direct_inherited_and_special_interiors_and_reports_unknowns() {
    let temp = std::env::temp_dir().join(format!("wizrust-evidence-{}", std::process::id()));
    let raw = temp.join("raw");
    let maps = raw.join("misc/GUI-WorldData/Maps");
    fs::create_dir_all(&maps).unwrap();
    let direct = "Aquila/AQ_Z00_Hub";
    let inherited = "Aquila/Interiors/AQ_Z01_Apollo_Room";
    let special = "Aquila/Interiors/AQ_Z00_I01_GorgonCave";
    let ambiguous = "Amb/Zone";
    let unknown = "NoMap/Zone";
    for (p, f) in [
        (direct, "a"),
        (inherited, "b"),
        (special, "c"),
        (ambiguous, "d"),
        (unknown, "e"),
    ] {
        zone(&raw, f, p)
    }
    let mut blob = Vec::new();
    for (map, p) in [
        ("hub.xml", direct),
        ("palace.xml", inherited),
        ("hub.xml", special),
        ("hub.xml", ambiguous),
        ("other.xml", ambiguous),
    ] {
        blob.extend_from_slice(format!("|GUI|WorldData|Maps/{map}\0{p}\0").as_bytes())
    }
    fs::create_dir_all(raw.join("misc/Root")).unwrap();
    fs::write(raw.join("misc/Root/DoodleMapMap.xml"), blob).unwrap();
    fs::write(&maps.join("hub.xml"), b"Zone_00000001").unwrap();
    fs::write(&maps.join("palace.xml"), b"Zone_00000002").unwrap();
    fs::write(&maps.join("other.xml"), b"Zone_00000003").unwrap();
    let lang="1:Zone\n00000001\n\nGarden of Hesperides\n00000002\n\nMount Olympus\n00000003\n\nOther Place\n";
    fs::create_dir_all(raw.join("misc/Root/Locale/en-US")).unwrap();
    fs::write(raw.join("misc/Root/Locale/en-US/Zone.lang"), utf16(lang)).unwrap();
    let output = temp.join("out/zones.json");
    let reference = temp.join("zones.json");
    fs::write(
        &reference,
        format!(
            r#"{{"{direct}":"Garden of Hesperides","{special}":"Gorgon Cave","Missing/Ref":"x"}}"#
        ),
    )
    .unwrap();
    let status = Command::new(env!("CARGO_BIN_EXE_wizrust101-db"))
        .args(["compare", "--input"])
        .arg(&raw)
        .arg("--output")
        .arg(&output)
        .arg("--reference")
        .arg(&reference)
        .status()
        .unwrap();
    assert!(status.success());
    let j: serde_json::Value = serde_json::from_slice(&fs::read(output).unwrap()).unwrap();
    assert_eq!(j[direct], "Garden of Hesperides");
    assert_eq!(j[inherited], "Mount Olympus");
    assert_eq!(j[special], "Garden of Hesperides");
    assert!(j.get(ambiguous).is_none());
    assert!(j.get(unknown).is_none());
    let _ = fs::remove_dir_all(temp);
}
