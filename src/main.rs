/*
 
SML - A minecraft modded launcher CLI

Copyright (C) 2021 Stoozy

This program is free software; you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation; either version 2 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with this program; if not, write to the Free Software Foundation, Inc., 59 Temple Place, Suite 330, Boston, MA 02111-1307 USA
 
 */

extern crate clap;
extern crate ftp;
//extern crate env_logger;


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
use crate::invoker::Invoker;
use std::fs::{self, OpenOptions};

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
                .value_name("ID")
                .help("Removes instance with the ID provided")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .help("configures instance")
                .takes_value(false),
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

    if app.is_present("config") {
        ima.display_list();
        println!("Please enter which instance you would like to configure: ");

        let instances = ima.get_list();
        let id = util::get_u64().expect("Invalid number");
        let invoker_file_path = &instances[id as usize];

        dbg!(invoker_file_path.clone());
        let invoker_file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(invoker_file_path.clone())
            .expect("Unable to open sml invoker file");

        let mut invoker_json: serde_json::Value =
            serde_json::from_reader(invoker_file).expect("Invalid sml invoker json file");

        let mut new_custom_args = String::new();

        println!("Enter custom java flags: ");

        std::io::stdin()
            .read_line(&mut new_custom_args)
            .expect("Unable to get user input");

        // no need for newline characters
        let len = new_custom_args.trim_end_matches(&['\r', '\n'][..]).len();
        new_custom_args.truncate(len);

        invoker_json["custom_args"] = serde_json::Value::String(new_custom_args);

        std::fs::write(invoker_file_path, invoker_json.to_string())
            .expect("Unable to write to sml invoker file");

        println!("{}", Green.paint("Configuration complete!"))
    }

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

            let mut cwd = invoker_path.clone();
            cwd.pop(); // gets rid of the sml_invoker.json part of the  pathbuf

            invoker.invoke();
        }
        None => (),
    }

    // authentication
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
