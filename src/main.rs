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
use std::fs::{self};

use std::io::Write;

use crate::ima::InstanceManager;
use crate::invoker::Invoker;

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
            Arg::with_name("remove")
                .short("r")
                .value_name("ID")
                .help("Removes instance with the ID provided")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("install")
                .short("i")
                .long("install")
                .value_name("ID")
                .help("Searches for project in curseforge with given ID and installs it")
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

    match app.value_of("remove") {
        Some(id) => {
            let id_num = id.parse::<u64>().expect("Not a valid id");
            let instances = ima.get_list();
            let invoker_path = &instances[id_num as usize];

            let mut cwd = invoker_path.clone();
            cwd.pop();

            fs::remove_dir_all(cwd).expect("Unable to remove instance");
        }
        None => (),
    }

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

    match app.value_of("install") {
        Some(id) => {
            // if there is no userinfo, stop the setup process
            if !user_path.exists() {
                println!("Please authenticate first!");
                return;
            }
            sml::forge_setup(ima, id.parse::<u64>().expect("Not a valid id"), user_path);
        }
        None => {}
    };
}
