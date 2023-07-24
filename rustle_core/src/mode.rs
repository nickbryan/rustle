use crate::communication::Message;
use crate::ui::Position;
use crate::{Key, Row};
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Mode {
    Execute(Execute),
    Insert(Insert),
    Normal(Normal),
}

impl Default for Mode {
    fn default() -> Self {
        Self::Normal(Normal::default())
    }
}

impl Display for Mode {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Execute(_) => write!(f, "COMMAND"),
            Self::Insert(_) => write!(f, "INSERT"),
            Self::Normal(_) => write!(f, "NORMAL"),
        }
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Execute {
    row: Row,
    cursor_position: Position,
}

impl Execute {
    pub fn handle(&self, key: Key) -> Option<Message> {
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

    pub fn parse(&self, command_string: &str) -> Option<Message> {
        execute::command_for_input(command_string)
    }
}

mod execute {
    use crate::communication::Message;
    use nom::{
        branch::alt,
        character::complete::{anychar, char},
        combinator::{all_consuming, map, value},
        multi::many1,
        sequence::separated_pair,
        IResult,
    };

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

    pub fn command_for_input(input: &str) -> Option<Message> {
        if let Ok((_, command)) = all_consuming(alt((quit, save, save_as)))(input) {
            return Some(command);
        }

        None
    }

    #[cfg(test)]
    mod tests {
        use super::{command_for_input, quit, save, save_as};
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
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Insert;

impl Insert {
    pub fn handle(&self, key: Key) -> Option<Message> {
        match key {
            Key::Up => Some(Message::MoveCursorUp(1)),
            Key::Down => Some(Message::MoveCursorDown(1)),
            Key::Left => Some(Message::MoveCursorLeft(1)),
            Key::Right => Some(Message::MoveCursorRight(1)),
            Key::Home => Some(Message::MoveCursorLineStart),
            Key::End => Some(Message::MoveCursorLineEnd),
            Key::PageUp => Some(Message::MoveCursorPageUp),
            Key::PageDown => Some(Message::MoveCursorPageDown),
            Key::Delete => Some(Message::DeleteCharForward),
            Key::Backspace => Some(Message::DeleteCharBackward),
            Key::Enter => Some(Message::InsertLineBreak),
            Key::Char(ch) => Some(Message::InsertChar(ch)),
            Key::Esc => Some(Message::EnterMode(Mode::Normal(Normal::default()))),
            _ => None,
        }
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Normal {
    input_buffer: String,
}

impl Normal {
    pub fn handle(&mut self, key: Key) -> Option<Message> {
        if let Key::Char(ch) = key {
            self.input_buffer.push(ch);
        }

        if let Key::Esc = key {
            self.input_buffer.clear();
        }

        match key {
            Key::Home => Some(Message::MoveCursorLineStart),
            Key::End => Some(Message::MoveCursorLineEnd),
            Key::PageUp => Some(Message::MoveCursorPageUp),
            Key::PageDown => Some(Message::MoveCursorPageDown),
            Key::Insert => Some(Message::EnterMode(Mode::Insert(Insert::default()))),
            Key::Enter => Some(Message::MoveCursorDown(1)),
            _ => None,
        }
        .map_or_else(
            || {
                let command = normal::command_for_input(&self.input_buffer);
                self.input_buffer.clear();
                command
            },
            Some,
        )
    }
}

mod normal {
    use super::{Execute, Insert, Mode};
    use crate::communication::Message;
    use nom::{
        branch::alt,
        character::complete::{char, digit0, one_of},
        combinator::{all_consuming, map, recognize, value},
        sequence::pair,
        IResult,
    };

    pub fn command_for_input(input: &str) -> Option<Message> {
        if let Ok((_, command)) =
            all_consuming(alt((command_mode, insert_mode, movement_action)))(input)
        {
            return Some(command);
        }

        None
    }

    fn command_mode(input: &str) -> IResult<&str, Message> {
        value(
            Message::EnterMode(Mode::Execute(Execute::default())),
            char(':'),
        )(input)
    }

    fn insert_mode(input: &str) -> IResult<&str, Message> {
        value(
            Message::EnterMode(Mode::Insert(Insert::default())),
            char('i'),
        )(input)
    }

    fn non_zero_digit(input: &str) -> IResult<&str, char> {
        one_of("123456789")(input)
    }

    fn multiplier(input: &str) -> IResult<&str, &str> {
        recognize(pair(non_zero_digit, digit0))(input)
    }

    fn movement_key(input: &str) -> IResult<&str, char> {
        alt((char('h'), char('j'), char('k'), char('l')))(input)
    }

    fn single_move_action(input: &str) -> IResult<&str, Message> {
        map(movement_key, |c| match c {
            'h' => Message::MoveCursorLeft(1),
            'j' => Message::MoveCursorDown(1),
            'k' => Message::MoveCursorUp(1),
            'l' => Message::MoveCursorRight(1),
            _ => unreachable!(),
        })(input)
    }

    fn multi_move_action(input: &str) -> IResult<&str, Message> {
        map(pair(multiplier, movement_key), |(m, c)| match c {
            'h' => Message::MoveCursorLeft(m.parse::<usize>().unwrap()),
            'j' => Message::MoveCursorDown(m.parse::<usize>().unwrap()),
            'k' => Message::MoveCursorUp(m.parse::<usize>().unwrap()),
            'l' => Message::MoveCursorRight(m.parse::<usize>().unwrap()),
            _ => unreachable!(),
        })(input)
    }

    fn movement_action(input: &str) -> IResult<&str, Message> {
        alt((single_move_action, multi_move_action))(input)
    }
}
