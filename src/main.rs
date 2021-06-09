/*
 
SML - A minecraft modded launcher CLI

Copyright (C) 2021 Stoozy

This program is free software; you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation; either version 2 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with this program; if not, write to the Free Software Foundation, Inc., 59 Temple Place, Suite 330, Boston, MA 02111-1307 USA
 
 */

#[macro_use] extern crate prettytable;
extern crate clap;
extern crate ftp;


pub mod auth;
pub mod cf;
pub mod downloader;
pub mod forge;
pub mod instance;
pub mod manager;
pub mod invoker;
pub mod setup;
pub mod util;

use clap::*;

use std::io::Write;


use crate::manager::InstanceManager;
use crate::instance::Instance;
use std::fs::{self};

use ansi_term::Colour::*;
use env_logger;

#[tokio::main]
async fn main() {
    // hard code info logging
    std::env::set_var("RUST_LOG", "info");

    env_logger::init();

    let instances_path = util::get_instances_path().unwrap();
    let mut ima = InstanceManager::new(instances_path.clone());

    let mut user_path = instances_path;
    user_path.pop(); // get rid of instances dir
    user_path.push("userinfo.json");

    // create new app
    let app = App::new("SML")
        .version("0.1.0")
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
                .long("remove")
                .value_name("ID")
                .help("Removes instance with the ID provided")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("print-config")
                .long("print-config")
                .value_name("ID")
                .help("Shows the custom flags for an instance")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("config")
                .short("c")
                .value_name("ID")
                .long("config")
                .help("Configures instance with the ID provided")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("rename")
                .long("rename")
                .value_name("ID")
                .help("Rename the instance with provided ID")
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

    if app.is_present("list") {
        ima.display_list();
        return;
    }

    // CONFIGURE
    match app.value_of("config") {
        Some(id) =>{
            let instance_paths = ima.get_list();

            for instance_path in instance_paths {
                let instance = Instance::from(instance_path);
                if &instance.uuid()[0..8] == id {
                    
                    println!("Enter custom java flags: ");

                    let mut new_custom_args = String::new();

                    std::io::stdin()
                        .read_line(&mut new_custom_args)
                        .expect("Unable to get user input");

                    let len = new_custom_args.trim_end_matches(&['\r', '\n'][..]).len();
                    new_custom_args.truncate(len);

                    instance.set_config(new_custom_args);
                }
            }

        },
        None => ()
    }

    //  SHOW CONFIG
    match app.value_of("print-config") {
        Some(id) => {
            let instance_paths = ima.get_list();

            for instance_path in instance_paths {
                let instance = Instance::from(instance_path);
                if &instance.uuid()[0..8] == id {
                    instance.display_config(); 
                }

            }
        },
        None => (),
    };

    // RENAME
    match app.value_of("rename") {
        Some(id) => {
            let instance_paths = ima.get_list();

            for instance_path in instance_paths {
                let instance = Instance::from(instance_path);
                if &instance.uuid()[0..8] == id {
                    let mut new_name : String = String::new();

                    println!("Enter the new name: ");
                    std::io::stdin()
                        .read_line(&mut new_name)
                        .expect("Unable to get user input");

                    instance.rename(new_name);
                }

            }
        },
        None => (),
    };
    
    // REMOVE 
    match app.value_of("remove") {
        Some(id) => {
            let instance_paths = ima.get_list();

            for instance_path in instance_paths {

                let instance = Instance::from(instance_path);

                if &instance.uuid()[0..8] == id {
                    instance.delete();
                    std::process::exit(0);
                }
            }

            println!("{} {}",Red.paint("Instance not found: "), id);
        },
        None => (),

    }

    // LAUNCH
    match app.value_of("launch") {
        Some(id) => {
            let instance_paths = ima.get_list();
            for instance_path in instance_paths {
                let instance = Instance::from(instance_path);
                if &instance.uuid()[0..8] == id {
                    instance.launch();
                }
            }

        }
        None => (),
    }

    // AUTHENTICATION
    if app.is_present("authenticate") {
        let user = auth::handle_auth(ima.clone()).await.expect("Failed authentication");

        println!("{}", Green.paint("Authentication successful!"));
        std::io::stdout().flush().unwrap();

        let user_data = serde_json::to_string(&user).expect("Couldn't parse username and token");
        fs::write(user_path.clone(), user_data.as_bytes()).expect("Couldn't save user info");
    }

    match app.value_of("install") {
        Some(id) => {
            // if there is no userinfo, stop the setup process
            if !user_path.exists() {
                println!("{}", Red.paint("Please authenticate first!"));
                return;
            }
            setup::forge_setup(ima, id.parse::<u64>().expect("Not a valid id"), user_path).await;
        }
        None => {}
    };
}
