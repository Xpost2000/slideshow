/*
    beginnings of a slideshow program?

TODO: Please rewrite the tokenizer.
TODO: Fix lifetimes...
*/
use std::collections::HashMap;

mod markup;
use self::markup::*;
mod utility;
use self::utility::*;
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
#[derive(Debug, Clone, Copy)]
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

type Slide = Vec<Page>;
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

extern crate sdl2;

use sdl2::pixels::Color as SDLColor;
use sdl2::event::Event as SDLEvent;
use sdl2::keyboard::Keycode as SDLKeycode;

type SDL2WindowCanvas = sdl2::render::Canvas<sdl2::video::Window>;
// hashmaps of hashmaps?
// Yeah this is not a good idea, but whatever for now.

// TODO: convert this to &str.
struct SDL2FontAsset<'ttf> {
    file_name: String,
    stored_sizes : HashMap<u16, sdl2::ttf::Font<'ttf, 'static>>,
}

impl<'ttf> SDL2FontAsset<'ttf> {
    fn new(file_name: String) -> SDL2FontAsset<'ttf> {
        SDL2FontAsset {file_name: file_name.clone(), stored_sizes: HashMap::new(),}
    }

    fn new_with_common_sizes(file_name: String, ttf_context: &'ttf sdl2::ttf::Sdl2TtfContext) -> SDL2FontAsset<'ttf> {
        SDL2FontAsset {
            file_name: file_name.clone(),
            stored_sizes: {
                let mut okay_table_i_guess = HashMap::new();
                // probably common sizes.
                for font_size in &[10, 11, 12, 14, 16, 18, 24, 32, 36, 48, 64, 72, 84, 96, 128] {
                    okay_table_i_guess.insert(*font_size, ttf_context.load_font(file_name.clone(), *font_size).unwrap());
                }

                okay_table_i_guess
            }
        }
    }

    fn load_size_if_not_loaded(&mut self, font_size: u16, ttf_context: &'ttf sdl2::ttf::Sdl2TtfContext) {
        if let None = self.stored_sizes.get(&font_size) {
            self.stored_sizes.insert(font_size, ttf_context.load_font(self.file_name.clone(), font_size).unwrap());
        }
    }

    fn get_size(&mut self, font_size: u16, ttf_context: &'ttf sdl2::ttf::Sdl2TtfContext) -> &sdl2::ttf::Font<'ttf, 'static> {
        self.load_size_if_not_loaded(font_size, ttf_context);
        self.stored_sizes.get(&font_size).unwrap()
    }

    fn get_size_mut(&mut self, font_size: u16, ttf_context: &'ttf sdl2::ttf::Sdl2TtfContext) -> &mut sdl2::ttf::Font<'ttf, 'static> {
        self.load_size_if_not_loaded(font_size, ttf_context);
        self.stored_sizes.get_mut(&font_size).unwrap()
    }
}

struct SDL2GraphicsContext<'ttf> {
    window_canvas : SDL2WindowCanvas,
    ttf_context : &'ttf sdl2::ttf::Sdl2TtfContext,
    font_assets : HashMap<String, SDL2FontAsset<'ttf>>,
}

// lots of interface and safety changes to be made.
impl<'ttf> SDL2GraphicsContext<'ttf> {
    fn new(window: sdl2::video::Window, ttf_context : &'ttf sdl2::ttf::Sdl2TtfContext) -> SDL2GraphicsContext<'ttf> {
        SDL2GraphicsContext {
            window_canvas: window.into_canvas().build().unwrap(),
            ttf_context,
            font_assets: HashMap::new()
        }
    }

    fn add_font<'a>(&mut self, font_name: &'a str) -> &'a str {
        self.font_assets.insert(font_name.to_owned(),
                                SDL2FontAsset::new_with_common_sizes(font_name.to_owned(), &self.ttf_context));
        font_name
    }

    fn window(&self) -> &sdl2::video::Window {
        self.window_canvas.window()
    }

    fn window_mut(&mut self) -> &mut sdl2::video::Window {
        self.window_canvas.window_mut()
    }

    fn present(&mut self) {
        self.window_canvas.present();
    }

    fn clear_color(&mut self, clear_color: Color) {
        self.window_canvas.set_draw_color(
            SDLColor::RGBA(
                clear_color.r,
                clear_color.g,
                clear_color.b,
                clear_color.a,
            )
        );
        self.window_canvas.clear();
    }

    fn get_font_asset(&self, font_id: &str) -> Option<&SDL2FontAsset<'ttf>> {
        self.font_assets.get(font_id)
    } 

    fn get_font_asset_mut(&mut self, font_id: &str) -> Option<&mut SDL2FontAsset<'ttf>> {
        self.font_assets.get_mut(font_id)
    } 

    fn find_text_asset_by_size(&mut self, font_id: &str, font_size: u16) -> Option<&sdl2::ttf::Font<'ttf, 'static>> {
        let ttf_context = self.ttf_context;
        let font_asset = self.get_font_asset_mut(font_id);

        if let Some(font_asset) = font_asset {
            Some(font_asset.get_size(font_size, ttf_context))
        } else {
            None
        }
    }

    fn find_text_asset_by_size_mut(&mut self, font_id: &str, font_size: u16) -> Option<&mut sdl2::ttf::Font<'ttf, 'static>> {
        let ttf_context = self.ttf_context;
        let font_asset = self.get_font_asset_mut(font_id);

        if let Some(font_asset) = font_asset {
            Some(font_asset.get_size_mut(font_size, ttf_context))
        } else {
            None
        }
    }

    fn text_dimensions(&mut self, font_id: &str, text: &str, font_size: u16) -> (u32, u32) {
        if let Some(font_at_size) = self.find_text_asset_by_size(font_id, font_size) {
            let (width, height) = font_at_size.size_of(text).unwrap();
            (width, height)
        } else {
            (0, 0)
        }
    }

    fn render_text(&mut self, font_id: &str, x: f32, y: f32, text: &str, font_size: u16, color: Color, style: sdl2::ttf::FontStyle) {
        let ttf_context = self.ttf_context;
        let texture_creator = self.window_canvas.texture_creator();

        match self.find_text_asset_by_size_mut(font_id, font_size) {
            Some(font) => {
                if font.get_style() != style {
                    font.set_style(style);
                }
                let mut texture = texture_creator.create_texture_from_surface(
                    &font.render(text)
                        .blended(SDLColor::RGBA(255, 255, 255, 255))
                        .expect("how did this go wrong?")
                ).expect("how did this go wrong?");

                texture.set_color_mod(color.r, color.g, color.b);
                texture.set_alpha_mod(color.a);

                let sdl2::render::TextureQuery { width, height, .. } = texture.query();
                self.window_canvas.copy(&texture, None, Some(sdl2::rect::Rect::new(x as i32, y as i32, width, height))).unwrap();
            },
            None => {}
        }
    }

    fn render_filled_rectangle(&mut self, x: f32, y: f32, w: f32, h: f32, color: Color) {
        self.window_canvas.set_draw_color(
            SDLColor::RGBA(
                color.r,
                color.g,
                color.b,
                color.a
            )
        );
        self.window_canvas.fill_rect(sdl2::rect::Rect::new(x as i32, y as i32, w as u32, h as u32));
    }
}

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

    let slideshow_source = load_file("test.slide");
    let slideshow_source = remove_comments_from_source(&slideshow_source);
    let slideshow = compile_slide(&slideshow_source);

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
        use sdl2::ttf::FontStyle;

        if let Some(current_slide) = current_slide {
            // rendering the slide
            {
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

        graphics_context.present();
    }
}
