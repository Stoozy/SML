
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
