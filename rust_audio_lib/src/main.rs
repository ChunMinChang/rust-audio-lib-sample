extern crate rust_audio_lib; // Introduce the `rust_audio_lib` library crate.
use rust_audio_lib::utils; // Refer to `utils` module.
// use rust_audio_lib::utils::get_abs; // Refer to `get_abs` function in `utils` module.

fn main() {
    let x: i32 = -50;
    let abs: i32 = utils::get_abs(x);
    // let abs: i32 = get_abs(x);
    println!("abs of {} is {}", x, abs);
}