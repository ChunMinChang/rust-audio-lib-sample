# Creating a Project with Cargo

First, we creat a rust project with cargo
```
$ cargo new rust_audio_lib
    Created binary (application) `rust_audio_lib` project
```
and then check it's ok.
```
$ cd rust_audio_lib/
$ cat src/main.rs
fn main() {
    println!("Hello, world!");
}
$ cargo build
   Compiling rust_audio_lib v0.1.0 (file:///<path>/<to>/<projects>/rust_audio_lib)
    Finished dev [unoptimized + debuginfo] target(s) in 0.41s
$ cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.02s
     Running `target/debug/rust_audio_lib`
Hello, world!
```

Everything is ok. Great, let's start our journey.

## Reference
- [Hello, Cargo!][cargo]

[cargo]: https://doc.rust-lang.org/book/second-edition/ch01-03-hello-cargo.html "Hello, Cargo!"