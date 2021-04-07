use std::io::Write;
use std::path::PathBuf;

pub struct Invoker {
    java: String,
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

    pub fn display_invocation(&self) -> () {
        println!("{}", self.ccmd.clone().unwrap());
    }

    pub fn save_invocation_to_file(&self, path: PathBuf) {
        let cmd = self.ccmd.clone().unwrap();

        if !path.exists() {
            std::fs::create_dir_all(path.clone()).unwrap();
        }

        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .unwrap();

        file.write_all(cmd.as_bytes()).unwrap();
    }

    pub fn invoke(&self) -> () {
        // make sure command is not empty
        if !self.ccmd.is_none() {
            // open subprocess with command here ...
        }
    }
}
