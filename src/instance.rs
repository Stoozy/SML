use ansi_term::Color::*;
use std::fs;
use std::path::PathBuf;

#[derive(Clone)]
pub struct Instance {
    name: String,
    path: PathBuf,
}


impl Instance {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }
}


impl Instance {
    pub fn new(n: String, p: PathBuf) -> Instance {
        Instance { name: n, path: p }
    }

    pub fn parse_config_file(&self) -> String {
        let retstr: String = "".to_string();

        retstr
    }

    pub fn create_config_file(&self) {
        let mut conf_fpath = self.path.clone();
        conf_fpath.push("sml_config.json");
        fs::File::create(conf_fpath).expect("Wasn't able to create SML config file");
    }

    pub fn get_path(&self) -> PathBuf {
        self.path.clone()
    }
}


