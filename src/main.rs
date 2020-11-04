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
                println!("error: {}", error_message);
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
}
