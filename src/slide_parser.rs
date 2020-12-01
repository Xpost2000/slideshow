use crate::color::Color;
use crate::color::COLOR_WHITE;
use crate::color::COLOR_BLACK;
use crate::slide::*;
use crate::utility::*;

pub struct SlideSettingsContext {
    pub current_background_color: Color,
    pub current_element_color: Color,
    pub current_font_size: u16,
    pub current_font_path: Option<String>,
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

#[derive(Debug, Clone)]
// tokenized commands.
pub struct SlideLineCommand <'a> {
    pub name: &'a str,
    pub args: Vec<&'a str>,
}

#[derive(Debug)]
pub enum Command <'a> {
    Reset, // Total reset
    ResetFont, // TODO think of better thing.
    SetFont(&'a str),
    SetBackgroundColor(Color),
    SetColor(Color),
    SetFontSize(u16),
    SetVirtualResolution(u32, u32),
    SetTransition(SlideTransition),
}

// TODO!
pub fn execute_command(context: &mut SlideSettingsContext, command: Command) {
    match command {
        Command::SetColor(color) => {context.current_element_color = color;},
        Command::SetBackgroundColor(color) => {context.current_background_color = color;},
        Command::SetFontSize(font_size) => {context.current_font_size = font_size;}
        // the compiled slide should not depend on the source...
        Command::SetFont(font_name) => {context.current_font_path = Some(font_name.to_owned());},
        Command::ResetFont => {context.current_font_path = None;},
        _ => { println!("{:?} is an unknown command or not handled here", command); }
    }
}

// This will call parse command and execute command
pub fn handle_command(context: &mut SlideSettingsContext, command: SlideLineCommand) {
    let command = parse_single_command(command);
    if let Some(command) = command {
        execute_command(context, command);
    }
}

pub fn parse_page(context: &mut SlideSettingsContext, page_lines: Vec<&str>) -> Page {
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
            const REPLACE_TABS_WITH_N_SPACES : &'static str = "    ";
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
                    ).replace('\t', REPLACE_TABS_WITH_N_SPACES),
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

/*
    I need to write a proper tokenizer. Instead of this
FIXME
*/
// Tokenizes a command into a real command.
// TODO!
pub fn parse_single_command<'a>(command: SlideLineCommand<'a>) -> Option<Command<'a>> {
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
        },
        "resolution" => {
            let width = if let Some(next) = &args.next() {
                match next.parse::<u32>() {Ok(value) => value, Err(_) => 1280,}
            } else { 1280 };
            let height = if let Some(next) = &args.next() {
                match next.parse::<u32>() {Ok(value) => value, Err(_) => 720,}
            } else { 720 };
            Some(Command::SetVirtualResolution(width, height))
        },
        "transition" => {
            // $transition:fade_to:#000000:cubic_ease_in:1.25
            // I don't actually do any detailed checking...
            // that's a TODO, but that means a lot of this will be rewritten, probably.
            let type_string_of_transition = args
                .next()
                .unwrap_or(&"horizontal_slide");
            let type_of_transition =
                match *type_string_of_transition {
                    "horizontal" | "horizontal_slide" => SlideTransitionType::HorizontalSlide,
                    "vertical" | "vertical_slide" => SlideTransitionType::VerticalSlide,
                    "fade" | "color_fade" | "fade_to" => {
                        let color = args.next().unwrap_or(&"#000000FF");
                        SlideTransitionType::FadeTo(Color::parse_hexadecimal_literal(*color)
                                                    .unwrap_or(COLOR_BLACK))
                    },
                    _ => { SlideTransitionType::HorizontalSlide },
                };
            let easing_function_name = args.next().unwrap_or(&"linear");
            let easing_function_type =
                match *easing_function_name {
                    "linear" => EasingFunction::Linear,
                    "quadratic_ease_in" => EasingFunction::QuadraticEaseIn,
                    "quadratic_ease_out" => EasingFunction::QuadraticEaseOut,
                    "cubic_ease_in" => EasingFunction::CubicEaseIn,
                    "cubic_ease_out" => EasingFunction::CubicEaseOut,
                    _ => { EasingFunction::Linear },
                };
            let time_duration = args.next()
                .unwrap_or(&"1.0")
                .parse::<f32>()
                .unwrap_or(1.0);
            Some(Command::SetTransition(
                SlideTransition {
                    transition_type: type_of_transition,
                    easing_function: easing_function_type,
                    time: 0.0,
                    finish_time: time_duration,
                }
            ))
        },
        _ => { None },
    }
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

pub fn parse_slide_command(line : &str) -> Option<Vec<SlideLineCommand>> {
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
                                    // char_iterator.skip(end);
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

pub fn compile_slide(slide_source : &String) -> Slide {
    let mut slide = Slide::default();
    let mut pages = Vec::new();
    let mut current_context = SlideSettingsContext::default();
    let mut line_iterator = slide_source.lines().enumerate();

    while let Some((index, line)) = line_iterator.next() {
        match parse_slide_command(&line) {
            Some(commands) => {
                match commands[0].name {
                    "page" => {
                        let end_page_index = find_closing_command(&mut line_iterator, "end_page");

                        if let Some(end_page_index) = end_page_index {
                            let index = index+1;
                            let end_page_index = end_page_index;
                            let page_source_lines : Vec<&str> = slide_source.lines().collect();
                            let new_page = parse_page(&mut current_context, page_source_lines[index..end_page_index].to_vec());
                            pages.push(new_page);
                        } else {
                            println!("Error! EOF before an end page!");
                        }
                    },
                    "resolution" => {
                        let cmd = parse_single_command(commands[0].clone());
                        if let Command::SetVirtualResolution(w, h) = cmd.unwrap() {
                            slide.resolution = (w, h);
                        }
                    },
                    "transition" => {
                        let cmd = parse_single_command(commands[0].clone());
                        if let Command::SetTransition(transition) = cmd.unwrap() {
                            slide.transition = Some(transition);
                        }
                    },
                    _ => {
                        for command in commands {
                            handle_command(&mut current_context, command);
                        }
                    },
                }
            },
            None => {
                if line.len() > 0 {
                    println!("warning: Plain text should not be outside of a page!");
                }
            },
        }
    }

    slide.pages = pages;
    slide
}
