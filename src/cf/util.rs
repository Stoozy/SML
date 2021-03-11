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
