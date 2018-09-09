extern crate rust_audio_lib; // Introduce the `rust_audio_lib` library crate
use rust_audio_lib::utils; // Refer to `utils` module
// use rust_audio_lib::utils::double_abs; // Refer to `double_abs` function

fn main() {
    let x: i32 = -50;
    let abs: i32 = utils::double_abs(x);
    // let abs: i32 = double_abs(x);
    println!("Double of |{}| is {}", x, abs);
}