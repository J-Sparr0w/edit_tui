use std::{
    ffi::c_void,
    fmt::Write,
    io::{Write as W, stdout},
    mem::MaybeUninit,
    os::windows::io::AsHandle,
    ptr::{self, null, null_mut},
};

use thiserror::Error;
use windows_sys::{
    Win32::{
        Foundation, Globalization,
        Storage::FileSystem,
        System::{
            Console::{
                self, CONSOLE_SCREEN_BUFFER_INFO, COORD, DISABLE_NEWLINE_AUTO_RETURN,
                ENABLE_EXTENDED_FLAGS, ENABLE_PROCESSED_OUTPUT, ENABLE_VIRTUAL_TERMINAL_INPUT,
                ENABLE_VIRTUAL_TERMINAL_PROCESSING, ENABLE_WINDOW_INPUT, ENABLE_WRAP_AT_EOL_OUTPUT,
                GetConsoleCP, GetConsoleMode, GetConsoleOutputCP, GetConsoleScreenBufferInfo,
                GetNumberOfConsoleInputEvents, GetStdHandle, INPUT_RECORD, ReadConsoleA,
                ReadConsoleInputA, SetConsoleMode,
            },
            Diagnostics::Debug::OutputDebugStringA,
            IO::CancelIoEx,
        },
    },
    core::BOOL,
    w,
};

use crate::{
    command::{self, Command},
    event::{Event, KeyPressState, ModifierKeyCode},
};
#[derive(Debug, Error)]
pub enum ConsoleError {
    #[error("Could not change Console Mode, error code: [{0}]")]
    SetConsoleMode(u32),
    #[error("Could not query Console Mode, error code: [{0}]")]
    QueryConsoleMode(u32),
    #[error("Could not change Console Page, error code: [{0}]")]
    SetConsoleCodePage(u32),
    #[error("Could not query Console Page, error code: [{0}]")]
    QueryConsoleCodePage(u32),
    #[error("Could not query Terminal Size: [{0}]")]
    QueryTerminalSize(u32),
    #[error("Could not query number of Console Events: [{0}]")]
    QueryNumberOfConsoleEvents(u32),
    #[error("Could not read Console Input: [{0}]")]
    ReadConsoleInput(u32),
}

const INVALID_CONSOLE_MODE: u32 = u32::MAX;
pub struct ConsoleState {
    stdin: Foundation::HANDLE,
    stdout: Foundation::HANDLE,
    old_stdin_mode: u32,
    old_stdout_mode: u32,
    stdin_cp_old: u32,
    stdout_cp_old: u32,
}

impl ConsoleState {
    pub fn size() -> Result<COORD, ConsoleError> {
        // the coordinates of a character cell in a console screen buffer. The origin of the coordinate system (0,0) is at the top, left cell of the buffer.
        // X
        // The horizontal coordinate or column value. The units depend on the function call.

        // Y
        // The vertical coordinate or row value. The units depend on the function call.

        let size;
        unsafe {
            let mut screen_buffer_info: CONSOLE_SCREEN_BUFFER_INFO =
                CONSOLE_SCREEN_BUFFER_INFO::default();
            check_nonzero_success(GetConsoleScreenBufferInfo(
                GLOBAL_CONSOLE_STATE.stdout,
                &mut screen_buffer_info,
            ))
            .map_err(|_| ConsoleError::QueryTerminalSize(get_last_error_code()))?;
            size = screen_buffer_info.dwSize;
        }
        Ok(size)
    }
}

static mut GLOBAL_CONSOLE_STATE: ConsoleState = ConsoleState {
    stdin: null_mut(),
    stdout: null_mut(),
    old_stdin_mode: INVALID_CONSOLE_MODE,
    old_stdout_mode: INVALID_CONSOLE_MODE,
    stdin_cp_old: 0,
    stdout_cp_old: 0,
};

pub fn initialize() -> Result<(), ConsoleError> {
    init_std()?;
    enable_raw_mode()?;
    Ok(())
}

pub fn init_std() -> Result<(), ConsoleError> {
    unsafe {
        GLOBAL_CONSOLE_STATE.stdin = GetStdHandle(Console::STD_INPUT_HANDLE);
        GLOBAL_CONSOLE_STATE.stdout = GetStdHandle(Console::STD_OUTPUT_HANDLE);

        // stdin can be redirected
        if !ptr::eq(GLOBAL_CONSOLE_STATE.stdin, Foundation::INVALID_HANDLE_VALUE)
            && matches!(
                FileSystem::GetFileType(GLOBAL_CONSOLE_STATE.stdin),
                FileSystem::FILE_TYPE_DISK | FileSystem::FILE_TYPE_PIPE
            )
        {
            GLOBAL_CONSOLE_STATE.stdin = FileSystem::CreateFileW(
                w!("CONIN$"),
                Foundation::GENERIC_READ | Foundation::GENERIC_WRITE,
                FileSystem::FILE_SHARE_READ | FileSystem::FILE_SHARE_WRITE,
                null_mut(),
                FileSystem::OPEN_EXISTING,
                0,
                null_mut(),
            );
        }

        if ptr::eq(GLOBAL_CONSOLE_STATE.stdin, Foundation::INVALID_HANDLE_VALUE)
            || ptr::eq(
                GLOBAL_CONSOLE_STATE.stdout,
                Foundation::INVALID_HANDLE_VALUE,
            )
        {
            return Err(ConsoleError::SetConsoleMode(get_last_error_code()));
        }
    }
    Ok(())
}

fn check_nonzero_success(ret: BOOL) -> Result<(), ()> {
    if ret == 0 {
        return Err(());
    } else {
        return Ok(());
    }
}

pub fn enable_raw_mode() -> Result<(), ConsoleError> {
    unsafe {
        GLOBAL_CONSOLE_STATE.old_stdin_mode = GetConsoleCP();
        GLOBAL_CONSOLE_STATE.old_stdout_mode = GetConsoleOutputCP();

        check_nonzero_success(GetConsoleMode(
            GLOBAL_CONSOLE_STATE.stdin,
            &raw mut GLOBAL_CONSOLE_STATE.old_stdin_mode,
        ))
        .map_err(|_| ConsoleError::QueryConsoleMode(get_last_error_code()))?;

        check_nonzero_success(GetConsoleMode(
            GLOBAL_CONSOLE_STATE.stdout,
            &raw mut GLOBAL_CONSOLE_STATE.old_stdout_mode,
        ))
        .map_err(|_| ConsoleError::QueryConsoleMode(get_last_error_code()))?;

        check_nonzero_success(Console::SetConsoleCP(Globalization::CP_UTF8))
            .map_err(|_| ConsoleError::SetConsoleCodePage(get_last_error_code()))?;

        check_nonzero_success(Console::SetConsoleOutputCP(Globalization::CP_UTF8))
            .map_err(|_| ConsoleError::SetConsoleCodePage(get_last_error_code()))?;

        check_nonzero_success(SetConsoleMode(
            GLOBAL_CONSOLE_STATE.stdin,
            ENABLE_WINDOW_INPUT | ENABLE_EXTENDED_FLAGS | ENABLE_VIRTUAL_TERMINAL_INPUT,
        ))
        .map_err(|_| ConsoleError::SetConsoleMode(get_last_error_code()))?;
        check_nonzero_success(SetConsoleMode(
            GLOBAL_CONSOLE_STATE.stdout,
            ENABLE_PROCESSED_OUTPUT
                | ENABLE_WRAP_AT_EOL_OUTPUT
                | ENABLE_VIRTUAL_TERMINAL_PROCESSING
                | DISABLE_NEWLINE_AUTO_RETURN,
        ))
        .map_err(|_| ConsoleError::SetConsoleMode(get_last_error_code()))?;

        #[cfg(debug_assertions)]
        {
            use crate::sys;

            let mut current_stdin = 0;
            let mut current_stdout = 0;
            check_nonzero_success(GetConsoleMode(
                GLOBAL_CONSOLE_STATE.stdin,
                &raw mut current_stdin,
            ))
            .map_err(|_| ConsoleError::QueryConsoleMode(get_last_error_code()))?;

            check_nonzero_success(GetConsoleMode(
                GLOBAL_CONSOLE_STATE.stdout,
                &raw mut current_stdout,
            ))
            .map_err(|_| ConsoleError::QueryConsoleMode(get_last_error_code()))?;

            assert_eq!(
                current_stdin,
                ENABLE_WINDOW_INPUT | ENABLE_EXTENDED_FLAGS | ENABLE_VIRTUAL_TERMINAL_INPUT
            );
            assert_eq!(
                current_stdout,
                ENABLE_PROCESSED_OUTPUT
                    | ENABLE_WRAP_AT_EOL_OUTPUT
                    | ENABLE_VIRTUAL_TERMINAL_PROCESSING
                    | DISABLE_NEWLINE_AUTO_RETURN
            );
            // sys::write_stdout(&format!(
            //     "stdin raw mode is : {}\n",
            //     current_stdin
            //         == ENABLE_WINDOW_INPUT | ENABLE_EXTENDED_FLAGS | ENABLE_VIRTUAL_TERMINAL_INPUT
            // ))?;
            // sys::write_stdout(&format!(
            //     "stdout raw mode is : {}\n",
            //     current_stdout
            //         == ENABLE_PROCESSED_OUTPUT
            //             | ENABLE_WRAP_AT_EOL_OUTPUT
            //             | ENABLE_VIRTUAL_TERMINAL_PROCESSING
            //             | DISABLE_NEWLINE_AUTO_RETURN
            // ))?;
        }
    }
    Ok(())
}

pub fn write_stdout(text: &str) -> Result<(), ConsoleError> {
    let mut chars_written: u32 = 0;

    unsafe {
        let len = text.len();
        let mut offset = 0;

        if len == 0 {
            return Ok(());
        }

        while offset < len {
            //can use both WriteConsole(There is only WriteConsoleA or WriteConsoleW in rust windows) and WriteFile to write to output screen buffer.
            //WriteConsoleA might probably work fine.
            if check_nonzero_success(FileSystem::WriteFile(
                GLOBAL_CONSOLE_STATE.stdout,
                text.as_ptr().add(offset),
                len as u32,
                &mut chars_written,
                null_mut(),
            ))
            .is_err()
                || chars_written == 0
            {
                break;
            }
            offset += chars_written as usize;
        }
    }

    Ok(())
}

pub fn read() -> Result<Vec<Event>, ConsoleError> {
    const LEN: u32 = 1024;
    let mut total_events_read: u32 = 0;
    let mut unread_events = 0;
    let mut buf = Vec::with_capacity(LEN as usize);
    let mut events = Vec::new();
    unsafe {
        check_nonzero_success(GetNumberOfConsoleInputEvents(
            GLOBAL_CONSOLE_STATE.stdin,
            &mut unread_events,
        ))
        .map_err(|_| ConsoleError::QueryNumberOfConsoleEvents(get_last_error_code()))?;
        while unread_events > 0 && total_events_read <= LEN {
            let mut events_read = 0;
            check_nonzero_success(ReadConsoleInputA(
                GLOBAL_CONSOLE_STATE.stdin,
                buf.as_mut_ptr(),
                LEN,
                &mut events_read,
            ))
            .map_err(|_| ConsoleError::ReadConsoleInput(get_last_error_code()))?;
            total_events_read += events_read;
            // write_stdout(&format!(
            //     "unread_events: {}, total_events_read: {}",
            //     unread_events, total_events_read
            // ))?;
            unread_events = 0;
        }

        for input in buf {
            write_stdout(&format!("input: {:?}", input.EventType))?;
            match input.EventType as u32 {
                Console::KEY_EVENT => {
                    let event = input.Event.KeyEvent;
                    let ch = event.uChar.UnicodeChar;
                    //bKeyDown : If the key is pressed, this member is TRUE. Otherwise, this member is FALSE (the key is released).
                    if ch != 0 {
                        if let Some(ch) = char::from_u32(ch as u32) {
                            let ctrl = (event.dwControlKeyState & 0x0008 != 0)
                                || (event.dwControlKeyState & 0x0004 != 0);

                            let alt = (event.dwControlKeyState & 0x0002 != 0)
                                || (event.dwControlKeyState & 0x0001 != 0);

                            let shift = event.dwControlKeyState & 0x0010 != 0;
                            let key_press_state;
                            if event.bKeyDown != 0 {
                                key_press_state = KeyPressState::KeyDown;
                            } else {
                                key_press_state = KeyPressState::KeyUp;
                            }

                            let key_event = Event::Key {
                                ch,
                                modifiers: ModifierKeyCode::new()
                                    .set_ctrl(ctrl)
                                    .set_alt(alt)
                                    .set_shift(shift),
                                state: key_press_state,
                            };

                            events.push(key_event);
                        }
                    }
                }
                Console::WINDOW_BUFFER_SIZE_EVENT => {}
                _ => {}
            }
        }
    }
    Ok(events)
}

pub fn deinit() -> Result<(), ConsoleError> {
    disable_raw_mode()?;
    Ok(())
}
pub fn disable_raw_mode() -> Result<(), ConsoleError> {
    unsafe {
        if GLOBAL_CONSOLE_STATE.stdin_cp_old != 0 {
            Console::SetConsoleCP(GLOBAL_CONSOLE_STATE.stdin_cp_old);
            GLOBAL_CONSOLE_STATE.stdin_cp_old = 0;
        }
        if GLOBAL_CONSOLE_STATE.stdout_cp_old != 0 {
            Console::SetConsoleOutputCP(GLOBAL_CONSOLE_STATE.stdout_cp_old);
            GLOBAL_CONSOLE_STATE.stdout_cp_old = 0;
        }
        if GLOBAL_CONSOLE_STATE.old_stdin_mode != INVALID_CONSOLE_MODE {
            check_nonzero_success(SetConsoleMode(
                GLOBAL_CONSOLE_STATE.stdin,
                GLOBAL_CONSOLE_STATE.old_stdin_mode,
            ))
            .map_err(|_| ConsoleError::SetConsoleMode(get_last_error_code()))?;
        }
        if GLOBAL_CONSOLE_STATE.old_stdout_mode != INVALID_CONSOLE_MODE {
            check_nonzero_success(SetConsoleMode(
                GLOBAL_CONSOLE_STATE.stdout,
                GLOBAL_CONSOLE_STATE.old_stdout_mode,
            ))
            .map_err(|_| ConsoleError::SetConsoleMode(get_last_error_code()))?;
        }
    }

    Ok(())
}

fn get_last_error_code() -> u32 {
    unsafe { Foundation::GetLastError() }
}

extern "system" fn console_ctrl_handler(_ctrl_type: u32) -> BOOL {
    unsafe {
        OutputDebugStringA("ctrl_handler invoked \0".as_ptr());
        // GLOBAL_CONSOLE_STATE.wants_exit = true;
        // windows_sys::Win32::System::IO::CancelIoEx(GLOBAL_CONSOLE_STATE.stdin, null());
        check_nonzero_success(CancelIoEx(GLOBAL_CONSOLE_STATE.stdin, null()))
            .expect("CancelIoEx Failed");
    }
    Foundation::TRUE
}
