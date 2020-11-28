/*
    beginnings of a slideshow program?

TODO: Please rewrite the tokenizer.
*/
mod markup;
use self::markup::*;
mod utility;
use self::utility::*;
mod graphics_context;
use self::graphics_context::*;
mod color;
use self::color::*;
mod slide_parser;
use self::slide_parser::*;

use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
struct TextElement {
    /*font?*/
    x: f32,
    y: f32, // TODO: Rename to linebreaks between. Or something
    text: String, // In the case I allow variables or something...
    color: Color,
    font_size: u16,
    font_name: Option<String>,
}
impl Hash for TextElement {
    fn hash<H: Hasher>(&self, state: &mut H) {
        fn canonicalize_position(e: &TextElement) -> (i32, i32) {
            (e.x.round() as i32, e.y.round() as i32)
        }

        canonicalize_position(self).hash(state);
        self.text.hash(state);
        self.color.hash(state);
        self.font_size.hash(state);
        self.font_name.hash(state);
    }
}
#[derive(Debug, Hash, Clone)]
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

impl Page {
    fn calculate_hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        let mut hasher_state = DefaultHasher::new();
        self.hash(&mut hasher_state);
        hasher_state.finish()
    }
}

impl Default for Page {
    fn default() -> Page {
        Page {
            background_color: COLOR_WHITE,
            text_elements: Vec::new()
        }
    }
}

enum SlideTransitionType {
    HorizontalSlide,
    VerticalSlide,
    FadeTo(Color),
}

struct SlideTransition {
    transition_type: SlideTransitionType, // type is keyword :(
    easing_function: EasingFunction,
    time: f32,
    finish_time: f32,
}
impl SlideTransition {
    fn finished_transition(&self) -> bool {
        self.time >= self.finish_time
    }
    fn easing_amount(&self) -> f32 {
        self.easing_function.evaluate(0.0, 1.0, self.time/self.finish_time)
    }
}
struct Slide {
    file_name : String, // owned string for hot reloading.
    last_modified_time: std::time::SystemTime,

    pages : Vec<Page>,
    current_page : isize,

    transition : Option<SlideTransition>,
}
impl Default for Slide {
    fn default() -> Slide {
        Slide {
            file_name: String::new(),
            pages: Vec::new(),
            // transition: None,
            transition: Some(
                SlideTransition {
                    transition_type: SlideTransitionType::HorizontalSlide,
                    easing_function: EasingFunction::Linear,
                    time: 0.0,
                    finish_time: 1.0
                }),
            current_page: isize::default(),
            last_modified_time: std::time::SystemTime::now(),// eh...
        }
    }
}

impl Slide {
    // handle errors more explictly...
    fn file_last_modified_time(&self) -> std::time::SystemTime {
        let metadata = std::fs::metadata(&self.file_name)
            .expect("metadata retrieval failed... Deleted file?");
        metadata.modified().expect("metadata modified time failed? No time?")
    }
    /*
    This will load and keep the current page around something with a similar hash.
    If it has the same page count it will reliably return the same page but modified.

    The hash idea isn't very great, but might be okay if I replace it with a similarity
    score system or something.
     */
    fn reload(&mut self) -> Result<(), ()> {
        let previous_page_count = self.len();
        let previous_current_page = self.current_page();

        let new_slide = Slide::new_from_file(&self.file_name);
        if let Some(slide) = new_slide {
            if slide.last_modified_time > self.last_modified_time {
                if previous_page_count == slide.len() {
                    *self = slide;
                    self.current_page = previous_current_page;
                } else {
                    let current_page_hash = self.get_current_page()
                        .expect("should have page...").calculate_hash();
                    let mut hash_delta: u64 = std::u64::MAX;
                    let mut closest_hash_and_index: (u64, usize) = (0, 0);
                    for (index, page) in slide.pages.iter().enumerate() {
                        let page_hash = page.calculate_hash();
                        if page_hash == current_page_hash {
                            closest_hash_and_index = (current_page_hash, index);
                            break;
                        } else {
                            let min = std::cmp::min(page_hash, current_page_hash);
                            let max = std::cmp::max(page_hash, current_page_hash);
                            if (max - min) < hash_delta {
                                hash_delta = max - min;
                                closest_hash_and_index = (page_hash, index);
                            }
                        }
                    }
                    *self = slide;
                    self.current_page = closest_hash_and_index.1 as isize;
                }
            }
            Ok(())
        } else {
            Err(())
        }
    }

    fn len(&self) -> usize {
        self.pages.len()
    }
    fn current_page(&self) -> isize {
        self.current_page
    }
    fn get_current_page(&self) -> Option<&Page> {
        self.get(self.current_page as usize)
    }

    fn get(&self, index: usize) -> Option<&Page> {
        self.pages.get(index)
    }

    fn next_page(&mut self) -> isize {
        let desired_next_page = self.current_page + 1;
        self.current_page += 1;
        self.current_page = clamp_i32(self.current_page as i32, 0, self.len() as i32) as isize;
        desired_next_page
    }

    fn previous_page(&mut self) -> isize {
        let desired_next_page = self.current_page - 1;
        self.current_page -= 1;
        self.current_page = clamp_i32(self.current_page as i32, 0, self.len() as i32) as isize;
        desired_next_page
    }

    fn new_from_file(file_name: &str) -> Option<Slide> {
        match load_file(file_name) {
            Ok(file_source) => {
                let slideshow_source = remove_comments_from_source(&file_source);

                let mut new_slide = Slide {
                    file_name: file_name.to_owned(),
                    pages: compile_slide_pages(&slideshow_source),
                    current_page: 0,
                    .. Slide::default()
                };
                new_slide.last_modified_time = new_slide.file_last_modified_time();
                Some(new_slide)
            },
            Err(_) => {
                None
            }
        }
    }
}
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
                        // I feel like this loop can be replaced...
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
                new_page.text_elements.push(TextElement{
                    x: 0.0,
                    y: current_line as f32,
                    text: String::from(
                        if let Some('$') = line.chars().nth(0) {
                            &line[1..]
                        } else {
                            &line
                        }
                    ),
                    font_size: context.current_font_size,
                    font_name: context.current_font_path.clone(),
                    color: context.current_element_color
                });
                current_line = 0;
            } else {
                current_line += 1;
            }
        }
    }

    new_page
}

// aux function
fn find_closing_command(line_iterator: &mut std::iter::Enumerate<std::str::Lines>, match_name: &str) -> Option<usize> {
    loop {
        let next = line_iterator.next();
        match next {
            Some((index, line)) => {
                if let Some(commands) = parse_slide_command(&line) {
                    if commands[0].name == match_name {
                        break Some(index);
                    }
                }
            },
            None => {
                break None;
            }
        }
    }
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
                    let end_page_index = find_closing_command(&mut line_iterator, "end_page");

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
                if line.len() > 0 {
                    println!("warning: Plain text should not be outside of a page!");
                }
            },
        }
    }

    slide
}

use sdl2::event::Event as SDLEvent;
use sdl2::keyboard::Keycode as SDLKeycode;

const DEFAULT_WINDOW_WIDTH : u32 = 1280;
const DEFAULT_WINDOW_HEIGHT : u32 = 720;
const DEFAULT_SLIDE_WHEN_NONE_GIVEN : &'static str = "test.slide";

#[derive(Clone, Copy)]
enum ApplicationScreen {
    InvalidOrNoSlide,
    Options,
    ShowingSlide,
    SelectSlideToLoad,
    ChangePage(isize, isize),
    Quit,
}

const DEFAULT_LAST_WRITE_TIMER_INTERVAL : f32 = 0.20;
struct ApplicationState {
    state: ApplicationScreen,

    // options state,
    currently_selected_resolution: usize,
    last_write_timer: f32,
    // everything else I guess
    slideshow: Option<Slide>,
}

impl ApplicationState {
    fn new(command_line_arguments: &Vec<String>) -> ApplicationState {
        ApplicationState {
            state: ApplicationScreen::ShowingSlide,
            currently_selected_resolution: 0,
            last_write_timer: 0.0,

            slideshow:
            Slide::new_from_file(
                match command_line_arguments.len() {
                    1 => {
                        DEFAULT_SLIDE_WHEN_NONE_GIVEN
                    }
                    2 => {
                        &command_line_arguments[1]
                    },
                    _ => {
                        println!("The only command line argument should be the slide file!");
                        DEFAULT_SLIDE_WHEN_NONE_GIVEN
                    }
                }
            )
        }
    }

    fn update(&mut self, delta_time: f32) {
        // TODO better timer and make it read based on last write/modified time...
        if let Some(slideshow) = &mut self.slideshow {
            if self.last_write_timer <= 0.0 {
                self.last_write_timer = DEFAULT_LAST_WRITE_TIMER_INTERVAL;
                slideshow.reload();
            }
        }
        self.last_write_timer -= delta_time;

        match self.state {
            ApplicationScreen::Quit | ApplicationScreen::InvalidOrNoSlide |
            ApplicationScreen::Options => {},
            ApplicationScreen::SelectSlideToLoad => {
                self.state = ApplicationScreen::ShowingSlide;
            },
            ApplicationScreen::ShowingSlide => {
                if let None = &self.slideshow {
                    self.state = ApplicationScreen::InvalidOrNoSlide;
                }
            },
            ApplicationScreen::ChangePage(first, second) => {
                if let Some(slideshow) = &mut self.slideshow {
                    let valid_transition = slideshow.get(first as usize).is_some() && slideshow.get(second as usize).is_some();
                    if let None = slideshow.transition {
                        self.state = ApplicationScreen::ShowingSlide;
                    } else if let Some(transition) = &mut slideshow.transition {
                        if valid_transition && !transition.finished_transition() {
                            transition.time += delta_time;
                        } else {
                            self.state = ApplicationScreen::ShowingSlide;
                            transition.time = 0.0;
                        }
                    }
                }
            },
        }
    }

    fn draw(&self, graphics_context: &mut SDL2GraphicsContext) {
        let default_font = graphics_context.add_font("data/fonts/libre-baskerville/LibreBaskerville-Regular.ttf");

        fn draw_slide_page(page: &Page,
                           graphics_context: &mut SDL2GraphicsContext,
                           default_font: &str) {
            graphics_context.render_filled_rectangle(0.0, 0.0,
                                                     graphics_context.logical_width() as f32,
                                                     graphics_context.logical_height() as f32,
                                                     page.background_color);

            let mut last_font_size : u16 = 0;
            let mut cursor_y : f32 = 0.0;
            graphics_context.logical_resolution = VirtualResolution::Virtual(1280, 720);

            for text in &page.text_elements {
                let font_size = text.font_size;
                let mut cursor_x : f32 = 0.0;

                let markup_lexer = MarkupLexer::new(&text.text);
                let drawn_font =
                    if let Some(font) = &text.font_name {
                        graphics_context.add_font(font)
                    } else {
                        default_font 
                    };

                let (_, height) = graphics_context.text_dimensions(drawn_font, &text.text, font_size);
                if last_font_size == 0 { last_font_size = height as u16; }
                cursor_y += last_font_size as f32 * text.y;

                for markup in markup_lexer {
                    let text_content = markup.get_text_content();
                    let width = graphics_context.logical_text_dimensions(drawn_font, text_content, font_size).0;
                    graphics_context.render_text(drawn_font,
                                                 cursor_x, cursor_y,
                                                 text_content,
                                                 font_size,
                                                 text.color,
                                                 markup.get_text_drawing_style());
                    // render decoration
                    match markup {
                        Markup::Strikethrough(_) => {
                            graphics_context.render_filled_rectangle(cursor_x,
                                                                     cursor_y + (font_size as f32 / 1.8),
                                                                     width as f32,
                                                                     font_size as f32 / 10.0,
                                                                     text.color);
                        }
                        Markup::Underlined(_) => {
                            graphics_context.render_filled_rectangle(cursor_x,
                                                                     cursor_y + (font_size as f32),
                                                                     width as f32,
                                                                     font_size as f32 / 13.0,
                                                                     text.color);
                        }
                        _ => {},
                    }
                    cursor_x += (width) as f32;
                }

                cursor_y += (height as f32);
                last_font_size = font_size;
            }
        }

        match self.state {
            ApplicationScreen::Quit | ApplicationScreen::SelectSlideToLoad => {},
            ApplicationScreen::InvalidOrNoSlide => {
                graphics_context.clear_color(Color::new(10, 10, 16, 255));
                graphics_context.logical_resolution = VirtualResolution::Display;
                let font_size = graphics_context.font_size_percent(0.073);
                let (width, height) = graphics_context.text_dimensions(default_font, "Invalid / No slide file", font_size);
                graphics_context.render_text(default_font,
                                             ((graphics_context.logical_width() as i32 / 2) - (width as i32) / 2) as f32,
                                             ((graphics_context.logical_height() as i32 / 2) - (height as i32) / 2) as f32,
                                             "Invalid / No slide file",
                                             font_size,
                                             COLOR_WHITE,
                                             sdl2::ttf::FontStyle::NORMAL);
            },
            ApplicationScreen::Options => {
                graphics_context.logical_resolution = VirtualResolution::Display;
                graphics_context.clear_color(Color::new(10, 10, 16, 255));
                let heading_font_size = graphics_context.font_size_percent(0.08);
                let (width, heading_height) = graphics_context.text_dimensions(default_font, "Resolution Select", heading_font_size);
                graphics_context.render_text(default_font,
                                             ((graphics_context.logical_width() as i32 / 2) - (width as i32) / 2) as f32,
                                             0.0,
                                             "Resolution Select",
                                             heading_font_size,
                                             COLOR_WHITE,
                                             sdl2::ttf::FontStyle::NORMAL);
                let resolutions = graphics_context.get_avaliable_resolutions();
                let resolution_count = resolutions.iter().count();
                let resolutions_to_show = 8; 

                let mut draw_cursor_y : f32 = (heading_height*2) as f32;

                for (index, resolution) in resolutions[self.currently_selected_resolution..
                                                       (self.currently_selected_resolution+resolutions_to_show)
                                                       .min(resolution_count)].iter().enumerate() {
                    let is_selected = (index == 0);
                    let resolution_string =
                        if is_selected {
                            format!("* {} x {}", resolution.0, resolution.1)
                        } else {
                            format!("{} x {}", resolution.0, resolution.1)
                        };
                    let font_size =
                        if is_selected {
                            graphics_context.font_size_percent(0.073)
                        } else {
                            graphics_context.font_size_percent(0.057)
                        };
                    let (width, height) = graphics_context.text_dimensions(default_font, &resolution_string, font_size);
                    graphics_context.render_text(default_font,
                                                 ((graphics_context.logical_width() as i32 / 2) - (width as i32) / 2) as f32,
                                                 draw_cursor_y,
                                                 &resolution_string,
                                                 font_size,
                                                 if is_selected {
                                                     COLOR_RIPE_LEMON 
                                                 } else {
                                                     COLOR_WHITE
                                                 } ,
                                                 sdl2::ttf::FontStyle::NORMAL);
                    draw_cursor_y += height as f32;
                }
            },
            ApplicationScreen::ChangePage(first, second) => {
                let slideshow = &self.slideshow.as_ref().unwrap();
                if let Some(transition) = &slideshow.transition {
                    let easing_amount = transition.easing_amount();
                    match transition.transition_type {
                        SlideTransitionType::HorizontalSlide => {
                            let forward_direction = second > first;
                            graphics_context.camera.y = 0.0;

                            graphics_context.camera.x =
                                if forward_direction {
                                    0.0 - graphics_context.logical_width() as f32 * easing_amount
                                } else {
                                    0.0 + graphics_context.logical_width() as f32 * easing_amount
                                };
                            draw_slide_page(slideshow.get(first as usize).unwrap(), graphics_context, default_font);
                            graphics_context.camera.x =
                                if forward_direction {
                                    graphics_context.logical_width() as f32 - (graphics_context.logical_width() as f32 * easing_amount)
                                } else {
                                    -(graphics_context.logical_width() as f32) + (graphics_context.logical_width() as f32 * easing_amount)
                                };
                            draw_slide_page(slideshow.get(second as usize).unwrap(), graphics_context, default_font);
                        },
                        SlideTransitionType::VerticalSlide => {
                            let forward_direction = second > first;
                            graphics_context.camera.x = 0.0;

                            graphics_context.camera.y =
                                if forward_direction {
                                    0.0 - graphics_context.logical_height() as f32 * easing_amount
                                } else {
                                    0.0 + graphics_context.logical_height() as f32 * easing_amount
                                };
                            draw_slide_page(slideshow.get(first as usize).unwrap(), graphics_context, default_font);
                            graphics_context.camera.y =
                                if forward_direction {
                                    graphics_context.logical_height() as f32 - (graphics_context.logical_height() as f32 * easing_amount)
                                } else {
                                    -(graphics_context.logical_height() as f32) + (graphics_context.logical_height() as f32 * easing_amount)
                                };
                            draw_slide_page(slideshow.get(second as usize).unwrap(), graphics_context, default_font);
                        },
                        SlideTransitionType::FadeTo(Color) => {
                            // split time into two halves.
                        },
                    }
                }
            },
            ApplicationScreen::ShowingSlide => {
                if let Some(slideshow) = &self.slideshow {
                    graphics_context.camera.x = 0.0;
                    graphics_context.camera.y = 0.0;
                    if let Some(current_slide) = slideshow.get_current_page() {
                        draw_slide_page(current_slide, graphics_context, default_font);
                    } else {
                        graphics_context.clear_color(Color::new(10, 10, 16, 255));
                        graphics_context.logical_resolution = VirtualResolution::Display;
                        let font_size = graphics_context.font_size_percent(0.073);
                        let (width, height) = graphics_context.text_dimensions(default_font, "stupid slide needs pages... feed me", font_size);
                        graphics_context.render_text(default_font,
                                                     ((graphics_context.logical_width() as i32 / 2) - (width as i32) / 2) as f32,
                                                     ((graphics_context.logical_height() as i32 / 2) - (height as i32) / 2) as f32,
                                                     "stupid slide needs pages... feed me",
                                                     font_size,
                                                     COLOR_WHITE,
                                                     sdl2::ttf::FontStyle::NORMAL);
                    }
                }
            },
        }
    }

    fn handle_input(&mut self, graphics_context: &mut SDL2GraphicsContext, event_pump: &mut sdl2::EventPump, delta_time: f32) {
        match self.state {
            ApplicationScreen::Quit => {},
            ApplicationScreen::SelectSlideToLoad => {},
            ApplicationScreen::InvalidOrNoSlide => {
                for event in event_pump.poll_iter() {
                    match event {
                        SDLEvent::Quit {..} => {
                            self.state = ApplicationScreen::Quit;
                        },
                        SDLEvent::KeyDown { keycode: Some(SDLKeycode::Escape), .. } =>  {
                            self.state = ApplicationScreen::Quit;
                        },
                        SDLEvent::KeyDown { keycode: Some(SDLKeycode::O), .. } => {
                            self.state = ApplicationScreen::Options;
                        },
                        SDLEvent::KeyDown { keycode: Some(SDLKeycode::L), .. } => {
                            self.state = ApplicationScreen::SelectSlideToLoad;
                        },
                        _ => {}
                    }
                }
            },
            ApplicationScreen::Options => {
                let resolutions = graphics_context.get_avaliable_resolutions();
                let resolution_count = resolutions.iter().count();
                self.currently_selected_resolution = self.currently_selected_resolution.max(0);
                self.currently_selected_resolution = self.currently_selected_resolution.min(resolution_count-1);
                for event in event_pump.poll_iter() {
                    match event {
                        SDLEvent::Quit {..} => {
                            self.state = ApplicationScreen::Quit;
                        },
                        SDLEvent::KeyDown { keycode: Some(SDLKeycode::F), ..} => {
                            graphics_context.toggle_fullscreen();
                        },
                        SDLEvent::KeyDown { keycode: Some(SDLKeycode::Return), .. } =>  {
                            let resolution_list = graphics_context.get_avaliable_resolutions();
                            if let Some(resolution_pair) = resolution_list.get(self.currently_selected_resolution) {
                                graphics_context.set_resolution((resolution_pair.0 as u32, resolution_pair.1 as u32));
                            }
                        },
                        SDLEvent::KeyDown { keycode: Some(SDLKeycode::Up), .. } => {
                            if self.currently_selected_resolution > 0 {
                                self.currently_selected_resolution -= 1;
                            }
                        },
                        SDLEvent::KeyDown { keycode: Some(SDLKeycode::Down), .. } => {
                            self.currently_selected_resolution += 1;
                        },
                        SDLEvent::KeyDown { keycode: Some(SDLKeycode::O), .. } => {
                            self.state = ApplicationScreen::ShowingSlide;
                        },
                        _ => {}
                    }
                }
            },
            ApplicationScreen::ChangePage(_, _) => {
                for event in event_pump.poll_iter() {
                    match event {
                        SDLEvent::Quit {..} => {
                            self.state = ApplicationScreen::Quit;
                        },
                        SDLEvent::KeyDown {..} => {
                            self.state = ApplicationScreen::ShowingSlide;
                            if let Some(slideshow) = &mut self.slideshow {
                                if let Some(transition) = &mut slideshow.transition {
                                    transition.time = 0.0;
                                }
                            }
                        },
                        _ => {}
                    }
                }
            }
            ApplicationScreen::ShowingSlide => {
                for event in event_pump.poll_iter() {
                    match event {
                        SDLEvent::Quit {..} => {
                            self.state = ApplicationScreen::Quit;
                        },
                        SDLEvent::KeyDown { keycode: Some(SDLKeycode::Escape), .. } =>  {
                            self.slideshow = None;
                            self.state = ApplicationScreen::InvalidOrNoSlide;
                        },
                        SDLEvent::KeyDown { keycode: Some(SDLKeycode::F), ..} => {
                            graphics_context.toggle_fullscreen();
                        },
                        #[cfg(debug_assertions)]
                        SDLEvent::KeyDown { keycode: Some(SDLKeycode::Num0), .. } => {
                            graphics_context.camera.scale = 1.0;
                        },
                        #[cfg(debug_assertions)]
                        SDLEvent::KeyDown { keycode: Some(SDLKeycode::Up), .. } => {
                            graphics_context.camera.scale += 1.0 * delta_time;
                        },
                        #[cfg(debug_assertions)]
                        SDLEvent::KeyDown { keycode: Some(SDLKeycode::Down), .. } => {
                            graphics_context.camera.scale -= 1.0 * delta_time;
                        },
                        SDLEvent::KeyDown { keycode: Some(SDLKeycode::R), .. } => {
                            if let Some(slideshow) = &mut self.slideshow {
                                slideshow.reload();
                            }
                        },
                        SDLEvent::KeyDown { keycode: Some(SDLKeycode::Right), .. } => {
                            if let Some(slideshow) = &mut self.slideshow {
                                self.state = ApplicationScreen::ChangePage(slideshow.current_page(),
                                                                           slideshow.next_page());
                            }
                        },
                        SDLEvent::KeyDown { keycode: Some(SDLKeycode::Left), .. } => {
                            if let Some(slideshow) = &mut self.slideshow {
                                self.state = ApplicationScreen::ChangePage(slideshow.current_page(),
                                                                           slideshow.previous_page());
                            }
                        },
                        SDLEvent::KeyDown { keycode: Some(SDLKeycode::O), .. } => {
                            self.state = ApplicationScreen::Options;
                        },
                        _ => {}
                    }
                }
            },
        }
    }
}

fn main() {
    let sdl2_context = sdl2::init().expect("SDL2 failed to initialize?");
    let video_subsystem = sdl2_context.video().unwrap();

    let sdl2_ttf_context = sdl2::ttf::init()
        .expect("SDL2 ttf failed to initialize?");
    let sdl2_image_context = sdl2::image::init(sdl2::image::InitFlag::PNG | sdl2::image::InitFlag::JPG)
        .expect("SDL2 image failed to initialize?");

    let window = video_subsystem.window("stupid slideshow", DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT)
        .position_centered()
        .build()
        .expect("Window failed to open?");

    let mut graphics_context = SDL2GraphicsContext::new(window,
                                                        &sdl2_ttf_context,
                                                        &sdl2_image_context,
                                                        &video_subsystem);
    let default_font = graphics_context.add_font("data/fonts/libre-baskerville/LibreBaskerville-Regular.ttf");
    // let dumb_test_texture = graphics_context.add_image("data/res/rust-logo-png-transparent.png");
    let resolutions = graphics_context.get_avaliable_resolutions();

    let mut event_pump = sdl2_context.event_pump().unwrap();

    use std::env;
    let arguments : Vec<String> = env::args().collect();
    let mut application_state = ApplicationState::new(&arguments);

    let mut sdl2_timer = sdl2_context.timer().unwrap();
    let mut delta_time = 0;
    'running: loop {
        let start_time = sdl2_timer.ticks();
        if let ApplicationScreen::Quit = application_state.state {
            break 'running;
        } else {
            let delta_time = delta_time as f32 / 1000.0;
            application_state.handle_input(&mut graphics_context, &mut event_pump, delta_time);
            application_state.update(delta_time);
            application_state.draw(&mut graphics_context);

            graphics_context.present();
        }
        let end_time = sdl2_timer.ticks();
        delta_time = end_time - start_time;
    }
}
