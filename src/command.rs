use std::{
    fmt::{self, Write},
    io::{self},
};

use constcat::concat;

pub trait Command {
    fn write_ansi<T: fmt::Write>(&self, writer: &mut T) -> fmt::Result;
}

const ESC: &str = "\0x1B";
const CSI: &str = concat!(ESC, '[');

pub struct Clear(pub ClearType);
impl Command for Clear {
    fn write_ansi<T: fmt::Write>(&self, mut writer: &mut T) -> fmt::Result {
        match self.0 {
            // To also clear the scroll back, emit L"\x1b[3J" as well.
            // 2J only clears the visible window and 3J only clears the scroll back.
            ClearType::All => write!(&mut writer, "{CSI}2J")?,
            ClearType::StartTillCursor => {
                write!(&mut writer, "{CSI}1J")?;
            }
            ClearType::CursorTillEnd => {
                write!(&mut writer, "{CSI}J")?;
            }
        }
        Ok(())
        // escape_sequences ->
        //      \x1b[J - clears from the cursor to the end
        //      \x1b[0J - same as \x1b[J
        //      \x1b[1J - clears upto the cursor
        //      \x1b[2J - Clear Screen
        // \0x1B is the hexadecimal value of ESC
    }
}
pub enum ClearType {
    All,
    StartTillCursor,
    CursorTillEnd,
}
pub struct MoveUp(pub u32);
impl Command for MoveUp {
    fn write_ansi<T: fmt::Write>(&self, mut writer: &mut T) -> Result<(), fmt::Error> {
        write!(&mut writer, "{CSI}{}A", self.0)?;
        Ok(())
    }
}
pub struct MoveDown(pub u32);
impl Command for MoveDown {
    fn write_ansi<T: fmt::Write>(&self, mut writer: &mut T) -> fmt::Result {
        write!(&mut writer, "{CSI}{}B", self.0)?;
        Ok(())
    }
}
pub struct MoveTo {
    pub x: u32,
    pub y: u32,
}
impl Command for MoveTo {
    fn write_ansi<T: fmt::Write>(&self, mut writer: &mut T) -> fmt::Result {
        write!(&mut writer, "{CSI}{};{}H", self.y, self.x)?;
        Ok(())
    }
}
impl MoveTo {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}
