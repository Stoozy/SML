use crate::cf::CFFile;
use crate::ima::Instance;
use ftp::{FtpStream};
use std::{fs};
use std::io::Write;
use zip::ZipArchive;
use crate::downloader::Downloader;
use indicatif::{ProgressBar, ProgressStyle};

const CHUNK_SIZE : usize = 8192;

pub struct Invoker{
    java : String,
    libpath: PathBuf,
    classpaths: Vec<PathBuf>,
    args: String,
    main: String
    ccmd: Option<String>
}

impl Invoker{

    pub fn new(jp: String, lp: PathBuf, cp: Vec<PathBuf>, a: String, mc: String) -> Invoker{
        Invoker{java:jp, libpath: lp, classpaths:cp, args: a, main:mc, ccmd: None}
    }


    pub fn gen_invocation(& self) {
        let cmd : String = java;
        cmd.push(format!(" -Djava.library.path={} ", libpath.display()));

        // classpaths
        cmd.push(" -cp "); 
        for cp in self.classpaths {
            let cp_str = format!("{}:", cp.display());
            cmd.push(cp_str);
        }

        // main class
        cmd.push(format!(" {} {}", main, args);

        self.ccmd = cmd;
    }

    pub fn invoke(& self) -> Result<(), Error> {
        // make sure command is not empty
        if self.ccmd.is_none(){
            // return error here
        }

        // open subprocess with command here ...
    }

}

// gets and extracts sml stage for forge version
pub fn get_stage(chosen_proj: CFFile, instance: Instance) {
    let stage_file_remote_path = format!("/shares/U/sml/{}-linux.zip", chosen_proj.version);

    let mut stage_filepath = instance.get_path();
    stage_filepath.pop();

    stage_filepath.push(format!("{}-linux.zip", chosen_proj.version));
    println!("MC Version is: {}", chosen_proj.version);

    // request server for sml stage
    let mut ftp_stream = FtpStream::connect("98.14.42.52:21").unwrap();
    let _ = ftp_stream.login("", "").unwrap();

    match fs::File::create(stage_filepath.clone()) {
        Err(e) => panic!("Couldn't create file {}", e),
        Ok(mut file) => {                
            let total_size = ftp_stream.size(stage_file_remote_path.as_str()).unwrap().unwrap() as u64;

            println!("Got total file size: {}", total_size);
            

            let pb = ProgressBar::new(total_size);
            pb.set_style(ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .progress_chars("=> "));


            let data = ftp_stream.simple_retr(stage_file_remote_path.as_str()).unwrap().into_inner();
            for i in 0..data.len()/CHUNK_SIZE{
                if(i != (data.len()/CHUNK_SIZE)-1){
                    file.write_all(& data[i*CHUNK_SIZE..(i+1)*CHUNK_SIZE]);
                    pb.set_position((i*CHUNK_SIZE) as u64);
                }else{                        
                    // write the entire last part 
                    file.write_all(& data[i*CHUNK_SIZE..]);
                    pb.set_position((i*CHUNK_SIZE) as u64);
                }
            }
            pb.finish_with_message("Finished downloading stage file");
        }
    };

    let file = fs::File::open(stage_filepath.clone()).expect("Error getting zip file");
    let mut zip = ZipArchive::new(file).unwrap();
    let extract_path = instance.get_path();
    //extract_path.pop();

    zip.extract(extract_path).expect("Error extracting forge");
    println!("Sucessfully extracted forge stage");
    println!("Cleaning up");
    fs::remove_file(stage_filepath).expect("Error deleting stage zip file");
}


pub fn get_modslist(chosen_proj: CFFile, instance: Instance){

    let download_url = chosen_proj.get_download_url();
    let mut download_path = instance.get_path();
    download_path.push("mods/");
    if !download_path.exists() {
        fs::create_dir(download_path.clone()).expect("Error creating mods folder");
    }
    download_path.push(chosen_proj.name.clone());

    println!("Got download url {}", download_url);
    println!("Got download path {}", download_path.display());

    let mut downloader = Downloader::new(download_url, download_path.clone());
    downloader.download().expect("Error downloading modslist");

    let mut mod_dirpath = instance.get_path().clone();
    mod_dirpath.push("mods/");

    // extract zip
    let modpack_zip = fs::File::open(download_path.clone()).expect("Couldn't open modslist");
    println!("Downloaded mods list");

    println!("Extracting mods list");
    let mut zip = ZipArchive::new(modpack_zip).unwrap();
    let mut extract_path = download_path.clone();
    extract_path.pop();

    zip.extract(extract_path)
        .expect("Error extracting mods list");

    fs::remove_file(download_path.clone()).expect("Error deleting stage zip file");
}