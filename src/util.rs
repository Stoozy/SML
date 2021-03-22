use std::io;
pub fn get_u64() -> Option<u64> {
    let mut input_text = String::new();
    io::stdin()
        .read_line(&mut input_text)
        .expect("Failed to get input");

    Some(
        input_text
            .trim()
            .parse::<u64>()
            .expect("Error parsing number"),
    )
}

pub fn is_greater_version(version1: &str, version2: &str) -> bool {
    // serialize version as int and check if greater
    let v1_c : Vec<&str> = version1.split(".").collect();
    let v2_c : Vec<&str> = version2.split(".").collect();

    let mut v1 = 0;
    let mut v2 = 0;
    
    for i in v1_c { v1 = v1*10 + i.parse::<i32>().unwrap(); }
    for i in v2_c { v2 = v2*10 + i.parse::<i32>().unwrap(); }

    v1 > v2
}

pub fn get_forge_args(json: serde_json::Value) -> Option<String>{
    let mut retstr = String::new();
    // get args here
    let args = json["arguments"]["game"].as_array().unwrap();
    for arg in args{
        retstr.push(' ');
        retstr.push_str(arg.as_str().unwrap());
    } 

    Some(retstr)
}

