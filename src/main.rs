/*
    beginnings of a slideshow program?

TODO: Please rewrite the tokenizer.
*/
mod markup;
use self::markup::*;

/*
I kind of want the slideshow to be a state machine API.
Kind of in the same way OpenGL is I guess.

So the commands set some background state to apply to the next element
or whatever.

At least that's how I have the slideshow file laid out.
Since this is just to compile the slide it's not really a big issue to do so.

The actual slide I guess can be compiled ahead of time.
I should probably do slide livereloading cause it's cool and also cause slides
are pretty visual and would probably be best with feedback
 */
#[derive(Debug, Clone)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

fn hexadecimal_to_decimal(literal: &str) -> u8 {
    let mut result : u8 = 0;
    for (index, ch) in literal.chars().enumerate() {
        let index = (literal.len()-1) - index;
        result +=
            match ch {
                '0' => {0}, '1' => {1}, '2' => {2},
                '3' => {3}, '4' => {4}, '5' => {5},
                '6' => {6}, '7' => {7}, '8' => {8},
                '9' => {9}, 'A' => {10}, 'B' => {11},
                'C' => {12}, 'D' => {13}, 'E' => {14}, 'F' => {15},
                _ => { panic!("undefined character for hexadecimal literal!"); }
            } * 16_u8.pow(index as u32);
    }
    result
}

impl Color {
    fn new(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color {r, g, b, a}
    }

    fn parse_hexadecimal_literal(hex: &str) -> Option<Color> {
        // TODO : This is not perfectly safe since it'll be fine up to
        // 255... This needs to be templated.

        if let Some('#') = hex.chars().nth(0) {
            let hex = &hex[1..];
            match hex.len() {
                // FFFFFF
                // FFFFFFFF
                6 | 8 => {
                    Some(Color::new(
                        hexadecimal_to_decimal(&hex[0..2]),
                        hexadecimal_to_decimal(&hex[2..4]),
                        hexadecimal_to_decimal(&hex[4..6]),
                        if hex.len() == 8 {
                            hexadecimal_to_decimal(&hex[6..8])
                        } else {
                            255
                        }
                    ))
                },
                _ => {
                    println!("Error parsing hexadecimal literal");
                    None
                }
            }
        } else {
            None
        }
    }
}
const COLOR_WHITE : Color = Color {r: 255, g: 255, b: 255, a: 255};
const COLOR_BLACK : Color = Color {r: 0, g: 0, b: 0, a: 0};

#[derive(Debug)]
struct TextElement {
    /*font?*/
    x: f32,
    y: f32,
    text: String, // In the case I allow variables or something...
}
#[derive(Debug)]
struct Page {
    background_color: Color,
    /*
    A Page is probably just going to consist of "elements"
    
    Like a TextElement
    and ImageElement
    and ShapeElement
    or whatever primitives I want to add...
     */
    text_elements: Vec<TextElement>,
}

impl Default for Page {
    fn default() -> Page {
        Page {
            background_color: COLOR_WHITE,
            text_elements: Vec::new()
        }
    }
}

type Slide = Vec<Page>;
/*
    stupid state machine
*/
struct SlideSettingsContext {
    current_background_color: Color,
    current_element_color: Color,
}

impl Default for SlideSettingsContext {
    fn default() -> SlideSettingsContext {
        SlideSettingsContext{
            current_background_color: COLOR_WHITE,
            current_element_color: COLOR_BLACK,
        }
    }
}

#[derive(Debug)]
// tokenized commands.
struct SlideLineCommand <'a> {
    name: &'a str,
    args: Vec<&'a str>,
}

#[derive(Debug)]
enum Command <'a> {
    Reset,
    SetFont(&'a str),
    SetBackgroundColor(Color),
    SetColor(Color),
}

/*
    I need to write a proper tokenizer. Instead of this
FIXME
*/
fn parse_slide_command(line : &str) -> Option<Vec<SlideLineCommand>> {
    /*Since I still lex by char, I really only need to support string literals...*/
    let mut tokenized_first_pass : Vec<&str> = Vec::new();
    let mut char_iterator = line.chars().enumerate();

    fn special_character(character: char) -> Option<char> {
        match character {
            '$' | ':' | '\"' | ' ' | '\t' | '\n' | '\r' => Some(character),
            _ => None
        }
    }

    // holy hell this is warty... Someone please teach me how to do this better...
    if let Some((_, character)) = char_iterator.next() {
        if character == '$' {
            while let Some((index, character)) = char_iterator.next() {
                match special_character(character) {
                    Some(character) => {
                        match character {
                            '$' => { panic!("error?"); },
                            '\"' => {
                                let start = index+1;
                                let end : Option<usize> = line[start..].find('\"');
                                if let Some(end) = end {
                                    let string_literal = &line[start..(start+end)];
                                    tokenized_first_pass.push(string_literal);
                                    for _ in 0..end { char_iterator.next(); }
                                }
                            },
                            _ => {
                            }
                        }
                    },
                    None => {
                        let start = index;
                        let end : Option<usize> =
                            loop {
                                if let Some((index, character)) = char_iterator.next() {
                                    match character {
                                        '\"' | ' ' | ':' | '\n' => {
                                            break Some(index);
                                        }
                                        _ => {}
                                    }
                                } else {
                                    break Some(line.len());
                                }
                            };
                        if let Some(end) = end { 
                            let token_value = &line[start..end];
                            tokenized_first_pass.push(token_value);
                            if let Some(':') = line.chars().nth(end) {
                                tokenized_first_pass.push(":");
                            }
                        }
                    }
                }
            }
        }
    }

    let mut commands : Vec<SlideLineCommand> = Vec::new();

    // No, actually it's just cause I want to write discount C, and
    // Rust is making me realize why it's not a good idea...
    /*TODO: Rust lexers are really painful with iterators... Oh god.*/
    /*No support for compound commands yet.*/
    if tokenized_first_pass.len() >= 1 {
        let mut token_iterator = tokenized_first_pass.iter();

        while let Some(token) = token_iterator.next() {
            match token {
                _ => {
                    let name = token;
                    let mut args : Vec<&str> = Vec::new();
                    while let Some(token) = &token_iterator.next() {
                        match **token {
                            ":" => {
                                if let Some(token) = &token_iterator.next() {
                                    args.push(token);
                                }
                            },
                            _ => { break; },
                        }
                    }
                    commands.push(SlideLineCommand{ name, args});
                }
            }
        }
    }

    // println!("commands {:?}", commands);

    if commands.len() >= 1 {
        Some(commands)
    } else {
        None
    }
}

// Tokenizes a command into a real command.
// TODO!
fn parse_single_command<'a>(command: SlideLineCommand<'a>) -> Option<Command<'a>> {
    let mut args = command.args.iter();

    match command.name {
        "color" | "background_color" => {
            if let Some(next) = &args.next() {
                let color = Color::parse_hexadecimal_literal(next);
                if let Some(color) = color {
                    Some(if command.name == "color" { Command::SetColor(color) }
                         else { Command::SetBackgroundColor(color) })
                } else {
                    None
                }
            } else {
                None
            }
        },
        "font" => {
            if let Some(next) = &args.next() {
                // println!("next: {}", next);
                Some(Command::SetFont(next))
            } else {
                None
            }
        },
        _ => { None },
    }
}

// TODO!
fn execute_command(context: &mut SlideSettingsContext, command: Command) {
    match command {
        Command::SetColor(color) => {context.current_element_color = color;},
        Command::SetBackgroundColor(color) => {context.current_background_color = color;},
        _ => { println!("{:?} is an unknown command", command); }
    }
}

// This will call parse command and execute command
fn handle_command(context: &mut SlideSettingsContext, command: SlideLineCommand) {
    let command = parse_single_command(command);
    if let Some(command) = command {
        execute_command(context, command);
    }
}

fn parse_page(context: &mut SlideSettingsContext, page_lines: Vec<&str>) -> Page {
    let mut current_line : u32 = 0;
    let mut new_page : Page = Page::default();

    // TODO: Account for dynamically sized text.
    // println!("lines: {:?}", page_lines);
    for line in page_lines {
        if let Some(commands) = parse_slide_command(&line) {
            for command in commands {
                handle_command(context, command);
                // update page properties based on command...
                {
                    new_page.background_color = context.current_background_color.clone();
                }
            }
        } else {
            if line.len() >= 1 {
                println!("plain text: {}", line);
                new_page.text_elements.push(TextElement{x: 0.0, y: current_line as f32, text: String::from(line)});
            }
            current_line += 1;
        }
    }

    new_page
}

fn compile_slide(slide_source : &String) -> Slide {
    let mut slide = Slide::new();
    let mut current_context = SlideSettingsContext::default();
    // The compiler "state" is "global" but I should probably reject
    // any text that isn't inside a currently compiled page...
    // text outside of slides should be a warning though and should probably
    // just be treated like a comment.
    let mut line_iterator = slide_source.lines().enumerate();
    while let Some((index, line)) = line_iterator.next() {
        match parse_slide_command(&line) {
            Some(commands) => {
                if commands[0].name == "page" {
                    let end_page_index = loop {
                        let next = line_iterator.next();
                        match next {
                            Some((index, line)) => {
                                if let Some(commands) = parse_slide_command(&line) {
                                    if commands[0].name == "end_page" {
                                        break Some(index);
                                    }
                                }
                            },
                            None => {
                                break None;
                            }
                        }
                    };

                    if let Some(end_page_index) = end_page_index {
                        let index = index+1;
                        let end_page_index = end_page_index;
                        let page_source_lines : Vec<&str> = slide_source.lines().collect();
                        let new_page = parse_page(&mut current_context, page_source_lines[index..end_page_index].to_vec());
                        slide.push(new_page);
                    } else {
                        println!("Error! EOF before an end page!");
                    }
                } else {
                    for command in commands {
                        handle_command(&mut current_context, command);
                    }
                }
            },
            None => {
                // TODO Handle $$ escaping...
                // Well actually it would probably work if I just made a custom format function
                // that could handle the escaping for me.
                println!("warning: Plain text should not be outside of a page!");
            },
        }
    }

    slide
}

fn remove_comments_from_source(source : &str) -> String {
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

fn load_file(file_name: &str) -> String {
    use std::io::Read;
    use std::fs::File;

    let mut result = String::new();
    let mut slide_file = File::open(file_name)
        .expect("There was an error in reading the file!");
    slide_file.read_to_string(&mut result).expect("Unable to read into string");
    result
}

extern crate sdl2;

use sdl2::pixels::Color as SDLColor;
use sdl2::event::Event as SDLEvent;
use sdl2::keyboard::Keycode as SDLKeycode;

fn clamp_i32(x: i32, min: i32, max: i32) -> i32 {
    if x < min {
        min
    } else if x > max {
        max
    } else {
        x
    }
}

fn main() {
    if false {
        println!("Testing markup");
        let source_test = "This is a *thing* Cool_right_ _sad _t t_";
        let markup_lex = MarkupLexer::new(source_test);
        for item in markup_lex {
            println!("{:#?}", item);
        }
        let markup_lex = MarkupLexer::new(source_test);
        println!("STITCHED TOGETHER STRING: {}", markup_lex.stitch());
    }

    let sdl2_context = sdl2::init().expect("SDL2 failed to initialize?");
    let video_subsystem = sdl2_context.video().unwrap();

    let sdl2_ttf_context = sdl2::ttf::init().expect("SDL2 ttf failed to initialize?");

    let slideshow_source = load_file("test.slide");
    let slideshow_source = remove_comments_from_source(&slideshow_source);
    let slideshow = compile_slide(&slideshow_source);

    const DEFAULT_WINDOW_WIDTH : u32 = 1280;
    const DEFAULT_WINDOW_HEIGHT : u32 = 720;
    let window = video_subsystem.window("stupid slideshow", DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT)
        .position_centered()
        .build()
        .expect("Window failed to open?");

    let mut window_canvas = window.into_canvas().build().unwrap();
    let texture_creator = window_canvas.texture_creator(); 

    let mut default_font = sdl2_ttf_context.load_font("data/fonts/libre-baskerville/LibreBaskerville-Regular.ttf", 36).unwrap();

    let mut running = true;

    let mut event_pump = sdl2_context.event_pump().unwrap();
    let mut current_slide_index : i32 = 0;

    while running {
        for event in event_pump.poll_iter() {
            match event {
                SDLEvent::Quit {..} =>  {running = false;},
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::Right), .. } => {
                    current_slide_index += 1;
                },
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::Left), .. } => {
                    current_slide_index -= 1;
                },
                _ => {}
            }
        }

        current_slide_index = clamp_i32(current_slide_index as i32, 0, slideshow.len() as i32);
        let current_slide : Option<&Page> = slideshow.get(current_slide_index as usize);

        println!("current-slide: {}", current_slide_index);
        if let Some(current_slide) = current_slide {
            // rendering the slide
            {
                window_canvas.set_draw_color(
                    SDLColor::RGBA(current_slide.background_color.r,
                                   current_slide.background_color.g,
                                   current_slide.background_color.b,
                                   current_slide.background_color.a));
                window_canvas.clear();

                for text in &current_slide.text_elements {
                    let mut cursor_x : f32 = text.x;
                    let mut cursor_y : f32 = text.y;

                    let mut texture = texture_creator.create_texture_from_surface(
                        &default_font.render(&text.text)
                            .blended(SDLColor::RGBA(255, 255, 255, 255)).expect("how did this go wrong?")
                    ).expect("how did this go wrong?");
                    window_canvas.set_draw_color(SDLColor::RGBA(0, 0, 0, 255));
                    let sdl2::render::TextureQuery { width, height, .. } = texture.query();
                    texture.set_color_mod(0, 0, 0);
                    texture.set_alpha_mod(255);
                    window_canvas.copy(&texture, None, Some(sdl2::rect::Rect::new(0, (text.y * height as f32) as i32, width, height)));

                    if false {
                        let markup_lexer = MarkupLexer::new(&text.text);
                        for markup in markup_lexer {
                            match markup {
                                Markup::Plain(text_content) => {
                                },
                                Markup::Bold(text_content) => {
                                },
                                Markup::Strikethrough(text_content) => {
                                },
                                Markup::Italics(text_content) => {
                                },
                                Markup::Underlined(text_content) => {
                                }
                            }
                        }
                    }
                }
            }
        } else {
            window_canvas.set_draw_color(SDLColor::RGB(10, 10, 16));
            window_canvas.clear();
            let mut texture = texture_creator.create_texture_from_surface(
                &default_font.render("stupid slide needs pages... Feed me")
                    .blended(SDLColor::RGBA(255, 255, 255, 255)).expect("how did this go wrong?")
            ).expect("how did this go wrong?");
            window_canvas.set_draw_color(SDLColor::RGBA(0, 0, 0, 255));
            let sdl2::render::TextureQuery { width, height, .. } = texture.query();
            window_canvas.copy(&texture, None, Some(sdl2::rect::Rect::new((DEFAULT_WINDOW_WIDTH as i32 / 2) - width as i32 / 2, (DEFAULT_WINDOW_HEIGHT as i32 / 2) - height as i32 / 2, width, height)));
        }

        window_canvas.present();
    }
}
