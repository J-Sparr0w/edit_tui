use crate::command::*;
use crate::command::{ClearType, Command};
use crate::event::Event;
use crate::terminal::Terminal;
use std::io::{Write, stdout};

#[derive(Default)]
pub struct Editor {
    wants_exit: bool,
    terminal: Terminal,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            wants_exit: false,
            terminal: Terminal::new().expect("Terminal initialization failed"),
        }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        self.initialize();
        let result = self.repl();
        self.terminate().unwrap();
        result.unwrap();
        print!("\nExiting...Goodbye. \r\n");
        Ok(())
    }

    fn initialize(&mut self) -> anyhow::Result<()> {
        // crossterm::terminal::enable_raw_mode()?;
        self.clear_screen();
        Ok(())
    }

    fn terminate(&self) -> anyhow::Result<()> {
        Ok(self.terminal.deinitialize()?)
    }

    fn clear_screen(&mut self) -> () {
        self.terminal.queue_cmd(Clear(ClearType::All));
    }

    fn refresh_screen(&mut self) -> anyhow::Result<()> {
        self.terminal.flush()?;
        Ok(())
    }

    fn repl(&mut self) -> anyhow::Result<()> {
        let screen_height = self.terminal.get_size()?.y;
        for _ in 0..screen_height - 1 {
            print!("~");
            print!("\r\n");
        }
        self.terminal.queue_cmd(MoveTo::new(0, 0));

        loop {
            let event = self.terminal.read()?;
            self.event_handler(&event);
            self.refresh_screen()?;
            if self.wants_exit {
                break;
            }
        }
        Ok(())
    }

    fn event_handler(&mut self, events: &[Event]) {
        for event in events {
            match *event {
                Event::Key(ch) => {
                    self.terminal.write_char_to_queue(ch);
                } // Event::Key(KeyEvent {
                  //     code,
                  //     modifiers,
                  //     kind,
                  //     state,
                  // }) => match code {
                  //     KeyCode::Char('q') if modifiers == KeyModifiers::CONTROL => {
                  //         self.wants_exit = true;
                  //     }
                  //     KeyCode::Char(key) => {
                  //         print!("{key}")
                  //     }
                  //     KeyCode::Enter => {
                  //         print!("\r\n~")
                  //     }
                  //     KeyCode::Backspace
                  //     | KeyCode::Left
                  //     | KeyCode::Right
                  //     | KeyCode::Up
                  //     | KeyCode::Down
                  //     | KeyCode::Home
                  //     | KeyCode::End
                  //     | KeyCode::PageUp
                  //     | KeyCode::PageDown
                  //     | KeyCode::Tab
                  //     | KeyCode::BackTab
                  //     | KeyCode::Delete
                  //     | KeyCode::Insert
                  //     | KeyCode::F(_)
                  //     | KeyCode::Null
                  //     | KeyCode::Esc
                  //     | KeyCode::CapsLock
                  //     | KeyCode::ScrollLock
                  //     | KeyCode::NumLock
                  //     | KeyCode::PrintScreen
                  //     | KeyCode::Pause
                  //     | KeyCode::Menu
                  //     | KeyCode::KeypadBegin => {}
                  //     KeyCode::Media(media_key_code) => {}
                  //     KeyCode::Modifier(modifier_key_code) => {}
                  // },
                  // Event::FocusGained | Event::FocusLost | Event::Paste(_) | Event::Resize(_, _) => {}
                  // Event::Mouse(mouse_event) => {}
                  // _ => {}
            }
        }
    }
}
