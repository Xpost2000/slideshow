use crate::color::Color;
use crate::color::COLOR_WHITE;
use crate::color::COLOR_BLACK;
use crate::slide::*;
use crate::utility::*;

pub struct SlideSettingsContext {
    pub current_line: u32,
    pub current_x: Option<f32>,
    pub current_y: Option<f32>,

    pub current_background_color: Color,
    pub current_element_color: Color,
    pub current_font_size: u16,
    pub current_font_path: Option<String>,
}

impl Default for SlideSettingsContext {
    fn default() -> SlideSettingsContext {
        SlideSettingsContext{
            current_line: 0,
            current_x: None,
            current_y: None,
            current_background_color: COLOR_WHITE,
            current_element_color: COLOR_BLACK,
            current_font_size: 48,
            current_font_path: None,
        }
    }
}

impl SlideSettingsContext {
    fn y(&self) -> Option<f32> {
        self.current_y
    }

    fn x(&self) -> Option<f32> {
        self.current_x
    }

    fn set_position(&mut self, x: Option<f32>, y: Option<f32>) {
        self.current_x = x;
        self.current_y = y;
        self.current_line = 0;
    }
}

#[derive(Debug, Clone)]
// tokenized commands.
pub struct SlideLineCommand <'a> {
    pub name: &'a str,
    pub args: Vec<&'a str>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Command <'a> {
    Reset, // Total reset
    ResetPosition,
    ResetFont, // TODO think of better thing.
    SetFont(&'a str),
    SetBackgroundColor(Color),
    SetColor(Color),
    SetFontSize(u16),
    SetVirtualResolution(u32, u32),
    SetTransition(SlideTransition),
    SetPosition(Option<f32>, Option<f32>),
    InsertImage(bool, &'a str, Option<f32>, Option<f32>),
}

// TODO!
pub fn execute_command(context: &mut SlideSettingsContext, command: Command) {
    match command {
        Command::SetColor(color) => {context.current_element_color = color;},
        Command::SetBackgroundColor(color) => {context.current_background_color = color;},
        Command::SetFontSize(font_size) => {context.current_font_size = font_size;}
        // the compiled slide should not depend on the source...
        Command::SetFont(font_name) => {context.current_font_path = Some(font_name.to_owned());},
        Command::ResetPosition => {context.set_position(None, None);}
        Command::ResetFont => {context.current_font_path = None;},
        Command::SetPosition(x, y) => {context.set_position(x,y);},
        _ => { println!("{:?} is an unknown command or not handled here", command); }
    }
}

pub fn execute_command_on_page(context: &mut SlideSettingsContext, command: Command, page: &mut Page) {
    match command {
        Command::InsertImage(background, path, width, height) => {
            page.elements.push(
                SlideElement::Image(
                    ImageElement {
                        location: path.to_owned(),
                        x: context.x(),
                        y: context.y(),
                        background,
                        w: width,
                        h: height,
                        color: context.current_element_color,
                    }
                )
            );
        },
        _ => { execute_command(context, command); }
    }
}

// This will call parse command and execute command
pub fn handle_command(context: &mut SlideSettingsContext, command: SlideLineCommand) {
    let command = parse_single_command(command);
    if let Some(command) = command {
        execute_command(context, command);
    }
}

pub fn handle_command_with_page(context: &mut SlideSettingsContext,
                                command: SlideLineCommand,
                                page: &mut Page) {
    let command = parse_single_command(command);
    if let Some(command) = command {
        execute_command_on_page(context, command, page);
    }
}

pub fn parse_page(context: &mut SlideSettingsContext, page_lines: Vec<&str>) -> Page {
    let mut new_page : Page = Page::default();
    context.current_line = 0;
    let mut current_line = 0;

    for line in page_lines {
        if let Some(commands) = parse_slide_command(&line) {
            match commands[0].name {
                "transition" => {
                    let cmd = parse_single_command(commands[0].clone());
                    if let Command::SetTransition(transition) = cmd.unwrap() {
                        new_page.transition = Some(transition);
                    }
                },
                _ => {
                    for command in commands {
                        handle_command_with_page(context, command, &mut new_page);
                        new_page.background_color = context.current_background_color;
                    }
                }
            }
        } else {
            const REPLACE_TABS_WITH_N_SPACES : &'static str = "    ";
            if line.len() >= 1 {
                new_page.elements.push(
                    SlideElement::Text(
                        TextElement{
                            x: context.x(),
                            y: context.y(),

                            // line_breaks: context.current_line,
                            line_breaks: current_line,
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
                        }));
                context.current_line = 0;
                current_line = 0;
            } else {
                context.current_line += 1;
                current_line += 1;
            }
        }
    }

    context.set_position(None, None);
    new_page
}

/*
    I need to write a proper tokenizer. Instead of this
FIXME
*/
// Tokenizes a command into a real command.
// TODO!
pub fn parse_single_command<'a>(command: SlideLineCommand<'a>) -> Option<Command<'a>> {
    use std::convert::TryFrom;
    let mut args = command.args.iter();

    match command.name {
        "reset-position" => {
            Some(Command::ResetPosition)
        },
        "image" | "bkimage" => {
            if let Some(image_resource_path) = args.next() {
                let is_non_interfering = match command.name {
                    "image" => false,
                    "bkimage" => true,
                    _ => false
                };

                let width = {
                    if let Some(width_string) = args.next() {
                        match *width_string {
                            _ =>
                                if let Ok(result) = width_string.parse::<f32>() {
                                    Some(result)
                                } else {
                                    None
                                },
                        }
                    } else {
                        None
                    }
                };

                let height = {
                    if let Some(height_string) = args.next() {
                        match *height_string {
                            _ =>
                                if let Ok(result) = height_string.parse::<f32>() {
                                    Some(result)
                                } else {
                                    None
                                },
                        }
                    } else {
                        None
                    }
                };

                Some(Command::InsertImage(is_non_interfering,
                                          image_resource_path,
                                          width,
                                          height))
            } else {
                None
            }
        },
        "set-position" => {
            let x = {
                let x_string = args.next().unwrap_or(&"current");
                match *x_string {
                    "current" => None,
                    _ => Some(x_string.parse::<f32>().unwrap_or(0.0))
                }
            };

            let y = {
                let y_string = args.next().unwrap_or(&"current");
                match *y_string {
                    "current" => None ,
                    _ => Some(y_string.parse::<f32>().unwrap_or(0.0))
                }
            };

            Some(Command::SetPosition(x, y))
        },
        "color" | "background_color" => {
            if let Some(next) = &args.next() {
                let color = Color::try_from(**next).unwrap_or(COLOR_BLACK);
                Some(if command.name == "color" { Command::SetColor(color) }
                     else { Command::SetBackgroundColor(color) })
            } else {
                None
            }
        },
        "font" => {
            if let Some(next) = &args.next() {
                Some(Command::SetFont(next))
            } else {
                Some(Command::ResetFont)
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
            let width =
                args.next()
                .unwrap_or(&"1280")
                .parse::<u32>().unwrap_or(1280);
            let height =
                args.next()
                .unwrap_or(&"720")
                .parse::<u32>().unwrap_or(720);
            Some(Command::SetVirtualResolution(width, height))
        },
        "transition" => {
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
                        SlideTransitionType::FadeTo(Color::try_from(*color)
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
                                    for _ in 0..end { char_iterator.next(); }
                                }
                            },
                            ':' => {tokenized_first_pass.push(":");}
                            _ => {}
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
                    commands.push(SlideLineCommand{ name, args });
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
