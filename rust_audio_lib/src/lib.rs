pub mod utils {
    #[derive(PartialEq)] // Enable comparison.
    pub enum Scope {
        Input,
        Output,
    }

    pub fn get_default_device_id(scope: Scope) -> Result<u32, i32> {
        let id: u32 = if scope == Scope::Input { 1 } else { 2 };
        get_property_data(id)
    }

    // Mock API
    fn get_property_data(id: u32) -> Result<u32, i32> {
        Err(id as i32)
    }

    #[cfg(test)] // Indicates this is only included when running `cargo test`
    mod tests { // A private internal submodule in utils
        use super::*; // To use the functions in utils

        #[test] // Built only within `cargo test`.
        fn test_get_property_data_invalid_id() {
            let invalid_id: u32 = 0;
            assert!(get_property_data(invalid_id).is_err());
        }

        #[test] // Built only within `cargo test`.
        fn test_get_property_data() {
            let id: u32 = 10;
            assert!(get_property_data(id).is_ok());
        }
    }
}