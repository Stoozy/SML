extern crate clap;
extern crate ftp;


pub mod downloader;
pub mod sml;
pub mod auth;
pub mod ima;
pub mod invoker;
pub mod cf;

use std::fs::{self, OpenOptions};
use std::path::PathBuf;
use clap::*;
use subprocess::Exec;

use std::io::{self, Write, Read};

use crate::downloader::Downloader;
use crate::ima::InstanceManager;
use crate::sml::User;
use crate::invoker::Invoker;

use ansi_term::Colour::*;

fn pause() {
    let mut stdout = io::stdout();
    stdout.write(b"Press Enter to continue...").unwrap();
    stdout.flush().unwrap();
    io::stdin().read(&mut [0]).unwrap();
}

fn get_instances_path() -> Option<PathBuf> {
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

fn main() -> () {

    // test area
    let mut ima = InstanceManager::new(get_instances_path().unwrap());

    let mut user_path = std::env::current_exe().unwrap();
    user_path.pop(); // get rid of executable
    user_path.push("userinfo.json");

    // create new app
    let app = App::new("SML")
        .version("1.0")
        .author("Stoozy")
        .about("A Minecraft Modded Launcher Command Line Interface")
        .arg(Arg::with_name("id")
             .short("i")
             .long("id")
             .help("Searches for project in curseforge with given id")
             .takes_value(true))
        .arg(Arg::with_name("authenticate")
             .short("a")
             .long("auth")
             .help("Log in through mojang")
             .takes_value(false))
        .get_matches();

    // authentication
    if app.is_present("authenticate"){
        let user = auth::handle_auth().expect("Failed authentication");

        println!("{}", Green.paint("Authentication successful!"));
        std::io::stdout().flush().unwrap();

        let user_data = serde_json::to_string(&user).expect("Couldn't parse username and token");
        fs::write(user_path.clone(), user_data.as_bytes()).expect("Couldn't save user info");

    }
    

    match app.value_of("id") {
        Some(id) => {

            let mut proj = cf::CFProject::new(
                id.parse::<u64>().expect("Not a valid id"),
                "https://api.cfwidget.com/".to_string());

            let choice = proj.get_choice();
            let instance = ima
                .create_instance(proj.files[choice].display.clone())
                .expect("Error creating instance");
           

            let mcv = proj.files[choice].version.clone();
            let fv = sml::get_fv_from_mcv(mcv.clone());
            let mcv_fv = format!("{}-{}", mcv, fv);


            let mut launcher_profiles_path = instance.get_path();
            launcher_profiles_path.push("launcher_profiles.json");
            fs::write(launcher_profiles_path, "{\"profiles\": {} }").expect("Error writing to launcher profiles") ;

            // https://files.minecraftforge.net/maven/net/minecraftforge/forge/1.16.5-36.1.0/forge-1.16.5-36.1.0-installer.jar

            let forge_url = format!("https://files.minecraftforge.net/maven/net/minecraftforge/forge/{}/forge-{}-installer.jar", mcv_fv, mcv_fv );

            let mut forge_path = instance.get_path().clone();
            forge_path.push(format!("forge-{}-installer.jar", mcv_fv ));

            let mut forge_dloader = Downloader::new();
            forge_dloader.set_url(forge_url);
            forge_dloader.set_path(forge_path.clone());
            forge_dloader.download().expect("Error downloading forge");

            println!();
            println!("When you are prompted by forge, {}", Yellow.paint("PASTE THE FOLLOWING DIRECTORY"));
            println!("{}", instance.get_path().display());
            println!();

            pause();

            // run the forge installer
            let cmd = format!("java -jar \"{}\"", forge_path.display());
            Exec::shell(cmd).join().unwrap();


            //sml::get_stage(proj.files[choice].clone(), instance.clone());
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
            forge_version_path.push(format!("versions/{}-forge-{}/{}-forge-{}.json", mcv, fv, mcv, fv));
 
            let mut vanilla_version_path = instance.get_path();
            vanilla_version_path.push(format!("versions/{}/{}.json", mcv, mcv));


            version_paths.push(vanilla_version_path.clone());



            println!("{}", Yellow.paint("Getting libraries..."));
            sml::get_libraries(libpath.clone(), version_paths.clone()).unwrap();

            version_paths.push(forge_version_path.clone());

            

            println!("{}", Yellow.paint("Getting mods..."));
            sml::get_mods(mods_path);



            println!("{}", Yellow.paint("Getting assets..."));
            sml::get_assets(instance.get_path().clone(), vanilla_version_path.clone()).unwrap();

            let access_token = match !user_path.exists(){
                true => {
                    let user =  auth::handle_auth().expect("Couldn't get access token");

                    let user_data = serde_json::to_string(&user)
                        .expect("Couldn't parse username and token");
                    fs::write(user_path, user_data.as_bytes())
                        .expect("Couldn't save user info");
                    user.token
                },
                false => { 
                    let user_file = OpenOptions::new()
                        .read(true)
                        .write(true)
                        .open(user_path.clone())
                        .unwrap();
                    let user: User = match serde_json::from_reader(user_file){
                        Ok(u) => u,
                        Err(_) => {
                            println!("Error occured getting user info");
                            let u = auth::handle_auth().expect("Failed authentication");
                            
                            println!("Authentication successful!");
                            std::io::stdout().flush().unwrap();
                        
                            let user_data = serde_json::to_string(&u)
                                .expect("Couldn't parse username and token");
                            fs::write(user_path.clone(), user_data.as_bytes())
                                .expect("Couldn't save user info");
                            u
                        },
                    };
                    user.token
                }

            };


            let forge_json_file = OpenOptions::new()
                                .read(true)
                                .write(true)
                                .open(forge_version_path.clone())
                                .expect("Couldn't open forge version file");

            let forge_json : serde_json::Value = serde_json::from_reader(forge_json_file).expect("Unable to parse forge json file");

            let mut vanilla_version_path = instance.get_path().clone();
            vanilla_version_path.push(format!("versions/{}/{}.json", mcv, mcv));

            let mut asset_index = proj.files[choice].version.clone();
            // remove the last . and number
            asset_index.remove(asset_index.len()-1);
            asset_index.remove(asset_index.len()-1);

            let main_class = forge_json["mainClass"]
                                    .as_str()
                                    .expect("Couldn't get main class");
            let forge_args = sml::get_forge_args(forge_json.clone());



            let classes = sml::get_cp_from_version(libpath.clone(), version_paths.clone());
            let mut classpaths : Vec<PathBuf> = Vec::new();

            for class in classes {
                classpaths.push(class.1);
            }
            
            // TODO: Properly get args
            let mut invoker = Invoker::new(
                "java ".to_string(),
                binpath,
                classpaths,
                format!("{} --assetsDir \"{}\" --assetIndex {} --gameDir \"{}\" --version  {} --accessToken {} --versionType release --userType mojang",  forge_args.unwrap(), assetspath.display(), asset_index, instance.get_path().display(), proj.files[choice].version, access_token),
                main_class.to_string()
                );


            invoker.gen_invocation();
            invoker.display_invocation();

        },
        None => {}
    };
}
