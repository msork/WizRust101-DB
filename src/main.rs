mod zones;
use std::{collections::BTreeMap, env, fs, path::PathBuf, process};
use zones::Status;
fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        process::exit(1)
    }
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "generate".into());
    let mut input = PathBuf::from("raw");
    let mut output = PathBuf::from("out/zones.json");
    let mut reference = PathBuf::from("/home/maxim/Downloads/zones.json");
    while let Some(a) = args.next() {
        match a.as_str() {
            "--input" => input = args.next().ok_or("--input needs a path")?.into(),
            "--output" => output = args.next().ok_or("--output needs a path")?.into(),
            "--reference" => reference = args.next().ok_or("--reference needs a path")?.into(),
            "-h" | "--help" => {
                println!("Usage: wizrust101-db [generate|compare] [--input DIR] [--output FILE] [--reference FILE]");
                return Ok(());
            }
            _ => return Err(format!("unknown argument: {a}").into()),
        }
    }
    if command != "generate" && command != "compare" {
        return Err(format!("unknown command: {command}").into());
    }
    let found = zones::discover(&input)?;
    let resolved: Vec<_> = found
        .iter()
        .filter(|z| matches!(z.status, Status::Resolved(_)))
        .cloned()
        .collect();
    for z in &found {
        match &z.status {
            Status::Unresolved(why) => eprintln!("unresolved {}: {why}", z.path),
            Status::Ambiguous(names) => eprintln!("ambiguous {}: {}", z.path, names.join(" | ")),
            _ => {}
        }
    }
    zones::write_json(&resolved, &output)?;
    eprintln!(
        "wrote {} resolved zones to {} ({} unresolved/ambiguous)",
        resolved.len(),
        output.display(),
        found.len() - resolved.len()
    );
    if command == "compare" {
        let reference_json: serde_json::Value = serde_json::from_slice(&fs::read(reference)?)?;
        let expected: BTreeMap<String, String> = reference_json
            .as_object()
            .ok_or("reference must be a JSON object")?
            .iter()
            .filter_map(|(k, v)| Some((k.clone(), v.as_str()?.to_owned())))
            .filter(|(k, _)| k.contains('/'))
            .collect();
        let generated: BTreeMap<String, String> = serde_json::from_slice(&fs::read(&output)?)?;
        let (mut exact, mut mismatch, mut missing, mut new) = (0, 0, 0, 0);
        for (k, v) in &expected {
            match generated.get(k) {
                Some(a) if a == v => exact += 1,
                Some(a) => {
                    mismatch += 1;
                    eprintln!("conflict {k}: reference={v:?}, extracted={a:?}")
                }
                None => {
                    missing += 1;
                    if let Some(z) = found.iter().find(|z| z.path == *k) {
                        eprintln!(
                            "{} {}: {:?}",
                            if matches!(z.status, Status::Ambiguous(_)) {
                                "ambiguous"
                            } else {
                                "unresolved"
                            },
                            k,
                            z.status
                        )
                    } else {
                        eprintln!("missing {k}: no raw zone discovered")
                    }
                }
            }
        }
        for k in generated.keys() {
            if !expected.contains_key(k) {
                new += 1;
                eprintln!("new zone {k}: {}", generated[k]);
            }
        }
        println!("reference zone keys: {}; resolved/reference: {}/{}; exact matches: {exact}; mismatches: {mismatch}; unresolved/missing: {missing}; new zones: {new}",expected.len(),expected.len()-missing,expected.len());
    }
    Ok(())
}
