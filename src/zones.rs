use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Resolved(String),
    Unresolved(String),
    Ambiguous(Vec<String>),
}
#[derive(Debug, Clone)]
pub struct Zone {
    pub path: String,
    pub status: Status,
}

pub fn discover(root: &Path) -> Result<Vec<Zone>, Box<dyn std::error::Error>> {
    if !root.is_dir() {
        return Err(format!("input root is not a directory: {}", root.display()).into());
    }
    let mut dirs = Vec::new();
    collect(root, &mut dirs)?;
    dirs.sort();
    let mut paths = BTreeSet::new();
    for dir in dirs {
        let file = dir.join("gamedata.bin");
        if file.is_file() {
            if let Some(path) = embedded_path(&fs::read(file)?) {
                paths.insert(path);
            }
        }
    }
    let loc = parse_zone_lang(&fs::read(root.join("misc/Root/Locale/en-US/Zone.lang"))?)?;
    let refs = parse_map_associations(&fs::read(root.join("misc/Root/DoodleMapMap.xml"))?, &paths);
    let mut map_cache = BTreeMap::new();
    for map in refs.values().flatten() {
        if map_cache.contains_key(map) {
            continue;
        }
        let p = root.join("misc/GUI-WorldData/Maps").join(map);
        let title = if p.is_file() {
            first_zone_key(&fs::read(p)?)
        } else {
            None
        };
        map_cache.insert(map.clone(), title.and_then(|k| loc.get(&k).cloned()));
    }
    let zones = paths
        .into_iter()
        .map(|path| {
            let names: BTreeSet<String> = refs
                .get(&path)
                .into_iter()
                .flatten()
                .filter_map(|m| map_cache.get(m).and_then(Clone::clone))
                .collect();
            let status = match names.len() {
                1 => Status::Resolved(names.into_iter().next().unwrap()),
                0 => Status::Unresolved("no explicit map title resolved".into()),
                _ => Status::Ambiguous(names.into_iter().collect()),
            };
            Zone { path, status }
        })
        .collect();
    Ok(zones)
}

fn collect(dir: &Path, dirs: &mut Vec<PathBuf>) -> io::Result<()> {
    dirs.push(dir.to_path_buf());
    for e in fs::read_dir(dir)? {
        let e = e?;
        if e.file_type()?.is_dir() {
            collect(&e.path(), dirs)?;
        }
    }
    Ok(())
}
fn embedded_path(bytes: &[u8]) -> Option<String> {
    let s = String::from_utf8_lossy(bytes);
    s.split(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '/' | '_' | '-')))
        .find(|x| {
            x.contains('/')
                && x.split('/').all(|p| !p.is_empty())
                && x.split('/')
                    .next()
                    .is_some_and(|p| p.chars().next().is_some_and(char::is_uppercase))
        })
        .map(str::to_owned)
}
fn parse_zone_lang(bytes: &[u8]) -> Result<BTreeMap<String, String>, Box<dyn std::error::Error>> {
    if !bytes.len().is_multiple_of(2) {
        return Err("Zone.lang has odd byte length".into());
    }
    let (pairs, _) = bytes.as_chunks::<2>();
    let units: Vec<u16> = pairs.iter().map(|b| u16::from_le_bytes(*b)).collect();
    let text = String::from_utf16(units.strip_prefix(&[0xfeff]).unwrap_or(&units))?;
    let lines: Vec<&str> = text.lines().collect();
    let mut out = BTreeMap::new();
    for i in 0..lines.len().saturating_sub(1) {
        let id = lines[i].trim();
        if id.len() == 8 && id.bytes().all(|b| b.is_ascii_digit()) {
            if let Some(value) = lines[i + 1..].iter().map(|line| line.trim()).find(|line| {
                !line.is_empty() && !(line.len() == 8 && line.bytes().all(|b| b.is_ascii_digit()))
            }) {
                out.insert(id.to_owned(), value.to_owned());
            }
        }
    }
    Ok(out)
}
fn first_zone_key(bytes: &[u8]) -> Option<String> {
    let s = String::from_utf8_lossy(bytes);
    let b = s.as_bytes();
    b.windows(13).find_map(|w| {
        (w.starts_with(b"Zone_") && w[5..].iter().all(u8::is_ascii_digit))
            .then(|| String::from_utf8_lossy(&w[5..]).into_owned())
    })
}
fn parse_map_associations(
    blob: &[u8],
    paths: &BTreeSet<String>,
) -> BTreeMap<String, BTreeSet<String>> {
    let marker = b"|GUI|WorldData|Maps/";
    let mut maprefs = Vec::new();
    let mut pos = 0;
    while let Some(i) = find_from(blob, marker, pos) {
        let start = i + marker.len();
        if let Some(endrel) = blob[start..].windows(4).position(|w| w == b".xml") {
            let end = start + endrel + 4;
            let map = String::from_utf8_lossy(&blob[start..end]).into_owned();
            maprefs.push((end, map));
            pos = end;
        } else {
            pos = start;
        }
    }
    let mut out: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for path in paths {
        let needle = path.as_bytes();
        let mut scan = 0;
        while let Some(i) = find_from(blob, needle, scan) {
            let ix = maprefs.partition_point(|(end, _)| *end <= i);
            if ix > 0 {
                let (end, map) = &maprefs[ix - 1];
                if i - *end <= 64 {
                    out.entry(path.clone()).or_default().insert(map.clone());
                }
            }
            scan = i + needle.len();
        }
    }
    out
}
fn find_from(haystack: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    if needle.is_empty() || from > haystack.len() {
        return None;
    }
    haystack[from..]
        .windows(needle.len())
        .position(|w| w == needle)
        .map(|i| i + from)
}
pub fn write_json(zones: &[Zone], output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let parent = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;
    let data: BTreeMap<&str, &str> = zones
        .iter()
        .filter_map(|z| {
            if let Status::Resolved(n) = &z.status {
                Some((z.path.as_str(), n.as_str()))
            } else {
                None
            }
        })
        .collect();
    let mut bytes = serde_json::to_vec_pretty(&data)?;
    bytes.push(b'\n');
    fs::write(output, bytes)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_localization_and_first_map_title() {
        let mut b = vec![0xff, 0xfe];
        for c in "1:Zone\n00001123\n\nGarden of Hesperides\n".encode_utf16() {
            b.extend(c.to_le_bytes())
        }
        let d = parse_zone_lang(&b).unwrap();
        assert_eq!(d["00001123"], "Garden of Hesperides");
        assert_eq!(
            first_zone_key(b"<Map>Zone_00001123</Map>Zone_00001124"),
            Some("00001123".into())
        );
    }
    #[test]
    fn only_explicit_associations_are_used() {
        let p = "Aquila/Interiors/AQ_Z01_Apollo_Room".to_string();
        let set = [p.clone()].into_iter().collect();
        let b = b"|GUI|WorldData|Maps/AQ_Z01_Palace.xml\0Aquila/Interiors/AQ_Z01_Apollo_Room";
        let a = parse_map_associations(b, &set);
        assert!(a[&p].contains("AQ_Z01_Palace.xml"));
        let none = parse_map_associations(b"Aquila/Interiors/AQ_Z01_Apollo_Room", &set);
        assert!(!none.contains_key(&p));
    }
}
