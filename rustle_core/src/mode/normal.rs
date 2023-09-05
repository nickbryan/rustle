use super::Mode;
use crate::editor::Command;
use crate::Key;
use nom::{
    branch::alt,
    character::complete::{char, digit0, one_of},
    combinator::{all_consuming, map, recognize, value},
    sequence::pair,
    IResult,
};

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Normal {
    input_buffer: String,
}

impl Normal {
    pub fn handle(&mut self, key: Key) -> Option<Command> {
        if let Key::Char(ch) = key {
            self.input_buffer.push(ch);
        }

        if let Key::Esc = key {
            self.input_buffer.clear();
        }

        match key {
            Key::Home => Some(Command::MoveCursorLineStart),
            Key::End => Some(Command::MoveCursorLineEnd),
            Key::PageUp => Some(Command::MoveCursorPageUp),
            Key::PageDown => Some(Command::MoveCursorPageDown),
            Key::Insert => Some(Command::EnterMode(Mode::Insert)),
            Key::Enter => Some(Command::MoveCursorDown(1)),
            Key::Left => Some(Command::BufferPrevious),
            Key::Right => Some(Command::BufferNext),
            Key::Up => Some(Command::VisualSplit),
            Key::Down => Some(Command::PreviousWindow),
            _ => None,
        }
        .map_or_else(
            || {
                let command = command_for_input(&self.input_buffer);

                if command.is_some() {
                    self.input_buffer.clear();
                }

                command
            },
            Some,
        )
    }
}

pub fn command_for_input(input: &str) -> Option<Command> {
    if let Ok((_, command)) =
        all_consuming(alt((command_mode, insert_mode, movement_action)))(input)
    {
        return Some(command);
    }

    None
}

fn command_mode(input: &str) -> IResult<&str, Command> {
    value(Command::EnterMode(Mode::Execute), char(':'))(input)
}

fn insert_mode(input: &str) -> IResult<&str, Command> {
    value(Command::EnterMode(Mode::Insert), char('i'))(input)
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

fn single_move_action(input: &str) -> IResult<&str, Command> {
    map(movement_key, |c| match c {
        'h' => Command::MoveCursorLeft(1),
        'j' => Command::MoveCursorDown(1),
        'k' => Command::MoveCursorUp(1),
        'l' => Command::MoveCursorRight(1),
        _ => unreachable!(),
    })(input)
}

fn multi_move_action(input: &str) -> IResult<&str, Command> {
    map(pair(multiplier, movement_key), |(m, c)| match c {
        'h' => Command::MoveCursorLeft(m.parse::<usize>().unwrap()),
        'j' => Command::MoveCursorDown(m.parse::<usize>().unwrap()),
        'k' => Command::MoveCursorUp(m.parse::<usize>().unwrap()),
        'l' => Command::MoveCursorRight(m.parse::<usize>().unwrap()),
        _ => unreachable!(),
    })(input)
}

fn movement_action(input: &str) -> IResult<&str, Command> {
    alt((single_move_action, multi_move_action))(input)
}
