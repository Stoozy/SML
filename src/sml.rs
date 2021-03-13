use crate::cf::CFFile;
use crate::ima::Instance;
use ftp::FtpStream;
use std::fs;
use std::io::Write;
use zip::ZipArchive;

pub fn handle_stage_unix(chosen_proj: CFFile, instance: Instance) {
    let stage_file_remote_path = format!("/shares/U/sml/{}-linux.zip", chosen_proj.version);

    let mut stage_filepath = instance.get_path();
    stage_filepath.pop();

    stage_filepath.push(format!("{}-linux.zip", chosen_proj.version));
    println!("MC Version is: {}", chosen_proj.version);

    // request server for sml stage
    let mut ftp_stream = FtpStream::connect("98.14.42.52:21").unwrap();
    let _ = ftp_stream.login("", "").unwrap();

    let stage_file_stream = ftp_stream
        .simple_retr(stage_file_remote_path.as_str())
        .unwrap();

    match fs::File::create(stage_filepath.clone()) {
        Err(e) => panic!("Couldn't create file {}", e),
        Ok(mut file) => {
            file.write(&stage_file_stream.into_inner()).unwrap();
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
