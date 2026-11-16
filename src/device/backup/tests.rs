use super::*;

#[test]
fn hostile_paths_types_duplicates_versions_and_size_headers_are_rejected() {
    let directory = std::env::temp_dir().join(format!("mealshop-archive-{}", random_id().unwrap()));
    fs::create_dir(&directory).unwrap();
    let manifest = Manifest {
        schema_version: 1,
        kind: "mealshop-app-only".to_owned(),
        engine_version: "0.1.0".to_owned(),
        created_at: 0,
        households: 0,
        files: 1,
        bytes: 1,
    };
    for (name, kind, size) in [
        ("../escaped", tar::EntryType::Regular, 1),
        ("/absolute", tar::EntryType::Regular, 1),
        ("data/../../escaped", tar::EntryType::Regular, 1),
        ("data/.membership.pending", tar::EntryType::Regular, 1),
        ("data/link", tar::EntryType::Symlink, 0),
        ("data/hard", tar::EntryType::Link, 0),
        ("data/sparse", tar::EntryType::GNUSparse, 0),
        ("data/large", tar::EntryType::Regular, MAX_FILE + 1),
    ] {
        let mut tar = tar::Builder::new(Vec::new());
        let bytes = serde_json::to_vec(&manifest).unwrap();
        append(
            &mut tar,
            Path::new("manifest.json"),
            bytes.len() as u64,
            &bytes[..],
        )
        .unwrap();
        let mut header = tar::Header::new_ustar();
        header.as_mut_bytes()[..name.len()].copy_from_slice(name.as_bytes());
        header.set_entry_type(kind);
        header.set_size(size);
        header.set_cksum();
        tar.append(&header, &b"x"[..]).unwrap();
        assert!(
            extract(&tar.into_inner().unwrap()[..], &directory).is_err(),
            "{name}"
        );
    }
    let mut tar = tar::Builder::new(Vec::new());
    let bytes = serde_json::to_vec(&manifest).unwrap();
    for _ in 0..2 {
        append(
            &mut tar,
            Path::new("manifest.json"),
            bytes.len() as u64,
            &bytes[..],
        )
        .unwrap();
    }
    assert!(extract(&tar.into_inner().unwrap()[..], &directory).is_err());
    let bytes = serde_json::to_vec(&Manifest {
        schema_version: 99,
        ..manifest
    })
    .unwrap();
    let mut tar = tar::Builder::new(Vec::new());
    append(
        &mut tar,
        Path::new("manifest.json"),
        bytes.len() as u64,
        &bytes[..],
    )
    .unwrap();
    assert!(matches!(
        extract(&tar.into_inner().unwrap()[..], &directory),
        Err("device_restore_version")
    ));
    fs::remove_dir_all(directory).unwrap();
}
