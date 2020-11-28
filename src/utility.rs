enum EasingFunction {
    CubicEaseIn(f32, f32, f32),
    CubicEaseOut(f32, f32, f32),
    QuadraticEaseIn(f32, f32, f32),
    QuadraticEaseOut(f32, f32, f32),
    Linear(f32, f32, f32),
}
pub fn cubic_ease_in(a: f32, b: f32, t: f32) -> f32 {
    b * t * t * t + a
}
pub fn cubic_ease_out(a: f32, b: f32, t: f32) -> f32{
    let t = t - 1.0;
    b * (t * t * t + 1.0) + a
}
pub fn quadratic_ease_in(a: f32, b: f32, t: f32) -> f32 {
    b * t * t + a
}
pub fn quadratic_ease_out(a: f32, b: f32, t: f32) -> f32 {
    -b * t * (t-2.0) + a
}
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    (1.0 - t) * a + t * b
}

impl EasingFunction {
    fn evaluate(&self) -> f32 {
        match *self {
            EasingFunction::CubicEaseIn(a, b, t) => cubic_ease_in(a, b, t),
            EasingFunction::CubicEaseOut(a, b, t) => cubic_ease_out(a, b, t),
            EasingFunction::QuadraticEaseIn(a, b, t) => quadratic_ease_in(a, b, t),
            EasingFunction::QuadraticEaseOut(a, b, t) => quadratic_ease_out(a, b, t),
            EasingFunction::Linear(a, b, t) => lerp(a, b, t),
        }
    }
}

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
