use serde_json::json;
use std::io::Write;
use std::path::PathBuf;

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
        cmd.push_str(format!(" -Djava.library.path=\"{}\" ", self.binpath.display()).as_str());

        match &self.custom_args {
            Some(args) => {
                cmd.push_str(format!(" {}", args).as_str());
            }
            None => (),
        }

        // classpaths
        cmd.push_str(" -cp ");
        for cp in self.classpaths.clone() {
            let cp_str = format!("\"{}\":", cp.display());
            cmd.push_str(cp_str.as_str());
        }

        // main class
        cmd.push_str(format!(" {} {}", self.main, self.args).as_str());

        self.ccmd = Some(cmd);
    }

    pub fn display_invocation(&self) {
        println!("{}", self.ccmd.clone().unwrap());
    }

    pub fn export_as_json(&mut self, path: PathBuf) {
        let cmd = self.ccmd.clone().unwrap();
        let mut file = std::fs::File::create(path).expect("Error writing command to file...");
        let binpath_arg = format!(" -Djava.library.path=\"{}\" ", self.binpath.display());

        // classpaths string
        let mut cp_arg = "".to_string();
        cp_arg.push_str(" -cp ");
        for cp in self.classpaths.clone() {
            let cp_str = format!("\"{}\":", cp.display());
            cp_arg.push_str(cp_str.as_str());
        }

        let custom_args = match &self.custom_args {
            Some(args) => args.as_str(),
            None => "",
        };

        let serialized_invoker_data = json!({
            "java":"java",
            "binpath" : binpath_arg,
            "custom_args": custom_args,
            "classpaths" : cp_arg,
            "mainclass" : self.main,
            "game_args" : self.args
        });

        let data = serde_json::to_string(&serialized_invoker_data)
            .expect("Couldn't convert json to string");
        file.write_all(data.as_bytes()).unwrap();
    }

    pub fn invoke(&self) {
        // make sure command is not empty
        if self.ccmd.is_some() {
            // open subprocess with command here ...
        }
    }
}
