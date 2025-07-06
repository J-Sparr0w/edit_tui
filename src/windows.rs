use std::{ffi::c_void, os::windows::io::AsHandle, ptr::null_mut};

use thiserror::Error;
use windows_sys::{
    Win32::{
        Foundation,
        System::Console::{
            self, ENABLE_EXTENDED_FLAGS, ENABLE_VIRTUAL_TERMINAL_INPUT, ENABLE_WINDOW_INPUT,
            GetStdHandle, INPUT_RECORD, SetConsoleMode,
        },
    },
    core::BOOL,
};

fn main() -> std::process::ExitCode {
    println!("Hello, world!");

    match App::run() {
        Ok(_) => std::process::ExitCode::SUCCESS,
        Err(_) => std::process::ExitCode::FAILURE,
    }
}

struct App;

impl App {
    fn run() -> Result<(), Box<dyn std::error::Error>> {
        init()?;
        Ok(())
    }
}

// escape_sequences ->
//      \x1b[J - clears from the cursor to the end
//      \x1b[0J - same as \x1b[J
//      \x1b[1J - clears upto the cursor

// fn check_bool_return(ret: Foundation::BOOL) -> bool{

// }

#[derive(Debug, Error)]
enum ConsoleError {
    #[error("Could not change Console Mode, error code: [{0}]")]
    ConsoleMode(u32),
}

struct ConsoleState {
    stdin: Foundation::HANDLE,
    stdout: Foundation::HANDLE,
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
}

static mut GLOBAL_CONSOLE_STATE: ConsoleState = ConsoleState {
    stdin: null_mut(),
    stdout: null_mut(),
};

fn init() -> Result<(), ConsoleError> {
    unsafe {
        GLOBAL_CONSOLE_STATE.stdin = GetStdHandle(Console::STD_INPUT_HANDLE);
        GLOBAL_CONSOLE_STATE.stdout = GetStdHandle(Console::STD_OUTPUT_HANDLE);
    }
    todo!()
}

fn check_nonzero_success(ret: BOOL) -> Result<(), ConsoleError> {
    if ret == 0 {
        return Err(ConsoleError::ConsoleMode(get_last_error_code()));
    } else {
        return Ok(());
    }
}

fn enable_raw_mode() -> Result<(), ConsoleError> {
    unsafe {
        check_nonzero_success(SetConsoleMode(
            GLOBAL_CONSOLE_STATE.stdin,
            ENABLE_WINDOW_INPUT | ENABLE_EXTENDED_FLAGS | ENABLE_VIRTUAL_TERMINAL_INPUT,
        ))?;
    }

    Ok(())
}

fn get_last_error_code() -> u32 {
    unsafe { Foundation::GetLastError() }
}

extern "system" fn console_ctrl_handler(_ctrl_type: u32) -> BOOL {
    unsafe {
        // GLOBAL_CONSOLE_STATE.wants_exit = true;
        // windows_sys::Win32::System::IO::CancelIoEx(GLOBAL_CONSOLE_STATE.stdin, null());
    }
    1
}
