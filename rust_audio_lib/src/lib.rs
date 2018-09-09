pub mod utils {
    extern "C" {
      fn abs(input: i32) -> i32;
    }

    // A wrapper for native C API.
    fn get_abs(x: i32) -> i32 {
        unsafe {
            abs(x)
        }
    }

    pub fn double_abs(x: i32) -> i32 {
        get_abs(x) * 2
    }

    #[cfg(test)] // Indicates this is only included when running `cargo test`
    mod tests { // (Internal) Submodule of utils
        use super::*;

        #[test] // Indicates this is a test function
        fn test_get_abs() {
            assert_eq!(get_abs(0), 0);
            assert_eq!(get_abs(10), 10);
            assert_eq!(get_abs(-10), 10);
        }
    }
}