extern crate rust_audio_lib; // Introduce the `rust_audio_lib` library crate.
use rust_audio_lib::utils; // Refer to `utils` module
// use rust_audio_lib::utils::double_abs; // Refer to `double_abs` function

#[test]
fn test_double_abs() {
    assert_eq!(0, utils::double_abs(0));
    assert_eq!(10, utils::double_abs(-5));
    assert_eq!(20, utils::double_abs(10));
    // assert_eq!(0, double_abs(0));
    // assert_eq!(10, double_abs(-5));
    // assert_eq!(20, double_abs(10));
}