# Moving APIs to Modules

(See the code [here][step3].)

With the APIs grow, we need a better way to organize code instead of putting lots of functions in one file. *Rust* provides a *module* system that can structure code in a systematical manner. See [here][modules] for more detail.

## Creating a Library Crate
The first step to create a library crate is to new a file named *lib.rs* under our *src* directory. Then, put the code into *src/lib.rs* as follows:

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
}
```

The keyword ```pub``` is to make the module and function **public**, so they can be called outside.
Next, we need to remove the moved functions from in *src/main.rs* and add the ```extern``` crate command to introduce the *rust_audio_lib* library crate. *rust_audio_lib* is the package name defined in *Cargo.toml*.

```rust
extern crate rust_audio_lib; // Introduce the `rust_audio_lib` library crate.

fn main() {
    let x: i32 = -50;
    let abs: i32 = rust_audio_lib::utils::get_abs(x);
    println!("abs of {} is {}", x, abs);
}
```

Now, there are two crates in our package:
- binary crate: root file is *src/main.rs*
- library crate: root file is *src/lib.rs*

This is a common pattern. Most functions are located in a library crate, and they are called in a binary crate. By separating the code into different crates, functions in library crate can be used by other projects.

## Namespace

In above example, we need a long qualified name to call a function(```rust_audio_lib::utils::get_abs(x)```). If we have modules inside modules, then it may become quite lengthy. To shorten the function call, we can use the keyword ```use```. See the example below:

*src/main.rs*
```rust
extern crate rust_audio_lib; // Introduce the `rust_audio_lib` library crate.
use rust_audio_lib::utils; // Refer to `utils` module.
// use rust_audio_lib::utils::get_abs; // Refer to `get_abs` function in `utils` module.

fn main() {
    let x: i32 = -50;
    let abs: i32 = utils::get_abs(x);
    // let abs: i32 = get_abs(x);
    println!("abs of {} is {}", x, abs);
}
```

The line ```use rust_audio_lib::utils``` means ```utils``` is referred to the ```utils``` module directly. Similarly, you can directly refer to ```get_abs``` by changing the line into ```use rust_audio_lib::utils::get_abs```.

Finally, let's run our project to make sure it works:
```
$ cargo build
   Compiling rust_audio_lib v0.1.0 (file:///<path>/<to>/<projects>/rust_audio_lib)
    Finished dev [unoptimized + debuginfo] target(s) in 2.34s
$ cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.03s
     Running `target/debug/rust_audio_lib`
abs of -50 is 50
```


## References
- [Modules][modules]
- [Controlling Visibility with ```pub```][visibility]
- [Bringing Names into Scope][namescope]

[modules]: https://doc.rust-lang.org/book/second-edition/ch07-00-modules.html "Using Modules to Reuse and Organize Code"
[visibility]: https://doc.rust-lang.org/book/second-edition/ch07-02-controlling-visibility-with-pub.html "Controlling Visibility with pub"
[namescope]: https://doc.rust-lang.org/book/second-edition/ch07-03-importing-names-with-use.html "Referring to Names in Different Modules"

[step3]: https://github.com/ChunMinChang/rust-audio-lib-sample/tree/e87cfaecc8c37013ef60d47285f2c6268d41c066/rust_audio_lib "Code for step 3"