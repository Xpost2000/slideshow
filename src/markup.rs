/*
I know there's a way to do it with just &str and slices, but
the iteration work! OMG it's so much.
*/
#[derive(Debug)]
pub enum Markup {
    Plain(String),
    Bold(String),
    Strikethrough(String),
    Italics(String),
    Underlined(String),
}

// I had to lookup a basic lexer in Rust... Cause holy s**t whatever I was
// doing was really confusing.
pub struct MarkupLexer<'a> {
    iterator: std::iter::Peekable<std::str::Chars<'a>>,
}

pub fn is_whitespace(c: char) -> bool {
    c == ' ' || c == '\n' || c == '\t' || c == '\r'
}

/*
   Real markup has multicharacter patterns, which
   should still be fairly easy to adopt here...
*/
impl<'a> MarkupLexer<'a> {
    // this is self consuming since the iterator
    // will be used up. Probably debugging stuff.
    pub fn stitch(self) -> String {
        let mut result = String::new();
        for item in self {
            match item {
                Markup::Plain(text_content) |
                Markup::Bold(text_content) |
                Markup::Strikethrough(text_content) |
                Markup::Italics(text_content) |
                Markup::Underlined(text_content) => {
                    result.push_str(&text_content);
                }
            }
        }
        result
    }

    fn is_special_character(c: char) -> bool {
        match c {
            '*' | '/' | '_' | '+' => true,
            _ => false,
        }
    }

    pub fn new(source: &'a str) -> MarkupLexer<'a> {
        MarkupLexer {
            iterator: source.chars().peekable()
        }
    }

    fn peek_character(&mut self) -> Option<&char> {
        self.iterator.peek()
    }

    fn next_character(&mut self) -> Option<char> {
        self.iterator.next()
    }

    fn next_words_until_special(&mut self) -> String {
        let mut sentence : String = String::new();
        let mut previous_character : Option<char> = None;

        while let Some(&character) = self.peek_character() {
            if MarkupLexer::is_special_character(character) {
                if let Some(&next_character) = self.peek_character() {
                    if let Some(previous_character) = previous_character {
                        if !is_whitespace(next_character) && is_whitespace(previous_character) {
                            return sentence;
                        }
                    }
                }
            }
            sentence.push(character);
            previous_character = Some(character);
            self.next_character().unwrap();
        }
        sentence
    }

    fn find_type(identifier: char, text_contents: String) -> Markup {
        match identifier {
            '*' => Markup::Bold(text_contents),
            '+' => Markup::Strikethrough(text_contents),
            '/' => Markup::Italics(text_contents),
            '_' => Markup::Underlined(text_contents),
            _ => Markup::Plain(text_contents),
        }
    }

    fn find_match_and_pass(&mut self, to_match: char) -> (String, bool) {
        let mut sentence : String = String::new();
        let mut previous_character : Option<char> = None;

        while let Some(&character) = self.peek_character() {
            if character == to_match {
                let good_match = 
                    if let Some(previous_character) = previous_character {
                        if !is_whitespace(previous_character) {true} else {false}
                    } else {
                        false
                    };
                self.next_character();
                return (sentence, good_match);
            }
            sentence.push(character);
            previous_character = Some(character);
            self.next_character().unwrap();
        }
        (sentence, false)
    }

    fn next_markup_item(&mut self) -> Option<Markup> {
        fn string_prepend(input: &String, c: char) -> String {
            let mut result = String::new();
            result.push(c);
            result.push_str(input);
            result
        }

        if let Some(&character) = self.peek_character() {
            if MarkupLexer::is_special_character(character) {
                self.next_character();
                if let (text_within_boundaries, true) = self.find_match_and_pass(character) {
                    Some(MarkupLexer::find_type(character, text_within_boundaries))
                } else {
                    Some(Markup::Plain(string_prepend(&self.next_words_until_special(), character)))
                }
            } else {
                Some(Markup::Plain(self.next_words_until_special()))
            }
        } else {
            None
        }
    }
}

impl<'a> Iterator for MarkupLexer<'a> {
    type Item = Markup;
    fn next(&mut self) -> Option<Self::Item> {
        self.next_markup_item()
    }
}
