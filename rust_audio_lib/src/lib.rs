pub mod utils {
    extern "C" {
      fn abs(input: i32) -> i32;
    }

    // A wrapper for native C API.
    pub fn get_abs(x: i32) -> i32 {
        unsafe {
            abs(x)
        }
    }
}