extern crate rust_audio_lib; // Introduce the `rust_audio_lib` library crate.
use rust_audio_lib::utils; // Refer to `utils` module

#[test]
fn test_get_default_device_id() {
    assert!(utils::get_default_device_id(utils::Scope::Input).is_ok());
    assert!(utils::get_default_device_id(utils::Scope::Output).is_ok());
}
