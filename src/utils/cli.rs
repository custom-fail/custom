use std::io::stdin;

pub fn confirm() -> bool {
    let mut buffer = String::new();
    stdin().read_line(&mut buffer).unwrap();
    match buffer.to_lowercase().as_str() {
        "y" => true,
        "n" => false,
        _ => confirm()
    }
}