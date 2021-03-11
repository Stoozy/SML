extern crate clap;
use clap::{Arg, App};
use curl::easy::Easy;
use serde_json::Value;
//use std::{io::stdout, str, io::Write};

use std::str;

fn main() -> Result<(), ureq::Error> {

    // create new app
    let app = App::new("SML")
                    .version("1.0")
                    .author("Stoozy")
                    .arg(Arg::with_name("project_id")
                        .short("i")
                        .long("project_id")
                        .takes_value(true))
                    .get_matches();


    match app.value_of("project_id"){
        Some(id) => {
            let url = format!("https://api.cfwidget.com/{}", id);
            println!("Request url: {}", url);

            let body : String = ureq::get(url.as_str())
                .call().unwrap()
                .into_string().unwrap();
            let v: Value = serde_json::from_str(body.as_str()).unwrap();

            println!("Title: {}", v["title"]);
        },
        None => {
            println!("No id was provided.");
        }
    }

    Ok(())
    
}
