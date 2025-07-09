use crate::command::*;
use crate::{
    command::{ClearType, Command},
    sys,
};
use std::io::{Write, stdout};

#[derive(Default)]
pub struct Editor {
    wants_exit: bool,
}

impl Editor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn run(&mut self) {
        Self::initialize();
        let result = self.repl();
        Self::terminate().unwrap();
        result.unwrap();
        print!("\nExiting...Goodbye. \r\n");
    }

    fn initialize() -> anyhow::Result<()> {
        // crossterm::terminal::enable_raw_mode()?;
        sys::initialize()?;
        Self::clear_screen();
        Ok(())
    }

    fn terminate() -> anyhow::Result<()> {
        Ok(sys::deinit()?)
    }

    fn clear_screen() -> () {
        sys::ConsoleState::queue(Clear(ClearType::All));
    }

    fn refresh_screen(&self) -> anyhow::Result<()> {
        sys::flush()?;
        Ok(())
    }

    fn repl(&mut self) -> Result<(), std::io::Error> {
        let mut stdout = stdout();
        let screen_height = crossterm::terminal::size()?;

        for i in 0..screen_height.1 - 1 {
            print!("~");
            print!("\r\n");
        }
        sys::ConsoleState::queue(MoveTo::new(0, 0));

        loop {
            let event = read()?;
            self.event_handler(&event);
            self.refresh_screen()?;
            if self.wants_exit {
                break;
            }
        }
        Ok(())
    }

    fn event_handler(&mut self, event: &Event) {
        match *event {
            Event::Key(KeyEvent {
                code,
                modifiers,
                kind,
                state,
            }) => match code {
                KeyCode::Char('q') if modifiers == KeyModifiers::CONTROL => {
                    self.wants_exit = true;
                }
                KeyCode::Char(key) => {
                    print!("{key}")
                }
                KeyCode::Enter => {
                    print!("\r\n~")
                }
                KeyCode::Backspace
                | KeyCode::Left
                | KeyCode::Right
                | KeyCode::Up
                | KeyCode::Down
                | KeyCode::Home
                | KeyCode::End
                | KeyCode::PageUp
                | KeyCode::PageDown
                | KeyCode::Tab
                | KeyCode::BackTab
                | KeyCode::Delete
                | KeyCode::Insert
                | KeyCode::F(_)
                | KeyCode::Null
                | KeyCode::Esc
                | KeyCode::CapsLock
                | KeyCode::ScrollLock
                | KeyCode::NumLock
                | KeyCode::PrintScreen
                | KeyCode::Pause
                | KeyCode::Menu
                | KeyCode::KeypadBegin => {}
                KeyCode::Media(media_key_code) => {}
                KeyCode::Modifier(modifier_key_code) => {}
            },
            Event::FocusGained | Event::FocusLost | Event::Paste(_) | Event::Resize(_, _) => {}
            Event::Mouse(mouse_event) => {}
            _ => {}
        }
    }
}
