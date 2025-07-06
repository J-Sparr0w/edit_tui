use editui::editor::Editor;
use std::process::ExitCode;

fn main() -> std::process::ExitCode {
    let mut editor = Editor::new();
    editor.run();
    ExitCode::SUCCESS
}
