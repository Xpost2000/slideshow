/*
    beginnings of a slideshow program?
*/
#[derive(Debug)]
enum MarkupRange {
    Bold(usize, usize),
    Strikethrough(usize, usize),
    Italics(usize, usize),
    Underlined(usize, usize),
}
impl MarkupRange {
    // Should be Result...
    // I probably shouldn't actually error out since this is likely just a warning.
    fn try_parse_region(identifier: char, begin: usize, character_iterator: &mut std::iter::Enumerate<std::str::Chars>) -> Option<MarkupRange> {
        let end : Result<Option<usize>, &'static str> = match identifier {
            '*'|'_'|'+'|'/' => loop {
                if let Some(item) = character_iterator.next() {
                    if item.1 == identifier { break Ok(Some(item.0)); }
                } else {
                    break Err("EOF, could not find matching character!");
                }
            } ,
            _ => Ok(None),
        };
        if let Ok(end) = end {
            if let Some(end) = end {
                match identifier {
                    '*' => Some(MarkupRange::Bold(begin, end)),
                    '+' => Some(MarkupRange::Strikethrough(begin, end)),
                    '/' => Some(MarkupRange::Italics(begin, end)),
                    '_' => Some(MarkupRange::Underlined(begin, end)),
                    _ => None
                }
            } else {
                None
            }
        } else {
            if let Err(error_message) = end {
                println!("warning: {}", error_message);
            }
            None
        }
    }
}
type MarkupInformation = Vec<MarkupRange>;
fn get_markup_info(input: &str) -> MarkupInformation {
    let mut info = MarkupInformation::new();

    let mut character_iterator = input.chars().enumerate();

    while let Some((index, character)) = character_iterator.next() {
        if let Some(markup) = MarkupRange::try_parse_region(character,
                                                            index+1,
                                                            &mut character_iterator) {
            info.push(markup);
        }
    }

    info
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
enum Command {
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
}

type Slide = Vec<Page>;
struct SlideSettingsContext {
    // statemachine thing?
}

impl Default for SlideSettingsContext {
    fn default() -> SlideSettingsContext {
        SlideSettingsContext{}
    }
}

impl SlideSettingsContext {
    
}

#[derive(Debug)]
enum SlideLineCommand <'a> {
    Command(&'a str),
    CompoundCommand(Vec<&'a str>),
}

// Tokenizes a command into a real command.
fn parse_single_command(line : &str) -> Option<Command> {
    None
}

fn execute_command(context: &mut SlideSettingsContext, command: Command) {
    match command {
        _ => {}
    }
}

// This will call parse command and execute command
fn handle_command(context: &mut SlideSettingsContext, command: SlideLineCommand) {
    unimplemented!("Not done");
}

fn parse_page(context: &mut SlideSettingsContext, page_lines: Vec<&str>) -> Page {
    let mut line_breaks : u32 = 0;
    unimplemented!("Not done");
}

fn compile_slide(slide_source : &String) -> Slide {
    let mut slide = Slide::new();
    let mut current_context = SlideSettingsContext::default();
    // I need a better way to get errors...
    fn parse_slide_command(line : &str) -> Option<SlideLineCommand> {
        let mut split_line : Vec<&str> = Vec::new();
        if line.chars().nth(0) == Some('$') {
            let next_character = line.chars().nth(1);
            if next_character == Some('$') {
                // I don't properly handle escaping $$... Whoops!
                None
            } else if next_character == None {
                None
            } else {
                if next_character == Some('(') {
                    let last_index = line.chars().count()-1;
                    let last_character = line.chars().nth(last_index);
                    if last_character == Some(')') {
                        split_line = line[2..last_index].split(" ").collect();
                    } else {
                        println!("Compound command not ended?");
                    }
                } else {
                    split_line = line[1..].split(" ").collect();
                }

                if split_line.len() > 1 {
                    Some(SlideLineCommand::CompoundCommand(split_line))
                } else if split_line.len() == 1 {
                    Some(SlideLineCommand::Command(split_line[0]))
                } else {
                    None
                }
            }
        } else {
            None
        }
    }

    // The compiler "state" is "global" but I should probably reject
    // any text that isn't inside a currently compiled page...
    // text outside of slides should be a warning though and should probably
    // just be treated like a comment.
    let mut line_iterator = slide_source.lines().enumerate();
    while let Some((index, line)) = line_iterator.next() {
        match parse_slide_command(&line) {
            Some(command) => {
                if let SlideLineCommand::Command("page") = command {
                    println!("Page directive!");
                    let end_page_index = loop {
                        let next = line_iterator.next();
                        match next {
                            Some((index, line)) => {
                                if let Some(command) = parse_slide_command(&line) {
                                    if let SlideLineCommand::Command("end_page") = command {
                                        println!("Found end page!");
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
                        let page_source_lines : Vec<&str> = slide_source.lines().enumerate().map(|pair| pair.1).collect();
                        let new_page = parse_page(&mut current_context, page_source_lines);
                        println!("{:#?}", new_page);
                        slide.push(new_page);
                    } else {
                        println!("Error! EOF before an end page!");
                    }
                } else {
                    handle_command(&mut current_context, command);
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

    // unimplemented!("not finished");
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

fn main() {
    if false {
        let test_string = "*bold* normal text +test crossed out+ this text is normal /this is italicized/";
        let markup_info = get_markup_info(&test_string);
        println!("{:?}", markup_info);
        for region in markup_info.iter() {
            match region {
                MarkupRange::Bold(begin, end) =>
                    println!("Bolded: {}", &test_string[*begin..*end]),
                MarkupRange::Strikethrough(begin, end) =>
                    println!("Strikethrough: {}", &test_string[*begin..*end]),
                MarkupRange::Italics(begin, end) =>
                    println!("Italics: {}", &test_string[*begin..*end]),
                MarkupRange::Underlined(begin, end) =>
                    println!("Underlined: {}", &test_string[*begin..*end]),
            }
        }
        println!("Trying to filter strings");
        let test_string = "# Comment will be not parsed!\n$page\n#Yes? Cool!\n$background_color #FF0000FF (test:\"abc\")\nThis is some text with attributes applied\n\nThere should be one line break\nNo line break!";
        let test_string = remove_comments_from_source(test_string);
        // I know it's redundant, but remove_comments_from_source should not turn into lines...
        // It just happened that reducing it to lines first was way easier.
        for line in test_string.lines() {
            println!("{}", line);
        }
    }

    let slideshow_source = load_file("test.slide");
    let slideshow_source = remove_comments_from_source(&slideshow_source);
    let slideshow = compile_slide(&slideshow_source);
    // render_present_slide(&slideshow);
}
