use serde_json::json;
use subprocess::Redirection;
use std::io::Write;
use std::path::PathBuf;
use subprocess::Exec;

pub struct Invoker {
    java: String,
    custom_args: Option<String>,
    binpath: PathBuf,
    classpaths: Vec<PathBuf>,
    args: String,
    main: String,
    ccmd: Option<String>,
    instance_name: String,
    user_name: String,
    auth_token: String
}

impl Invoker {
    pub fn new(java: String, binpath: PathBuf, classpaths: Vec<PathBuf>, args: String, main: String, instance_name: String, user_name : String, auth_token : String) -> Invoker {
        Invoker {
            java,
            custom_args: None,
            binpath,
            classpaths,
            args,
            main,
            ccmd: None,
            instance_name,
            user_name,
            auth_token
        }
    }

    pub fn gen_invocation(&mut self) {
        let mut cmd: String = self.java.clone();
        cmd.push_str(format!(" -Djava.library.path={}", self.binpath.display()).as_str());

        match &self.custom_args {
            Some(args) => {
                cmd.push_str(format!(" {} ", args).as_str());
            }
            None => (),
        }

        // classpaths
        cmd.push_str(" -cp ");
        if cfg!(windows) {
            cmd.push_str("\"")
        }
        for cp in self.classpaths.clone() {
            let cp_str = if cfg!(windows) {
                format!("{};", cp.display())
            } else {
                format!("\"{}\":", cp.display())
            };
            cmd.push_str(cp_str.as_str());
        }
        if cfg!(windows) {
            cmd.push_str("\"")
        }

        // do user info separately
        let userinfo_string = format!(" --accessToken {} --username {} ", 
                                      self.auth_token, self.user_name);
        self.args.push_str(userinfo_string.as_str());

        // main class
        cmd.push_str(format!(" {} {} ", self.main, self.args).as_str());

        self.ccmd = Some(cmd);
    }

    pub fn display_invocation(&self) {
        println!("{}", self.ccmd.clone().unwrap());
    }

    pub fn export_as_json(&mut self, path: PathBuf) {
        let mut file = std::fs::File::create(path).expect("Error writing command to file...");
        let binpath_arg = format!("{}", self.binpath.display());

        let custom_args = match &self.custom_args {
            Some(args) => args.as_str(),
            None => "",
        };

        let serialized_invoker_data = json!({
            "java":"java",
            "binpath" : binpath_arg,
            "custom_args": custom_args,
            "classpaths" : self.classpaths,
            "mainclass" : self.main,
            "game_args" : self.args,
            "user_name" : self.user_name,
            "instance_name" : self.instance_name,
            "auth_token" : self.auth_token
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

    pub fn invoke(&mut self) {
        let mut cps = "\"".to_string();
        for cp in &self.classpaths {
            cps.push_str(format!("{};", cp.display()).as_str());
        }
        cps.push_str("\"");
        //println!("{}", cps);

        println!("{}", self.ccmd.clone().unwrap());
        let mut cwd = self.binpath.clone();
        cwd.pop();

        self.gen_invocation();
        Exec::shell(self.ccmd.clone().unwrap())
            .cwd(cwd)
            .stdout(Redirection::Pipe)
            .popen()
            .unwrap()
            .detach(); // detach the process after launching

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
        }
    }
}
