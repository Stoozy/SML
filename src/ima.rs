use ansi_term::Color::*;
use std::fs;
use std::path::PathBuf;

#[derive(Clone)]
pub struct Instance {
    name: String,
    path: PathBuf,
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

pub struct InstanceManager {
    path: PathBuf,
}

impl InstanceManager {
    pub fn new(pb: PathBuf) -> InstanceManager {
        InstanceManager { path: pb }
    }

    pub fn get_path(&self) -> PathBuf {
        self.path.clone()
    }

    pub fn create_instance(&mut self, name: String) -> Option<Instance> {
        let mut ipath = self.path.clone();
        ipath.push(name.clone());

        // TODO: if it exists ask user if they
        //      want to delete the old one and
        //      create a new one

        // create folder if it doesn't exist
        if !ipath.exists() {
            println!("Creating instance dir at {}", ipath.display());
            fs::create_dir(&(*ipath.as_path())).expect("Error creating instances folder");
        } else {
            println!("Directory already exists! Overwriting ...");
        }

        Some(Instance::new(name, ipath))
    }

    pub fn get_list(&mut self) -> Vec<PathBuf> {
        let mut invoker_files: Vec<PathBuf> = Vec::new();

        let instances_dir = self.path.clone();
        for entry in fs::read_dir(instances_dir).unwrap() {
            let entry_path = entry.unwrap().path();
            let mut invoker_file = entry_path.clone();
            invoker_file.push("sml_invoker.json");

            if invoker_file.exists() {
                invoker_files.push(invoker_file);
            }
        }

        invoker_files
    }

    pub fn display_list(&mut self) {
        let instances_dir = self.path.clone();
        let mut counter: u64 = 0;

        for res in fs::read_dir(instances_dir).unwrap() {
            if let Ok(entry) = res {
                let ep = entry.path();

                let instance_name = entry.file_name();
                let mut invoker_file = ep.clone();
                invoker_file.push("sml_invoker.json");

                if invoker_file.exists() {
                    println!(
                        "[{}] {}",
                        Yellow.paint(format!("{}", counter)),
                        instance_name.to_str().unwrap()
                    );
                    counter += 1;
                }
            }
        }
    }
}
