use std::path::PathBuf;
use std::{
    fs::{self, File},
    io::{self, Read, Write},
};

pub fn assets_len(version_path: PathBuf) -> u64 {
    let version_file = File::open(version_path).unwrap();
    let version: serde_json::Value = serde_json::from_reader(version_file).unwrap();

    let url = match version["assetIndex"]["url"].as_str() {
        Some(val) => val,
        None => {
            return 0;
        }
    };

    let assets_json: serde_json::Value = ureq::get(url).call().unwrap().into_json().unwrap();

    let asset_objects = assets_json["objects"].as_object().unwrap();

    asset_objects.len() as u64
}

pub fn mods_len(mods_path: PathBuf) -> u64 {
    let mut mods_manifest_path = mods_path.clone();
    mods_manifest_path.push("manifest.json");

    let manifest_reader = File::open(mods_manifest_path).unwrap();
    let manifest: serde_json::Value =
        serde_json::from_reader(manifest_reader).expect("Couldn't get mod manifest");

    let mods = manifest["files"].as_array().unwrap();
    mods.len() as u64
}

pub fn pause() {
    let mut stdout = io::stdout();
    stdout.write(b"Press Enter to continue...").unwrap();
    stdout.flush().unwrap();
    io::stdin().read(&mut [0]).unwrap();
}

pub fn get_instances_path() -> Option<PathBuf> {
    std::env::current_exe().ok().and_then(|mut pb| {
        pb.pop();
        pb.push("instances/");
        // if path dne then, create one
        // (this should be first time only)
        if !pb.exists() {
            println!("Creating instances dir at {}", pb.display());
            fs::create_dir(pb.as_path().clone()).expect("Error creating instances folder");
        }
        Some(pb)
    })
}
pub fn get_u64() -> Option<u64> {
    let mut input_text = String::new();
    io::stdin()
        .read_line(&mut input_text)
        .expect("Failed to get input");

    Some(
        input_text
            .trim()
            .parse::<u64>()
            .expect("Error parsing number"),
    )
}

pub fn is_greater_version(version1: &str, version2: &str) -> bool {
    // serialize version as int and check if greater
    let v1_c: Vec<&str> = version1.split(".").collect();
    let v2_c: Vec<&str> = version2.split(".").collect();

    let mut v1 = 0;
    let mut v2 = 0;

    for i in v1_c {
        v1 = v1 * 10 + i.parse::<i32>().unwrap();
    }
    for i in v2_c {
        v2 = v2 * 10 + i.parse::<i32>().unwrap();
    }

    v1 > v2
}

pub fn get_forge_args(json: serde_json::Value) -> Option<String> {
    let mut retstr = String::new();
    // get args here
    let args = json["arguments"]["game"].as_array().unwrap();
    for arg in args.iter() {
        retstr.push(' ');
        retstr.push_str(arg.as_str().unwrap());
    }

    Some(retstr)
}
