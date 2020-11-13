/*
    beginnings of a slideshow program?

TODO: Please rewrite the tokenizer.
*/
mod markup;
use self::markup::*;

#[derive(Debug)]
struct TextElement {
    /*font?*/
    x: f32,
    y: f32,
    text: String, // In the case I allow variables or something...
}
#[derive(Debug)]
struct Page {
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
            text_elements: Vec::new()
        }
    }
}

type Slide = Vec<Page>;
// statemachine thing?
struct SlideSettingsContext {
}

impl Default for SlideSettingsContext {
    fn default() -> SlideSettingsContext {
        SlideSettingsContext{}
    }
}

#[derive(Debug)]
// tokenized commands.
struct SlideLineCommand <'a> {
    name: &'a str,
    args: Vec<&'a str>,
}

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
#[derive(Debug)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}
impl Color {
    fn new(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color {r, g, b, a}
    }

    // TODO
    fn parse_hexadecimal_literal(hex: &str) -> Color {
        Color::new(0, 0, 0, 0)
    }
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
                Some(if command.name == "color" { Command::SetColor(color) }
                     else { Command::SetBackgroundColor(color) })
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
    let mut line_breaks : u32 = 0; // used for calculating relative y.
    let mut new_page : Page = Page::default();

    println!("lines: {:?}", page_lines);
    for line in page_lines {
        if let Some(commands) = parse_slide_command(&line) {
            for command in commands {
                handle_command(context, command);
            }
        } else {
            if line.len() >= 1 {
                println!("plain text: {}", line);
                new_page.text_elements.push(TextElement{x: 0.0, y: 0.0, text: String::from(line)});
            } else {
                println!("line break!");
                line_breaks += 1;
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

fn render_page(page: &Page) {
    for text in &page.text_elements {
        let mut cursor_x : f32 = text.x;
        let mut cursor_y : f32 = text.y;

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

fn load_file(file_name: &str) -> String {
    use std::io::Read;
    use std::fs::File;

    let mut result = String::new();
    let mut slide_file = File::open(file_name)
        .expect("There was an error in reading the file!");
    slide_file.read_to_string(&mut result).expect("Unable to read into string");
    result
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
    if false {
        let slideshow_source = load_file("test.slide");
        let slideshow_source = remove_comments_from_source(&slideshow_source);
        let slideshow = compile_slide(&slideshow_source);
        println!("SLIDE:\n{:#?}", slideshow);
        for page in slideshow {
            render_page(&page);
        }
    }
    // render_present_slide(&slideshow);
}
