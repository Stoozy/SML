extern crate clap;
use clap::{Arg, App};
use curl::easy::Easy;
use std::{io::stdout, str, io::Write};

fn main() {

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
            let mut easy = Easy::new();
            easy.follow_location(true).unwrap();

            let url = format!("https://api.cfwidget.com/{}", id);
            println!("Request URL: {}", url);
            easy.url(url.as_str()).unwrap();
            easy.write_function(|data|{
                stdout().write_all(data).unwrap();

                //let data_str = 	str::from_utf8(data).unwrap();
                //println!("{}", data_str);

                //let parsed = json::parse(data_str).unwrap();
                //    
                //parsed.dump();

                Ok(data.len())
            }).unwrap();

			easy.perform().unwrap();
        },
        None => {
            println!("No id was provided.");
        }
    }
    
}
