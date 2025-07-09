use std::{
    ffi::c_void,
    fmt::Write,
    io::stdout,
    mem::MaybeUninit,
    os::windows::io::AsHandle,
    ptr::{self, null, null_mut},
};

use crossterm::queue;
use thiserror::Error;
use windows_sys::{
    Win32::{
        Foundation, Globalization,
        Storage::FileSystem,
        System::{
            Console::{
                self, DISABLE_NEWLINE_AUTO_RETURN, ENABLE_EXTENDED_FLAGS, ENABLE_PROCESSED_OUTPUT,
                ENABLE_VIRTUAL_TERMINAL_INPUT, ENABLE_VIRTUAL_TERMINAL_PROCESSING,
                ENABLE_WINDOW_INPUT, ENABLE_WRAP_AT_EOL_OUTPUT, GetConsoleCP, GetConsoleMode,
                GetConsoleOutputCP, GetNumberOfConsoleInputEvents, GetStdHandle, INPUT_RECORD,
                ReadConsoleA, ReadConsoleInputA, SetConsoleMode,
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
    event::Event,
};
#[derive(Debug, Error)]
pub enum ConsoleError {
    #[error("Could not change Console Mode, error code: [{0}]")]
    ConsoleMode(u32),
}

const INVALID_CONSOLE_MODE: u32 = u32::MAX;
pub struct ConsoleState {
    stdin: Foundation::HANDLE,
    stdout: Foundation::HANDLE,
    old_stdin_mode: u32,
    old_stdout_mode: u32,
    stdin_cp_old: u32,
    stdout_cp_old: u32,
    queue: String,
}

impl ConsoleState {
    pub fn read_console_input(
        hconsoleinput: *mut c_void,
        lpbuffer: *mut INPUT_RECORD,
        nlength: u32,
        lpnumberofeventsread: *mut u32,
    ) {
        unsafe {
            Console::ReadConsoleInputW(hconsoleinput, lpbuffer, nlength, lpnumberofeventsread);
        }
    }

    pub fn queue(cmd: impl Command) {
        unsafe {
            cmd.write_ansi(&mut GLOBAL_CONSOLE_STATE.queue);
        }
    }
}

static mut GLOBAL_CONSOLE_STATE: ConsoleState = ConsoleState {
    stdin: null_mut(),
    stdout: null_mut(),
    queue: String::new(),
    old_stdin_mode: INVALID_CONSOLE_MODE,
    old_stdout_mode: INVALID_CONSOLE_MODE,
    stdin_cp_old: 0,
    stdout_cp_old: 0,
};

pub fn initialize() -> Result<(), ConsoleError> {
    init()?;
    enable_raw_mode()?;
    Ok(())
}

pub fn init() -> Result<(), ConsoleError> {
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
            return Err(ConsoleError::ConsoleMode(get_last_error_code()));
        }
    }
    Ok(())
}

fn check_nonzero_success(ret: BOOL) -> Result<(), ConsoleError> {
    if ret == 0 {
        return Err(ConsoleError::ConsoleMode(get_last_error_code()));
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
        ))?;

        check_nonzero_success(GetConsoleMode(
            GLOBAL_CONSOLE_STATE.stdout,
            &raw mut GLOBAL_CONSOLE_STATE.old_stdout_mode,
        ))?;

        check_nonzero_success(Console::SetConsoleCP(Globalization::CP_UTF8))?;
        check_nonzero_success(Console::SetConsoleOutputCP(Globalization::CP_UTF8))?;

        check_nonzero_success(SetConsoleMode(
            GLOBAL_CONSOLE_STATE.stdin,
            ENABLE_WINDOW_INPUT | ENABLE_EXTENDED_FLAGS | ENABLE_VIRTUAL_TERMINAL_INPUT,
        ))?;
        check_nonzero_success(SetConsoleMode(
            GLOBAL_CONSOLE_STATE.stdout,
            ENABLE_PROCESSED_OUTPUT
                | ENABLE_WRAP_AT_EOL_OUTPUT
                | ENABLE_VIRTUAL_TERMINAL_PROCESSING
                | DISABLE_NEWLINE_AUTO_RETURN,
        ))?;
    }
    Ok(())
}

pub fn flush() -> Result<(), ConsoleError> {
    let mut chars_written: u32 = 0;

    unsafe {
        let queue_len = GLOBAL_CONSOLE_STATE.queue.len();
        let mut offset = 0;

        if queue_len == 0 {
            return Ok(());
        }

        while offset < queue_len {
            //can use both WriteConsole(There is only WriteConsoleA or WriteConsoleW in rust windows) and WriteFile to write to output screen buffer.
            //WriteConsoleA might probably work fine.
            if check_nonzero_success(FileSystem::WriteFile(
                GLOBAL_CONSOLE_STATE.stdout,
                GLOBAL_CONSOLE_STATE.queue.as_ptr().add(offset),
                queue_len as u32,
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
    const LEN: u32 = 128;
    let mut total_events_read: u32 = 0;
    let mut unread_events = 0;
    let mut buf = Vec::new();
    let mut events = Vec::new();
    unsafe {
        check_nonzero_success(GetNumberOfConsoleInputEvents(
            GLOBAL_CONSOLE_STATE.stdin,
            &mut unread_events,
        ));
        while unread_events > 0 && total_events_read <= LEN {
            let mut events_read = 0;
            check_nonzero_success(ReadConsoleInputA(
                GLOBAL_CONSOLE_STATE.stdin,
                buf.as_mut_ptr(),
                LEN,
                &mut events_read,
            ))?;
            total_events_read += events_read;
        }
        for input in buf {
            match input.EventType as u32 {
                Console::KEY_EVENT => {
                    let event = input.Event.KeyEvent;
                    let ch = event.uChar.UnicodeChar;
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
            ))?;
        }
        if GLOBAL_CONSOLE_STATE.old_stdout_mode != INVALID_CONSOLE_MODE {
            check_nonzero_success(SetConsoleMode(
                GLOBAL_CONSOLE_STATE.stdout,
                GLOBAL_CONSOLE_STATE.old_stdout_mode,
            ))?;
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
        check_nonzero_success(CancelIoEx(GLOBAL_CONSOLE_STATE.stdin, null()));
    }
    Foundation::TRUE
}
