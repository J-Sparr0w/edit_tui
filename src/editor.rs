use std::io::{Write, stdout};

use crossterm::{
    event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, read},
    execute,
    terminal::Clear,
};

#[derive(Default)]
pub struct Editor {
    wants_exit: bool,
}

impl Editor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn run(&mut self) {
        Self::initialize().unwrap();
        let result = self.repl();
        Self::terminate().unwrap();
        result.unwrap();
        print!("Exiting...Goodbye. \r\n");
    }

    fn initialize() -> Result<(), std::io::Error> {
        crossterm::terminal::enable_raw_mode()?;
        Self::clear_screen()
    }

    fn terminate() -> Result<(), std::io::Error> {
        Ok(crossterm::terminal::disable_raw_mode()?)
    }

    fn clear_screen() -> std::io::Result<()> {
        let mut stdout = std::io::stdout();
        execute!(stdout, Clear(crossterm::terminal::ClearType::All))
    }

    fn refresh_screen(&self) -> std::io::Result<()> {
        stdout().flush()?;
        Ok(())
    }

    fn repl(&mut self) -> Result<(), std::io::Error> {
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
