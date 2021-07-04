use serde_json::json;
use subprocess::Redirection;
use uuid::Uuid;
use std::io::Write;
use std::path::PathBuf;
use subprocess::Exec;

use crate::instance::InstanceType;

#[derive(Clone)]
pub struct Invoker {
    java: String,
    custom_args: Option<String>,
    binpath: PathBuf,
    classpaths: Vec<PathBuf>,
    args: String,
    main: String,
    ccmd: Option<String>,
    instance_type: InstanceType,
    instance_name: String,
    user_name: String,
    auth_token: String,
    uuid: String
}

impl Invoker {
    pub fn new(java: String, binpath: PathBuf, classpaths: Vec<PathBuf>, args: String, main: String, instance_name: String, instance_type: InstanceType, user_name : String, auth_token : String, uuid: String) -> Invoker {
        Invoker {
            java,
            custom_args: None,
            binpath,
            classpaths,
            args,
            main,
            ccmd: None,
            instance_type,
            instance_name,
            user_name,
            auth_token,
            uuid
        }
    }

    pub fn set_name(&mut self, new_name: String ){
        self.instance_name = new_name;
    }

    pub fn gen_invocation(&mut self) {
        let mut cmd: String = self.java.clone();
        cmd.push_str(format!(" -Dfml.ignoreInvalidMinecraftCertificates=true  -Djava.library.path=./bin ").as_str());

        match &self.custom_args {
            Some(args) => {
                cmd.push_str(format!(" {} ", args).as_str());
            }
            None => (),
        }

        // classpaths
        cmd.push_str(" -cp ");
        //if cfg!(windows) {
        //    cmd.push_str("\"")
        //}
        
        for cp in self.classpaths.clone() {
            let cp_str = if cfg!(windows) {
                format!("{};", cp.display())
            } else {
                format!("\"{}\":", cp.display())
            };
            cmd.push_str(cp_str.as_str());
        }
        //if cfg!(windows) {
            //cmd.push_str("\"")
        //}

        // do user info separately
        let userinfo_string = format!(" --accessToken {} --username {} --uuid {} ", 
                                      self.auth_token, self.user_name, self.uuid);
        self.args.push_str(userinfo_string.as_str());

        // main class
        cmd.push_str(format!(" {} {} ", self.main, self.args).as_str());

        self.ccmd = Some(cmd);
    }

    pub fn display_invocation(&self) {
        match self.ccmd.clone() {
            Some(cmd) => {
                println!("{}", cmd);
            },
            None => println!("No command ") 
        }
    }

    pub fn export_as_json(&mut self, path: PathBuf) {
        let mut file = std::fs::File::create(path).expect("Error writing command to file...");

        let custom_args = match &self.custom_args {
            Some(args) => args.as_str(),
            None => "",
        };

        let new_uuid = Uuid::new_v4();

        let instance_type_str = match &self.instance_type {
            InstanceType::Forge => "FORGE",
            InstanceType::Vanilla => "VANILLA",
            InstanceType::Fabric => "FABRIC",
        };

        //using relative bin path
        let serialized_invoker_data = json!({
            "java":"java",
            "binpath" : "./bin",
            "custom_args": custom_args,
            "classpaths" : self.classpaths,
            "mainclass" : self.main,
            "game_args" : self.args,
            "user_name" : self.user_name,
            "instance_name" : self.instance_name,
            "auth_token" : self.auth_token,
            "instance_uuid" : new_uuid.to_string(),
            "instance_type": instance_type_str,
            "uuid": self.uuid.to_string()
        });

        let data = serde_json::to_string(&serialized_invoker_data)
            .expect("Couldn't convert json to string");
        file.write_all(data.as_bytes()).unwrap();
    }

    pub fn get_cmd(&mut self) -> String {
        match self.ccmd.clone() {
            Some(v) => return v,
            None => {
                self.gen_invocation();
                return self.get_cmd();
            }
        };
    }

    pub fn invoke(&mut self, instance_path: PathBuf, verbose : bool) {
        let mut cps = "\"".to_string();
        for cp in &self.classpaths {
            cps.push_str(format!("{};", cp.display()).as_str());
        }
        cps.push_str("\"");


        self.gen_invocation();
        //self.display_invocation();
        
        if verbose {

            println!("{}", self.ccmd.clone().unwrap());
            dbg!(self.ccmd.clone().unwrap()); 
            dbg!(instance_path.clone());
            // keep output in terminal and keep subprocess
            Exec::shell(self.ccmd.clone().unwrap())
                .cwd(instance_path)
                .popen()
                .unwrap();
        }else{
            Exec::shell(self.ccmd.clone().unwrap())
                .cwd(instance_path)
                .stdout(Redirection::Pipe)
                .popen()
                .unwrap()
                .detach();
 
        }
         // detach the process after launching

        std::process::exit(0);
    }
}

impl From<&PathBuf> for Invoker {
    fn from(fp: &PathBuf) -> Self {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(fp)
            .expect("Couldn't open file");

        let invoker_json: serde_json::Value =
            serde_json::from_reader(file).expect("Couldn't parse invoker json file");

        let binpath = invoker_json["binpath"].as_str().unwrap();
        let c_paths = invoker_json["classpaths"].as_array().unwrap();
        let c_args = invoker_json["custom_args"].as_str().unwrap();
        let game_args = invoker_json["game_args"].as_str().unwrap();
        let main_class = invoker_json["mainclass"].as_str().unwrap();
        let java_path = invoker_json["java"].as_str().unwrap();
        let instance_name = invoker_json["instance_name"].as_str().unwrap();
        let user_name = invoker_json["user_name"].as_str().unwrap();
        let auth_token = invoker_json["auth_token"].as_str().unwrap();
        let uuid = invoker_json["uuid"].as_str().unwrap();
        let instance_type_str = invoker_json["instance_type"].as_str().unwrap();


        let mut classpaths_vec: Vec<PathBuf> = Vec::new();
        for path in c_paths {
            let path_str = path.as_str().unwrap();
            classpaths_vec.push(PathBuf::from(path_str));
        }

        Invoker {
            java: String::from(java_path),
            custom_args: Some(String::from(c_args)),
            binpath: PathBuf::from(binpath),
            classpaths: classpaths_vec,
            args: String::from(game_args),
            main: String::from(main_class),
            ccmd: Some(String::from("")),
            instance_name: instance_name.to_string(),
            user_name: user_name.to_string(),
            auth_token: auth_token.to_string(),
            instance_type: match instance_type_str {
                "FORGE"     => InstanceType::Forge,
                "VANILLA"   => InstanceType::Vanilla,
                "FABRIC"    => InstanceType::Fabric,
                _ => InstanceType::Vanilla,
            },
            uuid: uuid.to_string()
        }
    }
}
