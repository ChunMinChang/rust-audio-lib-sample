# Testing

(See the code [here][step4].)

To practice *test-driven development (TDD)* in programming *Rust*, we need to write tests before implementing our functions. Let's look how to write tests with *Cargo*'s built-in framework.

The simplest way to write a test function is to add ```#[test]``` attribute on the line before ```fn``` as follows:

```rust
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

    #[test] // Indicates this is a test function
    fn test_get_abs() {
        assert_eq!(get_abs(0), 0);
        assert_eq!(get_abs(10), 10);
        assert_eq!(get_abs(-10), 10);
    }
}
```

The ```fn test_get_abs()``` here is a private internal function used only for testing. The functions annotated with ```#[test]``` are only built within the *test runner binary*. They are executed when we run ```$ cargo test```:

```
$ cargo test
   Compiling rust_audio_lib v0.1.0 (file:///<path>/<to>/<projects>/rust_audio_lib)
    Finished dev [unoptimized + debuginfo] target(s) in 0.70s
     Running target/debug/deps/rust_audio_lib-e7d8afc4518ed66d

running 1 test
test utils::test_get_abs ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

...
```

If the behavior of one API varies, you can write different tests for it:

```rust
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

    #[test]
    fn test_get_abs_zero() {
        assert_eq!(get_abs(0), 0);
    }

    #[test]
    fn test_get_abs_positive() {
        assert_eq!(get_abs(10), 10);
    }

    #[test]
    fn test_get_abs_negative() {
        assert_eq!(get_abs(-10), 10);
    }
}
```

```
$ cargo test
   Compiling rust_audio_lib v0.1.0 (file:///<path>/<to>/<projects>/rust_audio_lib)
    Finished dev [unoptimized + debuginfo] target(s) in 0.71s
     Running target/debug/deps/rust_audio_lib-e7d8afc4518ed66d

running 3 tests
test utils::test_get_abs_positive ... ok
test utils::test_get_abs_negative ... ok
test utils::test_get_abs_zero ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

...
```

It's overkilled in the above example, but you get the idea about how we can do.

## Threads in Testing
The tests are running in parallel by default. The testing framework will create new threads for each test. When one test thread has died, it's marked as failed. To verify it, we can run tests that operating some shared state:

```rust
mod tests {
    use std::fs;

    #[test]
    fn thread1() {
        let filename = "hello.txt";
        let data = "some data 1";
        fs::write(filename, data).expect("Unable to write file");
        let read = fs::read_to_string(filename).expect("Unable to read file");
        assert_eq!(read, data);
    }

    #[test]
    fn thread2() {
        let filename = "hello.txt";
        let data = "some data 2";
        fs::write(filename, data).expect("Unable to write file");
        let read = fs::read_to_string(filename).expect("Unable to read file");
        assert_eq!(read, data);
    }
}
```

The ```thread1()``` and ```thread2()``` run on their own threads and they both write data into the same file. What will happen if we run the tests? There are 3 possibilities:
- Both tests are passed.
- ```thread1()``` failed since ```thread2()``` changes the data before ```thread1()``` varify it.
- ```thread2()``` failed since ```thread1()``` changes the data before ```thread2()``` varify it.

**Situation 1**: Both tests are passed.

```
$ cargo test
    Finished dev [unoptimized + debuginfo] target(s) in 0.02s
     Running target/debug/deps/rust_audio_lib-e7d8afc4518ed66d

running 2 tests
test tests::thread1 ... ok
test tests::thread2 ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

...
```

**Situation 2**: ```thread1()``` failed since ```thread2()``` changes the data before ```thread1()``` varify it.

```
$ cargo test
    Finished dev [unoptimized + debuginfo] target(s) in 0.02s
     Running target/debug/deps/rust_audio_lib-e7d8afc4518ed66d

running 2 tests
test tests::thread2 ... ok
test tests::thread1 ... FAILED

failures:

---- tests::thread1 stdout ----
        thread 'tests::thread1' panicked at 'assertion failed: `(left == right)`
  left: `"some data 2"`,
 right: `"some data 1"`', src/lib.rs:10:9
note: Run with `RUST_BACKTRACE=1` for a backtrace.


failures:
    tests::thread1

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out

error: test failed, to rerun pass '--lib'
```

**Situation 3**: ```thread2()``` failed since ```thread1()``` changes the data before ```thread2()``` varify it.

```
$ cargo test
    Finished dev [unoptimized + debuginfo] target(s) in 0.02s
     Running target/debug/deps/rust_audio_lib-e7d8afc4518ed66d

running 2 tests
test tests::thread1 ... ok
test tests::thread2 ... FAILED

failures:

---- tests::thread2 stdout ----
        thread 'tests::thread2' panicked at 'assertion failed: `(left == right)`
  left: `"some data 1"`,
 right: `"some data 2"`', src/lib.rs:19:9
note: Run with `RUST_BACKTRACE=1` for a backtrace.


failures:
    tests::thread2

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out

error: test failed, to rerun pass '--lib'
```

If you want to run tests consecutively to avoid the above situation, use ```$ cargo test -- --test-threads=1``` to set the threads number to ```1```. Then we can stably pass the tests in this case like below:

```
$ cargo test -- --test-threads=1
    Finished dev [unoptimized + debuginfo] target(s) in 0.02s
     Running target/debug/deps/rust_audio_lib-e7d8afc4518ed66d

running 2 tests
test tests::thread1 ... ok
test tests::thread2 ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

...
```

However, it would be better if the APIs can run in parallel threads. The tests will be faster.

There are other powerful commands you can use for facilitating the automated testing. See more detail [here][poc].

## Unit Tests

To know if the units of code or APIs work as expected, testing them individually is a pragmatic methodology. *Unit testing* examines every unit of library separately, no matter it's public or private.

Besides the tests we used above, we can also put them into a test module. Both are *unit tests* techniques in *Cargo*. Look the following example:

*src/lib.rs*:
```rust
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
    mod tests { // A private internal submodule in utils
        use super::*; // To use the functions in utils

        #[test] // Indicates this is a test function
        fn test_get_abs() {
            assert_eq!(get_abs(0), 0);
            assert_eq!(get_abs(10), 10);
            assert_eq!(get_abs(-10), 10);
        }
    }
}
```

We could put all the tests into a ```mod tests```. The ```#[cfg(test)]``` indicates ```mod tests``` is only built when running ```cargo test```. The ```#[test]``` works in the same way as we mentioned above. It indicates ```fn test_get_abs()``` is a test function and is only built within the *test runner binary*.

*src/main.rs*:
```rust
extern crate rust_audio_lib; // Introduce the `rust_audio_lib` library crate
use rust_audio_lib::utils; // Refer to `utils` module
// use rust_audio_lib::utils::double_abs; // Refer to `double_abs` function

fn main() {
    let x: i32 = -50;
    let abs: i32 = utils::double_abs(x);
    // let abs: i32 = double_abs(x);
    println!("Double of |{}| is {}", x, abs);
}
```

The usage of ```rust_audio_lib``` crate in *src/main.rs* doesn't change. All we need just to introduce the ```rust_audio_lib``` library crate.

## Integration Tests

In addition to *unit tests*, *Cargo* also provides a built-in *integration tests* framework. The *unit testing* makes sure the units of code or APIs work as expected indivisually, while *integration testing* checks the units of code can work together. *Integration testing* groups the units of code into larger aggregates and verifies them work as what we planned. It's **entired external** to the library, so we can use the public APIs **exactly as what external code uses**.

To create an *integration test*, the first step is to create a *tests* directory. Next, we create a test file named *integration.rs* and put the following code into there:

*tests/integration.rs*

```rust
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
```

The usage of ```rust_audio_lib``` library crate is just like what we use in *src/main.rs*. We can only call **public** APIs. Another different thing is that we **don't** need to mark ```#[cfg(test)]``` attributes in the files under *tests* directory. All the files there are compiled only when we run ```cargo test```.

## References
- [Writing Automated Tests][testing]
- [Writing Tests][write]
- [Running Tests][run]
- [Test Organization][org]
- [Conditional compilation][cfg]

[testing]: https://doc.rust-lang.org/book/second-edition/ch11-00-testing.html "Writing Automated Tests"
[write]: https://doc.rust-lang.org/book/second-edition/ch11-01-writing-tests.html "How to Writing Tests"
[run]: https://doc.rust-lang.org/book/second-edition/ch11-02-running-tests.html "Controlling How Tests Are Run"
[poc]: https://doc.rust-lang.org/book/second-edition/ch11-02-running-tests.html#running-tests-in-parallel-or-consecutively "Running Tests in Parallel or Consecutively"
[org]: https://doc.rust-lang.org/book/second-edition/ch11-03-test-organization.html "Test Organization"
[cfg]: https://doc.rust-lang.org/reference/attributes.html#conditional-compilation "Conditional compilation"

[step4]: https://github.com/ChunMinChang/rust-audio-lib-sample/tree/6a0cae23b79e2fcaa5c93f949b818439c4c0755c/rust_audio_lib "Code for step4"