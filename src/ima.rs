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

    pub fn get_path(&self) -> PathBuf {
        self.path.clone()
    }
}

pub struct InstanceManager {
    path: PathBuf,
    instances: Vec<Instance>,
}

impl InstanceManager {
    pub fn new(pb: PathBuf) -> InstanceManager {
        InstanceManager {
            path: pb,
            instances: Vec::new(),
        }
    }

    pub fn get_path(&self) -> PathBuf {
        return self.path.clone();
    }

    pub fn create_instance(&mut self, name: String) -> Option<Instance> {
        let mut ipath = self.path.clone();
        ipath.push(name.clone());

        // create folder if it doesn't exist
        // TODO: if it exists ask user if they
        //      want to delete the old one and
        //      create a new one

        if !ipath.exists() {
            println!("Creating instance dir at {}", ipath.display());
            fs::create_dir(ipath.as_path().clone()).expect("Error creating instances folder");
        } else {
            println!("Directory already exists! Overwriting ...");
        }

        Some(Instance::new(name, ipath))
    }

    pub fn add_instance(&mut self, i: Instance) {
        self.instances.push(i);
    }
}
