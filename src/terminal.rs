use std::fmt::Write as _;

use crate::{command::Command, event::Event, sys};

#[derive(Debug, Default)]
pub struct Terminal {
    queue: String,
}

pub struct TerminalSize {
    pub x: u32,
    pub y: u32,
}

impl Terminal {
    pub fn new() -> anyhow::Result<Self> {
        sys::initialize()?;
        Ok(Self {
            queue: String::new(),
        })
    }

    pub fn deinitialize(&self) -> anyhow::Result<()> {
        sys::deinit()?;
        Ok(())
    }

    pub fn queue_cmd(&mut self, cmd: impl Command) {
        cmd.write_ansi(&mut self.queue);
    }

    pub fn write_str_to_queue(&mut self, text: &str) {
        self.queue.write_str(text);
    }

    pub fn write_char_to_queue(&mut self, ch: char) {
        self.queue.write_char(ch);
    }

    pub fn read(&self) -> anyhow::Result<Vec<Event>> {
        Ok(sys::read()?)
    }

    pub fn flush(&mut self) -> anyhow::Result<()> {
        Ok(sys::flush(&self.queue)?)
    }

    pub fn get_size(&self) -> anyhow::Result<TerminalSize> {
        // the coordinates of a character cell in a console screen buffer. The origin of the coordinate system (0,0) is at the top, left cell of the buffer.
        // X : The horizontal coordinate or column value. The units depend on the function call.
        // Y : The vertical coordinate or row value. The units depend on the function call.

        let coord = sys::ConsoleState::size()?;
        let size = TerminalSize {
            x: coord.X as u32,
            y: coord.Y as u32,
        };
        Ok(size)
    }
}
