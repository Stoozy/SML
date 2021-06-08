use std::{
    fs::{self, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

pub async fn assets_len(version_path: PathBuf) -> u64 {
    let version_file = File::open(version_path).unwrap();
    let version: serde_json::Value = serde_json::from_reader(version_file).unwrap();

    let url = match version["assetIndex"]["url"].as_str() {
        Some(val) => val,
        None => {
            return 0;
        }
    };


    let resp = reqwest::get(reqwest::Url::parse(url).unwrap())
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

    let assets_json : serde_json::Value = serde_json::from_str(resp.as_str()).unwrap();

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
    if cfg!(windows) {
        return std::env::current_exe().ok().and_then(|mut pb| {
            pb.pop();
            pb.push("instances");
            // if path dne then, create one
            // (this should be first time only)
            if !pb.exists() {
                println!("Creating instances dir at {}", pb.display());
                fs::create_dir_all(pb.clone()).expect("Error creating instances folder");
            }
            return Some(pb);
        });
    } else {
        // assuming linux here
        match std::env::var("HOME") {
            Ok(val) => {
                let mut pb = PathBuf::from(val);
                pb.push(".local/share/sml/instances");
                fs::create_dir_all(pb.clone()).expect("Unable to create sml dir");

                return Some(pb);
            }
            Err(_) => return None,
        };
    }
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

pub fn geq_version(version1: &str, version2: &str) -> bool {
    // serialize version as int and check if greater
    let v1_c: Vec<&str> = version1.split(".").collect();
    let v2_c: Vec<&str> = version2.split(".").collect();

    // 1.v1_c[1].x  > 1.v2_c[1].x

    let mut v1 = 0;
    let mut v2 = 0;
    
    for i in v1_c {
        v1 =  v1 * 10 +  match i.parse::<i32>(){
            Ok(val) => val,
            Err(_) => 0 
        };
    }

    for i in v2_c {
        v2 = v2 * 10 + match i.parse::<i32>(){
            Ok(val) => val,
            Err(_) =>  0
                //panic!(" v1:{}, v2:{} ",v1,v2)
        };
    }

    v1 >= v2
}

pub fn get_forge_args(json: serde_json::Value, is_pre_13 : bool) -> Option<String> {
    let mut retstr = String::new();
    // get args here
    if is_pre_13{
        // parse args here
    }else{
        let args = json["arguments"]["game"].as_array().unwrap();
        for arg in args.iter() {
            retstr.push(' ');
            retstr.push_str(arg.as_str().unwrap());
        }
    }
    

    Some(retstr)
}

pub fn copy_overrides(instance_path: PathBuf, overrides_path: PathBuf) {
    copy_dir_all(overrides_path.as_path(), instance_path.as_path())
        .expect("Could not copy overrides");
}

pub async fn get_fv_from_mcv(mcv: String) -> String {
    let versions_url =
        "https://files.minecraftforge.net/maven/net/minecraftforge/forge/promotions_slim.json";

    let resp = reqwest::get(reqwest::Url::parse(versions_url).unwrap())
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

    let versions_json: serde_json::Value = serde_json::from_str(resp.as_str()).unwrap();

    let key = format!("{}-recommended", mcv);
    versions_json["promos"][key]
        .as_str()
        .expect("Couldn't get forge versions list")
        .to_string()
}

pub fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

