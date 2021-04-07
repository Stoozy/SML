use crate::util;
use ansi_term::Color::Yellow;
use serde_json::Value;

#[derive(Clone)]
pub struct CFFile {
    pub id: u64,
    pub display: String,
    pub name: String,
    pub ftype: String,
    pub version: String,
}

impl CFFile {
    pub fn get_download_url(&self) -> String {
        let mut filename: Vec<char> = self.name.chars().collect();

        for i in 0..filename.len() {
            if filename[i] == ' ' {
                filename[i] = '+';
            }
        }

        let file: String = filename.into_iter().collect();

        format!(
            "https://media.forgecdn.net/files/{}/{}/{}",
            self.id / 1000,
            self.id % 1000,
            file
        )
    }
}

#[derive(Clone)]
pub struct CFProject {
    id: u64,
    api_url: String,
    pub files: Vec<CFFile>,
}

impl CFProject {
    pub fn new(pid: u64, url: String) -> CFProject {
        CFProject {
            id: pid,
            api_url: url,
            files: Vec::new(),
        }
    }

    pub fn get_json(&mut self) -> serde_json::Value {
        // get proper endpoint
        let body: String = ureq::get(format!("{}{}", self.api_url, self.id).as_str())
            .call()
            .unwrap()
            .into_string()
            .unwrap();
        serde_json::from_str(body.as_str()).unwrap()
    }

    pub fn get_choice(&mut self) -> usize {
        let res: Value = self.get_json();

        let files: &Vec<Value> = res["files"]
            .as_array()
            .expect("Error getting files: Invalid json");

        for (i, fileobj) in files.iter().enumerate() {
            // fill vector
            let cfile = CFFile {
                id: fileobj["id"].as_u64().unwrap(),
                display: fileobj["display"].as_str().unwrap().to_string(),
                name: fileobj["name"].as_str().unwrap().to_string(),
                ftype: fileobj["type"].as_str().unwrap().to_string(),
                version: fileobj["version"].as_str().unwrap().to_string(),
            };

            self.files.push(cfile);

            // print options
            println!(
                "  [{}]: {} - {}@{}",
                Yellow.paint(format!("{}", i)),
                fileobj["display"].as_str().unwrap(),
                fileobj["type"].as_str().unwrap(),
                fileobj["version"].as_str().unwrap()
            );
        }

        println!("Choose version (Enter a number): ");

        let choice: u64 = match util::get_u64() {
            Some(n) => n,
            None => 0,
        };

        choice as usize
    }
}
