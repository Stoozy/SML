use serde_json::json;
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
}

impl Invoker {
    pub fn new(jp: String, bp: PathBuf, cp: Vec<PathBuf>, a: String, mc: String) -> Invoker {
        Invoker {
            java: jp,
            custom_args: None,
            binpath: bp,
            classpaths: cp,
            args: a,
            main: mc,
            ccmd: None,
        }
    }

    pub fn gen_invocation(&mut self) {
        let mut cmd: String = self.java.clone();
        cmd.push_str(format!(" -Djava.library.path={}", self.binpath.display()).as_str());

        match &self.custom_args {
            Some(args) => {
                cmd.push_str(format!(" {}", args).as_str());
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

        // main class
        cmd.push_str(format!(" {} {}", self.main, self.args).as_str());

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
            "game_args" : self.args
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
        //let mut inovker_path = self.binpath.clone();
        //// pop bin/
        //invoker_path.pop();
        //// push filename
        //invoker_path.push_str("sml_invoker.json");
        //let mut invoker_file = OpenOptions::new()
        //                        .read(true)
        //                        .write(false)
        //                        .open(invoker_path)
        //                        .expect("Unable to open sml invoker file");

        //let mut invoker_json = serde_json::from_reader(invoker_file);
        //let binarg = format!("-Djava.library.path=\"{}\"", self.binpath.display());

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
            .popen()
            .unwrap();

        //let cmd = Command::new("java")
        //    .arg("-cp")
        //    .arg(cps)
        //    .arg(self.main.clone())
        //    .arg(self.args.clone())
        //    .current_dir(cwd);

        //cmd
        //    .output()
        //    .expect("Failed to launch instance");

        //let adir = cmd.get_current_dir().unwrap();
        //println!("{}", adir.display())
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
        }
    }
}
