extern "C" {
    fn abs(input: i32) -> i32;
}

// A wrapper for native C API.
fn get_abs(x: i32) -> i32 {
    unsafe {
        abs(x)
    }
}

fn main() {
    let x: i32 = -50;
    let abs: i32 = get_abs(x);
    println!("abs of {} is {}", x, abs);
}