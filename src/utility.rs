pub fn clamp_i32(x: i32, min: i32, max: i32) -> i32 {
    if x < min {
        min
    } else if x > max {
        max
    } else {
        x
    }
}

pub fn remove_comments_from_source(source : &str) -> String {
    let mut filtered = String::new();
    let lines : Vec<&str> = source.split("\n").collect();
    for line in lines.iter() {
        if !(line.chars().nth(0) == Some('#')) {
            for character in line.chars() {
                filtered.push(character);
            }
            filtered.push('\n');
        }
    }
    filtered
}

pub fn load_file(file_name: &str) -> Result<String, &'static str> {
    use std::io::Read;
    use std::fs::File;

    match File::open(file_name) {
        Ok(mut slide_file) => {
            let mut result = String::new();
            slide_file.read_to_string(&mut result)
                .expect("Unable to read into string");
            Ok(result)
        },
        Err(_) => {
            Err("Bad file")
        }
    }
}
