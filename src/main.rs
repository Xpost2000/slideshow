/*
    beginnings of a slideshow program?

TODO: Please rewrite the tokenizer.
TODO: Fix lifetimes...
*/
mod markup;
use self::markup::*;
mod utility;
use self::utility::*;
mod graphics_context;
use self::graphics_context::*;
mod color;
use self::color::*;

#[derive(Debug)]
struct TextElement {
    /*font?*/
    x: f32,
    y: f32, // TODO: Rename to linebreaks between. Or something
    text: String, // In the case I allow variables or something...
    color: Color,
    font_size: u16,
    font_name: Option<String>,
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

struct Slide {
    file_name : String, // owned string for hot reloading.
    pages : Vec<Page>,
    current_page : isize,
}

impl Slide {
    fn len(&self) -> usize {
        self.pages.len()
    }

    fn get_current_page(&self) -> Option<&Page> {
        self.get(self.current_page as usize)
    }

    fn get(&self, index: usize) -> Option<&Page> {
        self.pages.get(index)
    }

    fn next_page(&mut self) {
        self.current_page += 1;
        self.current_page = clamp_i32(self.current_page as i32, 0, self.len() as i32) as isize;
    }

    fn previous_page(&mut self) {
        self.current_page -= 1;
        self.current_page = clamp_i32(self.current_page as i32, 0, self.len() as i32) as isize;
    }

    fn new_from_file(file_name: &str) -> Option<Slide> {
        match load_file(file_name) {
            Ok(file_source) => {
                let slideshow_source = remove_comments_from_source(&file_source);
                Some(
                    Slide {
                        file_name: file_name.to_owned(),
                        pages: compile_slide_pages(&slideshow_source),
                        current_page: 0
                    }
                )
            },
            Err(_) => {
                None
            }
        }
    }
}
// type Slide = Vec<Page>;
/*
    stupid state machine
*/
struct SlideSettingsContext {
    current_background_color: Color,
    current_element_color: Color,
    current_font_size: u16,
    current_font_path: Option<String>,
}

impl Default for SlideSettingsContext {
    fn default() -> SlideSettingsContext {
        SlideSettingsContext{
            current_background_color: COLOR_WHITE,
            current_element_color: COLOR_BLACK,
            current_font_size: 48,
            current_font_path: None,
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
    Reset, // Total reset
    ResetFont, // TODO think of better thing.
    SetFont(&'a str),
    SetBackgroundColor(Color),
    SetColor(Color),
    SetFontSize(u16),
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
                            '$' => { return None; },
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
        "font-size" => {
            if let Some(next) = &args.next() {
                match next.parse::<u16>() {
                    Ok(value) => { Some(Command::SetFontSize(value)) },
                    Err(_) => None
                }
            } else {
                None
            }
        }
        "reset-font" => {
            Some(Command::ResetFont)
        }
        _ => { None },
    }
}

// TODO!
fn execute_command(context: &mut SlideSettingsContext, command: Command) {
    match command {
        Command::SetColor(color) => {context.current_element_color = color;},
        Command::SetBackgroundColor(color) => {context.current_background_color = color;},
        Command::SetFontSize(font_size) => {context.current_font_size = font_size;}
        // the compiled slide should not depend on the source...
        Command::SetFont(font_name) => {context.current_font_path = Some(font_name.to_owned());},
        Command::ResetFont => {context.current_font_path = None;},
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

    for line in page_lines {
        if let Some(commands) = parse_slide_command(&line) {
            for command in commands {
                handle_command(context, command);
                // update page properties based on command...
                {
                    new_page.background_color = context.current_background_color;
                }
            }
        } else {
            if line.len() >= 1 {
                new_page.text_elements.push(TextElement{x: 0.0,
                                                        y: current_line as f32,
                                                        text: String::from(
                                                            // VERY BAD STRING ESCAPE FOR NOW.
                                                            if let Some('$') = line.chars().nth(0) {
                                                                &line[1..]
                                                            } else {
                                                                &line
                                                            }
                                                        ),
                                                        font_size: context.current_font_size,
                                                        font_name: context.current_font_path.clone(),
                                                        color: context.current_element_color});
                current_line = 0;
            } else {
                current_line += 1;
            }
        }
    }

    new_page
}

fn compile_slide_pages(slide_source : &String) -> Vec<Page> {
    let mut slide = Vec::new();
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

use sdl2::event::Event as SDLEvent;
use sdl2::keyboard::Keycode as SDLKeycode;

fn main() {
    let sdl2_context = sdl2::init().expect("SDL2 failed to initialize?");
    let video_subsystem = sdl2_context.video().unwrap();

    let sdl2_ttf_context = sdl2::ttf::init().expect("SDL2 ttf failed to initialize?");

    const DEFAULT_WINDOW_WIDTH : u32 = 1280;
    const DEFAULT_WINDOW_HEIGHT : u32 = 720;
    let window = video_subsystem.window("stupid slideshow", DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT)
        .position_centered()
        .build()
        .expect("Window failed to open?");

    let mut graphics_context = SDL2GraphicsContext::new(window, &sdl2_ttf_context);
    let default_font = graphics_context.add_font("data/fonts/libre-baskerville/LibreBaskerville-Regular.ttf");

    let mut running = true;

    let mut event_pump = sdl2_context.event_pump().unwrap();
    let mut current_slide_index : i32 = 0;
    
    // bad command line argument handling atm.
    // Just filename or bust.

    use std::env;
    let arguments : Vec<String> = env::args().collect();
    let mut slideshow = Slide::new_from_file(
        match arguments.len() {
            1 => {
                "test.slide"
            }
            2 => {
                &arguments[1]
            },
            _ => {
                println!("The only command line argument should be the slide file!");
                "test.slide"
            }
        }
    );

    while running {
        for event in event_pump.poll_iter() {
            match event {
                SDLEvent::Quit {..} | SDLEvent::KeyDown { keycode: Some(SDLKeycode::Escape), .. } =>  {running = false;},
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::Right), .. } => {
                    if let Some(slideshow) = &mut slideshow {
                        slideshow.next_page();
                    }
                },
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::Left), .. } => {
                    if let Some(slideshow) = &mut slideshow {
                        slideshow.previous_page();
                    }
                },
                _ => {}
            }
        }

        /*
        While I don't explicitly need a state machine... It would probably be good practice
        to do some for this.
         */
        use sdl2::ttf::FontStyle;
        if let Some(slideshow) = &slideshow {
            if let Some(current_slide) = slideshow.get_current_page() {
                // rendering the slide
                graphics_context.clear_color(current_slide.background_color);

                let mut last_font_size : u16 = 0;
                let mut cursor_y : f32 = 0.0;
                for text in &current_slide.text_elements {
                    let font_size = text.font_size;
                    let mut cursor_x : f32 = 0.0;

                    let markup_lexer = MarkupLexer::new(&text.text);
                    let (_, height) = graphics_context.text_dimensions(default_font, &text.text, font_size);
                    // last_font_size should always be non-negative. If we don't have a last just
                    // use the current font size (only for like the first line).
                    if last_font_size == 0 { last_font_size = height as u16; }
                    cursor_y += last_font_size as f32 * text.y;
                    /*
                    I want to remove this.
                    There is a slight chance of this being kept, so I'll just have to factor it later,
                    since the markup splits things into segments which to re-render the whole string
                    require a cursor, to render in the right place. Not a big issue though.
                     */
                    let drawn_font =
                        if let Some(font) = &text.font_name {
                            graphics_context.add_font(font)
                        } else {
                            default_font 
                        };
                    for markup in markup_lexer {
                        let mut width = 0;
                        match markup {
                            Markup::Plain(text_content) => {
                                graphics_context.render_text(drawn_font,
                                                             cursor_x, cursor_y,
                                                             &text_content, font_size,
                                                             text.color,
                                                             FontStyle::NORMAL);
                                width = graphics_context.text_dimensions(drawn_font, &text_content, font_size).0;
                            },
                            Markup::Strikethrough(text_content) => {
                                graphics_context.render_text(drawn_font,
                                                             cursor_x, cursor_y,
                                                             &text_content, font_size,
                                                             text.color,
                                                             FontStyle::NORMAL);
                                width = graphics_context.text_dimensions(drawn_font, &text_content, font_size).0;
                                graphics_context.render_filled_rectangle(cursor_x, cursor_y + (font_size as f32 / 1.8), width as f32, font_size as f32 / 10.0, text.color);
                            },
                            Markup::Underlined(text_content) => {
                                graphics_context.render_text(drawn_font,
                                                             cursor_x, cursor_y,
                                                             &text_content, font_size,
                                                             text.color,
                                                             FontStyle::NORMAL);
                                width = graphics_context.text_dimensions(drawn_font, &text_content, font_size).0;
                                graphics_context.render_filled_rectangle(cursor_x, cursor_y + (font_size as f32), width as f32, font_size as f32 / 13.0, text.color);
                            }
                            Markup::Bold(text_content) => {
                                graphics_context.render_text(drawn_font,
                                                             cursor_x, cursor_y,
                                                             &text_content, font_size,
                                                             text.color,
                                                             FontStyle::BOLD);
                                width = graphics_context.text_dimensions(drawn_font, &text_content, font_size).0;
                            },
                            Markup::Italics(text_content) => {
                                graphics_context.render_text(drawn_font,
                                                             cursor_x, cursor_y,
                                                             &text_content, font_size,
                                                             text.color,
                                                             FontStyle::ITALIC);
                                width = graphics_context.text_dimensions(drawn_font, &text_content, font_size).0;
                            },
                        }
                        cursor_x += width as f32;
                    }

                    cursor_y += (height as f32);
                    last_font_size = font_size;
                }
            } else {
                graphics_context.clear_color(Color::new(10, 10, 16, 255));
                let (width, height) = graphics_context.text_dimensions(default_font, "stupid slide needs pages... feed me", 48);
                graphics_context.render_text(default_font,
                                             ((DEFAULT_WINDOW_WIDTH as i32 / 2) - (width as i32) / 2) as f32,
                                             ((DEFAULT_WINDOW_HEIGHT as i32 / 2) - (height as i32) / 2) as f32,
                                             "stupid slide needs pages... feed me", 48, COLOR_WHITE,
                                             FontStyle::NORMAL);
            }
        } else {
                graphics_context.clear_color(Color::new(10, 10, 16, 255));
                let (width, height) = graphics_context.text_dimensions(default_font, "Invalid / No slide file.", 48);
                graphics_context.render_text(default_font,
                                             ((DEFAULT_WINDOW_WIDTH as i32 / 2) - (width as i32) / 2) as f32,
                                             ((DEFAULT_WINDOW_HEIGHT as i32 / 2) - (height as i32) / 2) as f32,
                                             "Invalid / No slide file", 48, COLOR_WHITE,
                                             FontStyle::NORMAL);
        }

        graphics_context.present();
    }
}
