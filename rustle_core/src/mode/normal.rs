use super::Mode;
use crate::communication::Message;
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
            Key::Insert => Some(Message::EnterMode(Mode::Insert)),
            Key::Enter => Some(Message::MoveCursorDown(1)),
            Key::Left => Some(Message::BufferPrevious),
            Key::Right => Some(Message::BufferNext),
            Key::Up => Some(Message::VisualSplit),
            Key::Down => Some(Message::PreviousWindow),
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

pub fn command_for_input(input: &str) -> Option<Message> {
    if let Ok((_, command)) =
        all_consuming(alt((command_mode, insert_mode, movement_action)))(input)
    {
        return Some(command);
    }

    None
}

fn command_mode(input: &str) -> IResult<&str, Message> {
    value(Message::EnterMode(Mode::Execute), char(':'))(input)
}

fn insert_mode(input: &str) -> IResult<&str, Message> {
    value(Message::EnterMode(Mode::Insert), char('i'))(input)
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
