use crate::communication::Message;
use crate::Key;
use nom::{
    branch::alt,
    character::complete::{anychar, char},
    combinator::{all_consuming, map, value},
    multi::many1,
    sequence::separated_pair,
    IResult,
};

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Execute;

impl Execute {
    pub fn handle(key: Key) -> Option<Message> {
        match key {
            Key::Enter => Some(Message::EndCommandLineInput),
            Key::Char(ch) => Some(Message::InsertChar(ch)),
            Key::Left => Some(Message::MoveCursorLeft(1)),
            Key::Right => Some(Message::MoveCursorRight(1)),
            Key::Backspace => Some(Message::DeleteCharBackward),
            Key::Delete => Some(Message::DeleteCharForward),
            Key::Home => Some(Message::MoveCursorLineStart),
            Key::End => Some(Message::MoveCursorLineEnd),
            Key::Esc => Some(Message::AbortCommandLineInput),
            _ => None,
        }
    }

    pub fn parse(command_string: &str) -> Option<Message> {
        command_for_input(command_string)
    }
}

fn command_for_input(input: &str) -> Option<Message> {
    if let Ok((_, command)) = all_consuming(alt((open, quit, save, save_as)))(input) {
        return Some(command);
    }

    None
}

fn open(input: &str) -> IResult<&str, Message> {
    map(
        separated_pair(char('e'), char(' '), many1(anychar)),
        |(_, name)| Message::Open(name.into_iter().collect::<String>()),
    )(input)
}

fn quit(input: &str) -> IResult<&str, Message> {
    value(Message::Quit, all_consuming(char('q')))(input)
}

fn save(input: &str) -> IResult<&str, Message> {
    value(Message::Save, all_consuming(char('w')))(input)
}

fn save_as(input: &str) -> IResult<&str, Message> {
    map(
        separated_pair(char('w'), char(' '), many1(anychar)),
        |(_, name)| Message::SaveAs(name.into_iter().collect::<String>()),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::{command_for_input, open, quit, save, save_as};
    use crate::communication::Message;

    #[test]
    fn test_command_for_input() {
        let tests = vec![
            ("q", Message::Quit),
            ("w", Message::Save),
            ("w some_file.txt", Message::SaveAs("some_file.txt".into())),
        ];

        for (input, command) in tests {
            assert_eq!(command_for_input(input), Some(command));
        }
    }

    #[test]
    fn test_open() {
        assert!(open("e").is_err());
        assert_eq!(
            open("e test.txt"),
            Ok(("", Message::Open("test.txt".into())))
        );
    }

    #[test]
    fn test_quit() {
        assert!(quit("w").is_err());
        assert_eq!(quit("q"), Ok(("", Message::Quit)));
    }

    #[test]
    fn test_save() {
        assert!(save("q").is_err());
        assert_eq!(save("w"), Ok(("", Message::Save)));
    }

    #[test]
    fn test_save_as() {
        assert!(save_as("w").is_err());
        assert_eq!(
            save_as("w test.txt"),
            Ok(("", Message::SaveAs("test.txt".into())))
        );
    }
}
