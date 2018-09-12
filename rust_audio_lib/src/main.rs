extern crate rust_audio_lib;
use rust_audio_lib::utils; // Refer to `utils` module

fn show_result(scope: &utils::Scope) {
    let side = if scope == &utils::Scope::Input {
        "input"
    } else {
        "output"
    };
    match utils::get_default_device_id(&scope) {
        Ok(id) => {
            println!("default {} device id: {}", side, id);
        }
        Err(error) => {
            println!("Failed to get {} device id. Error: {}", side, error);
        }
    }
}

fn main() {
    show_result(&utils::Scope::Input);
    show_result(&utils::Scope::Output);
}
