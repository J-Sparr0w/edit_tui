use editui::editor::Editor;
use std::process::ExitCode;

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() -> std::process::ExitCode {
    let mut editor = Editor::new();
    match editor.run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}
