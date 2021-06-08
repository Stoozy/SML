use std::fs;
use std::fs::OpenOptions;
use std::path::PathBuf;
use uuid::Uuid;

use crate::invoker::Invoker;

#[derive(Clone)]
pub enum InstanceType {
    Forge,
    Vanilla,
    Fabric 
}

#[derive(Clone)]
pub struct Instance {
    name: String,
    path: PathBuf,
    invoker : Option<Invoker>,
    uuid : Option<Uuid>, 
}


impl Instance {

    pub fn new(n: String, p: PathBuf) -> Instance {
        Instance { name: n, path: p, invoker: None, uuid: None}
    }

    pub fn delete(&self)  {
        if self.path.exists() && self.path.is_dir(){
            fs::remove_dir_all(self.path.clone()).unwrap();

        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn rename(& mut self, new_name: String) {
        self.name = new_name;
    }

    pub fn uuid(&self) -> String {
        self.uuid.unwrap().to_string()
    }

    pub fn launch(&self) {
        match self.invoker.clone() {
            Some(mut invoker) => {
                invoker.invoke();
            },
            None => (),
        }
        //&self.invoker.unwrap().invoke();
    }

    pub fn get_path(&self) -> PathBuf {
        self.path.clone()
    }
}

impl From<PathBuf> for Instance {
   fn from(invoker_path: PathBuf) -> Self {


       let file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(invoker_path.clone())
                    .expect("Instance path invalid");

       let instance_json : serde_json::Value = serde_json::from_reader(file).unwrap();
       let instance_name = instance_json["instance_name"].as_str().expect("Invalid instance name");
       let instance_uuid = Uuid::parse_str(instance_json["instance_uuid"].as_str().unwrap()).expect("Invalid instance uuid");

       let mut instance_path = invoker_path.clone();
       instance_path.pop();

       Instance { 
           name: instance_name.to_string(), 
           path: instance_path, 
           invoker: Some(Invoker::from(&invoker_path)), 
           uuid: Some(instance_uuid)
       }

   }
}


