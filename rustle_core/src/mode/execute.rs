use crate::{editor::Command, Key};
use nom::IResult;

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Execute;

impl Execute {
    pub fn handle(key: Key) -> Option<Command> {
        match key {
            Key::Enter => Some(Command::EndCommandLineInput),
            Key::Char(ch) => Some(Command::InsertChar(ch)),
            Key::Left => Some(Command::MoveCursorLeft(1)),
            Key::Right => Some(Command::MoveCursorRight(1)),
            Key::Backspace => Some(Command::DeleteCharBackward),
            Key::Delete => Some(Command::DeleteCharForward),
            Key::Home => Some(Command::MoveCursorLineStart),
            Key::End => Some(Command::MoveCursorLineEnd),
            Key::Esc => Some(Command::AbortCommandLineInput),
            _ => None,
        }
    }

    pub fn parse(command_string: &str) -> Option<Command> {
        command_for_input(command_string)
    }
}

fn command_for_input(input: &str) -> Option<Command> {
    if let Ok((_, command)) =
        nom::combinator::all_consuming(nom::branch::alt((open, quit, save, save_as)))(input)
    {
        return Some(command);
    }

    None
}

fn open(input: &str) -> IResult<&str, Command> {
    nom::combinator::map(
        nom::sequence::separated_pair(
            nom::character::complete::char('o'),
            nom::character::complete::char(' '),
            nom::multi::many1(nom::character::complete::anychar),
        ),
        |(_, name)| Command::Open(name.into_iter().collect::<String>()),
    )(input)
}

fn quit(input: &str) -> IResult<&str, Command> {
    nom::combinator::value(
        Command::Quit,
        nom::combinator::all_consuming(nom::character::complete::char('q')),
    )(input)
}

fn save(input: &str) -> IResult<&str, Command> {
    nom::combinator::value(
        Command::Save,
        nom::combinator::all_consuming(nom::character::complete::char('w')),
    )(input)
}

fn save_as(input: &str) -> IResult<&str, Command> {
    nom::combinator::map(
        nom::sequence::separated_pair(
            nom::character::complete::char('w'),
            nom::character::complete::char(' '),
            nom::multi::many1(nom::character::complete::anychar),
        ),
        |(_, name)| Command::SaveAs(name.into_iter().collect::<String>()),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::{command_for_input, open, quit, save, save_as};
    use crate::editor::Command;

    #[test]
    fn test_command_for_input() {
        let tests = vec![
            ("q", Command::Quit),
            ("w", Command::Save),
            ("w some_file.txt", Command::SaveAs("some_file.txt".into())),
        ];

        for (input, command) in tests {
            assert_eq!(command_for_input(input), Some(command));
        }
    }

    #[test]
    fn test_open() {
        assert!(open("o").is_err());
        assert_eq!(
            open("o test.txt"),
            Ok(("", Command::Open("test.txt".into())))
        );
    }

    #[test]
    fn test_quit() {
        assert!(quit("w").is_err());
        assert_eq!(quit("q"), Ok(("", Command::Quit)));
    }

    #[test]
    fn test_save() {
        assert!(save("q").is_err());
        assert_eq!(save("w"), Ok(("", Command::Save)));
    }

    #[test]
    fn test_save_as() {
        assert!(save_as("w").is_err());
        assert_eq!(
            save_as("w test.txt"),
            Ok(("", Command::SaveAs("test.txt".into())))
        );
    }
}
