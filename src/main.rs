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
        if let Some(markup) = MarkupRange::try_parse_region(character, index+1, &mut character_iterator) {
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

fn parse_command(line : &str) -> Option<Command> {
    None
}

fn execute_command(command : Command) {
    match command {
        _ => {}
    }
}

struct Page {
    /*
    A Page is probably just going to consist of "elements"
    
    Like a TextElement
    and ImageElement
    and ShapeElement
    or whatever primitives I want to add...
     */
}

fn compile_slide(slide_source : &String) -> Vec<Page> {
    let mut line_breaks : u32 = 0;
    fn is_slide_command(line : &str) -> bool {
        if line.chars().nth(0) == Some('$') {
            if line.chars().nth(1) == Some('$') {
                false
            } else if line.chars().nth(1) == None {
                false
            } else {
                true
            }
        } else {
            false
        }
    }

    // The compiler "state" is "global" but I should probably reject
    // any text that isn't inside a currently compiled page...
    // text outside of slides should be a warning though and should probably
    // just be treated like a comment.
    for line in slide_source.lines() {
        if is_slide_command(&line) {
            println!("{} : COMMAND!", line);
            if let Some(command) = parse_command(&line[1..]) {
                execute_command(command);
            } else {
                println!("Warning: \"{}\" could not be interpreted as a real command", &line[1..]);
            }
        } else {
            if line.chars().count() > 0 {
                println!("{} : normal text : {} line breaks", line, line_breaks);
                line_breaks = 0;
            } else {
                println!("linebreak");
                line_breaks += 1;
            }
        }
    }

    unimplemented!("not finished");
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

fn main() {
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
    let test_page = compile_slide(&test_string);
    render_present_slide(&test_slide);
}
