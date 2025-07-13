use std::fmt::Write as _;

use crate::{
    command::{CSI, Command, ESC},
    event::Event,
    sys,
};

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
        Self::use_alternate_buffer()?;
        Ok(Self {
            queue: String::new(),
        })
    }

    pub fn use_alternate_buffer() -> anyhow::Result<()> {
        //ESC[?1049h 	Use Alternate Screen Buffer 	Switches to a new alternate screen buffer.
        // ESC[?1049l 	Use Main Screen Buffer 	Switches to the main buffer.
        let command = constcat::concat!(CSI, "?1049h");
        Ok(sys::write_stdout(command)?)
    }

    pub fn deinitialize(&self) -> anyhow::Result<()> {
        sys::deinit()?;
        Ok(())
    }

    pub fn queue_cmd(&mut self, cmd: impl Command) -> anyhow::Result<()> {
        Ok(cmd.write_ansi(&mut self.queue)?)
    }

    pub fn write_str_to_queue(&mut self, text: &str) -> anyhow::Result<()> {
        Ok(self.queue.write_str(text)?)
    }

    pub fn write_char_to_queue(&mut self, ch: char) -> anyhow::Result<()> {
        Ok(self.queue.write_char(ch)?)
    }

    pub fn read(&self) -> anyhow::Result<Vec<Event>> {
        Ok(sys::read()?)
    }

    pub fn flush(&mut self) -> anyhow::Result<()> {
        sys::write_stdout(&self.queue)?;
        self.queue.clear();
        Ok(())
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
