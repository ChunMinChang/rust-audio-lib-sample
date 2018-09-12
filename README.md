# rust-audio-lib-sample

The goal for these posts is to create a *Rust* library based on *CoreAudio* framework on *OS X*. The topics covered in this series are:
- How to call native *C* APIs from *Rust*
- How to call platform-dependent *C* APIs from *Rust*
- How to write modules in *Rust*
- How to write automated tests in *Cargo*
- How to write a build script in *Cargo*
- How to call Rust library from *C*

## Sample Code
The sample code for each step can be found in the beginning of each post. They are different commits on the [code][code] branch.

## TO-DO
- Find out why we need ```libresolv``` in compiled rust executable files
- Use rustdoc
- Use _*-sys_ modules on platform-dependent libraries
  - [Make a *-sys crate][kornel]
  - Use [bindgen][bindgen]

## Note
These posts may be changed anytime. These posts are actually my learning notes for programming Rust in the first time. I am very new to *Rust* and I just read the parts I thoght it may be helpful. Once I have better understanding of *Rust*, the posts are very likely to be rewritten. Please feel free to let me know if you find anything wrong.

[kornel]: https://kornel.ski/rust-sys-crate "Making a *-sys crate"
[bindgen]: https://github.com/rust-lang-nursery/ "rust-bindgen"

[code]: https://github.com/ChunMinChang/rust-audio-lib-sample/tree/code-v0 "Sample Code"