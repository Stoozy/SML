extern crate clap;
extern crate ftp;

pub mod auth;
pub mod downloader;
pub mod sml;

pub mod cf;
pub mod ima;
pub mod invoker;
pub mod util;

use clap::*;
use std::fs::{self, OpenOptions};
use std::path::PathBuf;
use subprocess::Exec;
use std::process::Command;

use std::io::Write;

use crate::downloader::Downloader;
use crate::ima::InstanceManager;
use crate::invoker::Invoker;
use crate::sml::User;

use ansi_term::Colour::*;

fn main() -> () {
    let mut ima = InstanceManager::new(util::get_instances_path().unwrap());

    let mut user_path = std::env::current_exe().unwrap();
    user_path.pop(); // get rid of executable
    user_path.push("userinfo.json");

    // create new app
    let app = App::new("SML")
        .version("1.0")
        .author("Stoozy <mahinsemail@gmail.com>")
        .about("A Minecraft Modded Launcher Command Line Interface")
        .arg(
            Arg::with_name("list")
                .long("list")
                .help("Lists all SML instances")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("launch")
                .long("launch")
                .help("Launches instance with specific ID")
                .value_name("ID")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("id")
                .short("i")
                .long("id")
                .value_name("ID")
                .help("Searches for project in curseforge with given ID")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("authenticate")
                .short("a")
                .long("auth")
                .help("Log in through mojang")
                .takes_value(false),
        )
        .get_matches();

    match app.value_of("launch") {
        Some(id) => {
            let id_num = id.parse::<u64>().expect("Not a valid id");
            let list_vec = ima.get_list();

            let invoker_path = &list_vec[id_num as usize];
            let mut invoker = Invoker::from(invoker_path);
            invoker.gen_invocation();

            let mut cwd = invoker_path.clone();
            cwd.pop(); // gets rid of the sml_invoker.json part of the  pathbuf

            invoker.invoke();
            //let cmd = invoker.get_cmd();
            //let process = Exec::shell(cmd).cwd(cwd);

            //let process = Command::new(cmd).output().unwrap();
            //println!("{}", process);
            //process.join().unwrap();
        }
        None => (),
    }

    if app.is_present("list") {
        ima.display_list();
        return ();
    }

    // authentication
    if app.is_present("authenticate") {
        let user = auth::handle_auth().expect("Failed authentication");

        println!("{}", Green.paint("Authentication successful!"));
        std::io::stdout().flush().unwrap();

        let user_data = serde_json::to_string(&user).expect("Couldn't parse username and token");
        fs::write(user_path.clone(), user_data.as_bytes()).expect("Couldn't save user info");
    }

    match app.value_of("id") {
        Some(id) => {
            // if there is no userinfo, stop the setup process
            if !user_path.exists() {
                println!("Please authenticate first!");
                return;
            }

            let mut proj = cf::CFProject::new(
                id.parse::<u64>().expect("Not a valid id"),
                "https://api.cfwidget.com/".to_string(),
            );

            let choice = proj.get_choice();
            let name = proj
                .files
                .iter()
                .nth(choice)
                .unwrap()
                .clone()
                .name()
                .clone()
                .chars()
                .map(|c| match c {
                    ' ' => '-',
                    _ => c,
                })
                .collect();

            let instance = ima.create_instance(name).expect("Error creating instance");

            let mcv = proj.files[choice].version.clone();
            let fv = sml::get_fv_from_mcv(mcv.clone());
            let mcv_fv = format!("{}-{}", mcv, fv);

            let mut launcher_profiles_path = instance.get_path();
            launcher_profiles_path.push("launcher_profiles.json");
            fs::write(launcher_profiles_path, "{\"profiles\": {} }")
                .expect("Error writing to launcher profiles");

            // example url
            // https://files.minecraftforge.net/maven/net/minecraftforge/forge/1.16.5-36.1.0/forge-1.16.5-36.1.0-installer.jar

            let forge_url = format!("https://files.minecraftforge.net/maven/net/minecraftforge/forge/{}/forge-{}-installer.jar", mcv_fv, mcv_fv );
            let mut forge_path = instance.get_path().clone();
            forge_path.push(format!("forge-{}-installer.jar", mcv_fv));

            let mut forge_dloader = Downloader::new();
            forge_dloader.set_url(forge_url);
            forge_dloader.set_path(forge_path.clone());
            forge_dloader
                .download(false)
                .expect("Error downloading forge");

            println!();
            println!(
                "When you are prompted by forge, {}",
                Yellow.paint("PASTE THE FOLLOWING DIRECTORY")
            );
            println!("{}", instance.get_path().display());
            println!();

            util::pause();

            // run the forge installer
            let cmd = format!("java -jar \"{}\"", forge_path.display());
            //Exec::shell(cmd).cwd(instance.get_path()).join().unwrap();
            Command::new("java").arg("-jar").arg(forge_path).output().unwrap();

            sml::get_modslist(proj.files[choice].clone(), instance.clone());

            let mut mods_path = instance.get_path().clone();
            mods_path.push("mods/");

            let mut libpath = instance.get_path().clone();
            libpath.push("libraries/");

            let mut binpath = instance.get_path().clone();
            binpath.push("bin/");

            let mut assetspath = instance.get_path().clone();
            assetspath.push("assets/");

            let mut version_paths = Vec::new();
            let mut forge_version_path = instance.get_path();
            forge_version_path.push(format!(
                "versions/{}-forge-{}/{}-forge-{}.json",
                mcv, fv, mcv, fv
            ));

            let mut vanilla_version_path = instance.get_path();
            vanilla_version_path.push(format!("versions/{}/{}.json", mcv.clone(), mcv.clone()));

            version_paths.push(vanilla_version_path.clone());

            let mut vp = version_paths.clone();

            println!("{}", Yellow.paint("Getting libraries..."));

            sml::get_binaries(vanilla_version_path.clone(), instance.get_path().clone());
            sml::get_libraries(libpath.clone(), version_paths.clone()).unwrap();

            vp.push(forge_version_path.clone());

            println!("{}", Yellow.paint("Getting mods..."));

            let mp = mods_path.clone();

            sml::get_mods(mcv.clone(), mp).unwrap();
            println!("{}", Yellow.paint("Getting assets..."));

            sml::get_assets(instance.get_path(), vanilla_version_path.clone()).unwrap();

            let mut overrides_path = mods_path.clone();
            overrides_path.push("overrides/");
            sml::copy_overrides(instance.get_path(), overrides_path);

            //let user = auth::handle_auth().expect("Couldn't get access token");
            //let user_data =
            //    serde_json::to_string(&user).expect("Couldn't parse username and token");
            //fs::write(user_path, user_data.as_bytes()).expect("Couldn't save user info");

            let userfile = OpenOptions::new()
                .read(true)
                .write(true)
                .open(user_path)
                .expect("Problem opening user info file");

            let userinfo = serde_json::from_reader(userfile).unwrap();

            let user: User = serde_json::from_value(userinfo).unwrap();

            let user_name = user.name;
            let access_token = user.token;

            let forge_json_file = OpenOptions::new()
                .read(true)
                .write(true)
                .open(forge_version_path.clone())
                .expect("Couldn't open forge version file");

            let forge_json: serde_json::Value =
                serde_json::from_reader(forge_json_file).expect("Unable to parse forge json file");

            let mut vanilla_version_path = instance.get_path().clone();
            vanilla_version_path.push(format!("versions/{}/{}.json", mcv.clone(), mcv.clone()));

            let mut asset_index = proj.files[choice].version.clone();
            // remove the last . and number
            asset_index.remove(asset_index.len() - 1);
            asset_index.remove(asset_index.len() - 1);

            let main_class = forge_json["mainClass"]
                .as_str()
                .expect("Couldn't get main class");
            let forge_args = util::get_forge_args(forge_json.clone());

            let classes = sml::get_cp_from_version(libpath.clone(), vp.clone());
            let mut classpaths: Vec<PathBuf> = Vec::new();

            for class in classes {
                classpaths.push(class.1);
            }

            let mut invoker = Invoker::new(
                "java ".to_string(),
                binpath,
                classpaths,
                format!("{} --assetsDir {} --assetIndex {} --gameDir {} --version  {} --username {} --accessToken {} --versionType release --userType mojang",  forge_args.unwrap(), assetspath.display(), asset_index, instance.get_path().display(), proj.files[choice].version, user_name, access_token),
                main_class.to_string()
                );

            let mut cmd_fp = instance.get_path();
            cmd_fp.push("sml_invoker.json");

            invoker.gen_invocation();
            invoker.export_as_json(cmd_fp);
            println!("{}", Green.paint("Setup is complete!"));
        }
        None => {}
    };
}
