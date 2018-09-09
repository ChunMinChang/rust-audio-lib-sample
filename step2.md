# Calling Native C APIs from Rust

(See the code [here][step2].)

## Foreign Function Interface
> A foreign function interface (FFI) is a mechanism by which a program written in one programming language can call routines or make use of services written in another.

Rust has a keyword ```extern``` that provides *FFI* mechanism to call those functions written in another language. We could can a *C* function like the following:

```rust
extern "C" {
    fn abs(input: i32) -> i32;
}

fn main() {
    unsafe {
        println!("Absolute value of -3 according to C: {}", abs(-3));
    }
}
```

Functions within ```extern``` block are always called in a ```unsafe``` block. Since *Rust* has no way to guarantee the memory safety of the external functions written in other language, we need to use ```unsafe``` to label the dangerous parts and convince *Rust compiler* to trust we know what exactly we are doing. See [here][unsafe] for more details.

Applying what we've learned above, we can rewrite *src/main.rs* into the following code:
```rust
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
```

To check if it works, we just need to build and run it.
```
$ cargo clean
$ cargo build
   Compiling rust_audio_lib v0.1.0 (file:///<path>/<to>/<projects>/rust_audio_lib)
    Finished dev [unoptimized + debuginfo] target(s) in 0.44s
$ cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.02s
     Running `target/debug/rust_audio_lib`
abs of -50 is 50
```

## Why can *Rust* call *C* functions without specifying any linked libraries?

### Shared Object Dependencies in *C*

Let's look a *C* example: *ffi.c*:

```c
#include <math.h>   // sin
#include <stdio.h>  // printf
#include <stdlib.h> // abs

int main() {
  const double PI = 3.14159265358979323846;
  double degree = 30.0;
  printf("Absolute value of -3: %d\n", abs(-3));
  printf("The sine of %f degree: %lf\n", degree, sin(degree * PI / 180));
  return 0;
}
```

The above code can be compiled by *gcc*:

```
$ gcc -o ffi ffi.c
$ ./ffi
Absolute value of -3: 3
The sine of 30.000000 degree: 0.500000
```

To check what libraries it links, we can use [```otool -L```][otool] (use [```ldd```][ldd] if you're on linux) to show the shared object dependencies:

```
$ man otool
```

```
LLVM-OTOOL(1)

NAME
       llvm-otool - the temporary command line shim for otool to objdump command line translation
...
SPECIFIC TRANSLATIONS OF OPTIONS
...
       -l     The objdump(1) option to display the load commands is  -private-headers  which  also  always  displays  the  Mach
              header.

       -L     The objdump(1) option to display the names and version numbers of the shared libraries that the object file uses,
              as well as the shared library ID if the file is a shared library is -dylibs-used.
...
```

```
$ otool -L ffi
ffi:
        /usr/lib/libSystem.B.dylib (compatibility version 1.0.0, current version 1252.50.4)
```

The [*libSystem*][libsys] is the system library on *Mac OS*. It includes [```libc```][libc], the standard *C* library. The headers we used: ```<math.h>```, ```<stdio.h>``` and ```<stdlib.h>``` are all contained in it. That's why we can use those functions.

Now, let's see if the *C* code above works without headers:

```c
int main() {
  const double PI = 3.14159265358979323846;
  double degree = 30.0;
  printf("Absolute value of -3: %d\n", abs(-3));
  printf("The sine of %f degree: %lf\n", degree, sin(degree * PI / 180));
  return 0;
}
```

Surprisingly, it can be compiled successfully, just with *warnings*:

```
$ gcc -o ffi ffi.c
ffi.c:4:3: warning: implicitly declaring library function 'printf' with type 'int (const char *, ...)'
      [-Wimplicit-function-declaration]
  printf("Absolute value of -3: %d\n", abs(-3));
  ^
ffi.c:4:3: note: include the header <stdio.h> or explicitly provide a declaration for 'printf'
ffi.c:4:40: warning: implicitly declaring library function 'abs' with type 'int (int)' [-Wimplicit-function-declaration]
  printf("Absolute value of -3: %d\n", abs(-3));
                                       ^
ffi.c:4:40: note: include the header <stdlib.h> or explicitly provide a declaration for 'abs'
ffi.c:5:50: warning: implicitly declaring library function 'sin' with type 'double (double)'
      [-Wimplicit-function-declaration]
  printf("The sine of %f degree: %lf\n", degree, sin(degree * PI / 180));
                                                 ^
ffi.c:5:50: note: include the header <math.h> or explicitly provide a declaration for 'sin'
3 warnings generated.
$ ./ffi
Absolute value of -3: 3
The sine of 30.000000 degree: 0.500000
```

It works in the same way as the code including headers.

```
$ otool -L ffi
ffi:
        /usr/lib/libSystem.B.dylib (compatibility version 1.0.0, current version 1252.50.4)
```

And the library it links is same too. In fact, if we run ```$ otool -l ffi``` to get the load commands, we will see the results are same no matter we include the headers or not. That's interesting. Let's do another experiment on linking. Create a *empty.c*:

```c
int main() {
  return 0;
}
```

and see it's dependencies:

```
$ gcc -o empty empty.c
$ ./empty
$ otool -L empty
empty:
	/usr/lib/libSystem.B.dylib (compatibility version 1.0.0, current version 1252.50.4)
```

The compiled executable file still has a dependency to [*libSystem*][libsys], even without including any headers. From the experiment, we know the *gcc* on *Mac OS* will link to *libSystem* no matter we define headers or not. That's why the ```abs``` and ```sin``` in *ffi.c* can work without including headers.


### Shared Object Dependencies in *Rust*

Next, let's look back to see a *rust* example: *ffi.rs*:

```rust
extern "C" {
    fn abs(x: i32) -> i32; // stdlib.h
    fn sin(x: f64) -> f64; // math.h
}

fn main() {
    const PI: f64 = 3.14159265358979323846;
    let degree: f64 = 30.0;
    unsafe {
        println!("Absolute value of -3 according to C: {}", abs(-3));
        println!("The sine of {} degree according to C: {}", degree,
                 sin(degree * PI / 180.0));
    }
}
```

To get the *link* information, we use an experimental *rustc* option: ```print-link-args``` with ```-Z``` flag. It works only in **nightly** *rustc* at this moment. If you don't have nightly *rustc*, you can install nightly toolchain by ```$ rustup install nightly``` and switch default toolchain to it by ```$ rustup default nightly```. Otherwise, you will get an error like: [```error: the option `Z` is only accepted on the nightly compiler```][stkov].

If we compile *ffi.rs* by ```$ rustc -Z print-link-args ffi.rs```:

```
$ rustc -Z print-link-args ffi.rs
"cc" "-m64" "-L" "/<path>/<user_name>/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib" "ffi.ffi0.rcgu.o" "ffi.ffi1.rcgu.o" "ffi.ffi2.rcgu.o" "ffi.ffi3.rcgu.o" "ffi.ffi4.rcgu.o" "ffi.ffi5.rcgu.o" "-o" "ffi" "ffi.crate.allocator.rcgu.o" "-Wl,-dead_strip" "-nodefaultlibs" "-L" "/<path>/<user_name>/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib" "/<path>/<user_name>/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/libstd-5426f7d812c33791.rlib" "/<path>/<user_name>/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/libpanic_unwind-2fc89f7407c2fdf1.rlib" "/<path>/<user_name>/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/liballoc_jemalloc-7973437d75e41551.rlib" "/<path>/<user_name>/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/libunwind-84afc50178e78273.rlib" "/<path>/<user_name>/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/liballoc_system-f2e8cf90553ff84f.rlib" "/<path>/<user_name>/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/liblibc-9afba5cb13b6de54.rlib" "/<path>/<user_name>/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/liballoc-8e2dd1e6e8b3dee0.rlib" "/<path>/<user_name>/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/libcore-7e32fa628631e8e6.rlib" "/<path>/<user_name>/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/libcompiler_builtins-a4054d9b44401c28.rlib" "-lSystem" "-lresolv" "-lpthread" "-lc" "-lm"
```

we will get lots of link information here. The flags ```-L``` and ```-l``` here probably work like [what *gcc* does][gcclinking]. From ```rustc --help```:

```
$ rustc --help
Usage: rustc [OPTIONS] INPUT

Options:
    -h, --help          Display this message
        --cfg SPEC      Configure the compilation environment
    -L [KIND=]PATH      Add a directory to the library search path. The
                        optional KIND can be one of dependency, crate, native,
                        framework or all (the default).
    -l [KIND=]NAME      Link the generated crate(s) to the specified native
                        library NAME. The optional KIND can be one of static,
                        dylib, or framework. If omitted, dylib is assumed.
    ...
```

we can know what ```-L``` and ```-l``` mean:
- ```-L```: Add a directory to the library search path.
- ```-l```: Link the generated crate(s) to the specified native library NAME.

The ```-L``` is set to ```libstd```, ```libpanic_unwind```, ..., ```liblibc```, ```libcompiler```. I guess it means *rustc* will search all its [internal libraries][rust] by default and link the functions if it's necessary.

The ```-l``` is set to ```-lSystem```, ```-lresolv```, ..., ```-lm```. The ```-lSystem``` may indicate to the [*libSystem*][libsys], which is the system library linked to the ```ffi.c``` above. Let's check the *shared object dependencies* for *ffi.rs*:

```
$ otool -L ffi
ffi:
        /usr/lib/libSystem.B.dylib (compatibility version 1.0.0, current version 1252.50.4)
        /usr/lib/libresolv.9.dylib (compatibility version 1.0.0, current version 1.0.0)
```

It's obvious that *ffi.rs* also depends on *libSystem* as what *ffi.c* does. If we run ```./ffi```, we will find the result is different from what we have in *C* code:

```
$ ./ffi
Absolute value of -3 according to C: 3
The sine of 30 degree according to C: 0.49999999999999994
```

We can use ```$ otool -l ffi``` to get the load commands to compare. The result is longer than what we get from *C* code. (TODO: Maybe that is because we have a *libresolv* here ? why we have a *libresolv* here?)

In fact, *Rust* will link to *libSystem* automatically no matter we use ```extern``` or not. Let's see an example *empty.rs*:
```rust
fn main() {}
```

```
$ rustc -Z print-link-args empty.rs
"cc" "-m64" "-L" "/Users/cchang/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib" "empty.empty0.rcgu.o" "empty.empty1.rcgu.o" "empty.empty2.rcgu.o" "empty.empty3.rcgu.o" "empty.empty4.rcgu.o" "-o" "empty" "empty.crate.allocator.rcgu.o" "-Wl,-dead_strip" "-nodefaultlibs" "-L" "/Users/cchang/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib" "/Users/cchang/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/libstd-5426f7d812c33791.rlib" "/Users/cchang/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/libpanic_unwind-2fc89f7407c2fdf1.rlib" "/Users/cchang/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/liballoc_jemalloc-7973437d75e41551.rlib" "/Users/cchang/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/libunwind-84afc50178e78273.rlib" "/Users/cchang/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/liballoc_system-f2e8cf90553ff84f.rlib" "/Users/cchang/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/liblibc-9afba5cb13b6de54.rlib" "/Users/cchang/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/liballoc-8e2dd1e6e8b3dee0.rlib" "/Users/cchang/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/libcore-7e32fa628631e8e6.rlib" "/Users/cchang/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/libcompiler_builtins-a4054d9b44401c28.rlib" "-lSystem" "-lresolv" "-lpthread" "-lc" "-lm"
$ ./empty
$ otool -L empty
empty:
	/usr/lib/libSystem.B.dylib (compatibility version 1.0.0, current version 1252.50.4)
	/usr/lib/libresolv.9.dylib (compatibility version 1.0.0, current version 1.0.0)
```

It's clear that the *empty* still has a dependency on *libSystem*, even there is no ```extern``` block. That's why we don't need to specify what library we need to link. *Rust* will link to *libSystem* by default. This is pretty similar to what *gcc* on *Mac OS* does.


### Conclusion
The *Rust* program will link to [*libSystem*][libsys] by default. The [*libSystem*][libsys] is the system library on *Mac OS*. It includes the standard *C* library: [```libc```][libc], so you can use all the *C* functions defined in [```libc```][libc] without any link settings.

To link to other native libraries that are not included in [```libc```][libc], we will need to set an attribute like ```#[link(name = "...")``` to specify what library we need. We will learn this in later chapter. Stay tuned!

## Is there any overhead to Rust FFI?
Short answer is no. See [here][overhead].

## Referecnes
- [Wiki: Foreign function interface][wiki]
- [Foreign Function Interface][ffi]
- [Calling C from Rust][jvns]
- [Using ```extern``` Functions to Call External Code][unsafe]
- Display shared object dependencies
  - [```otool -L```][otool]
  - [```ldd```][ldd]
- [The System Library: libSystem][libsys]
- [C standard library][libc]
- [rust-lang/rust][rust]

[wiki]: https://en.wikipedia.org/wiki/Foreign_function_interface "Foreign function interface"
[ffi]: https://doc.rust-lang.org/book/ffi.html "FFI"
[jvns]: https://jvns.ca/blog/2016/01/18/calling-c-from-rust/ "Calling C from Rust"
[unsafe]: https://doc.rust-lang.org/book/second-edition/ch19-01-unsafe-rust.html#using-extern-functions-to-call-external-code "Using extern Functions to Call External Code"

[otool]: http://www.manpagez.com/man/1/otool/ "man page: otool"
[ldd]: http://man7.org/linux/man-pages/man1/ldd.1.html "man page: ldd"
[gcclinking]: https://gcc.gnu.org/onlinedocs/gcc/Link-Options.html "GCC: Options for Linking"

[stkov]: https://stackoverflow.com/questions/48675235/error-the-option-z-is-only-accepted-on-the-nightly-compiler "error: the option `Z` is only accepted on the nightly compiler"

[libsys]: https://docstore.mik.ua/orelly/unix3/mac/ch05_02.htm "The System Library: libSystem"
[libc]: https://en.wikipedia.org/wiki/C_standard_library "C standard library"

[rust]: https://github.com/rust-lang/rust/tree/master/src "rust-lang/rust"

[overhead]: https://www.reddit.com/r/rust/comments/4sbmco/is_there_any_overhead_to_rust_ffi/ "Is there any overhead to Rust FFI?"

[step2]: https://github.com/ChunMinChang/rust-audio-lib-sample/tree/9c91ac92df10edcd606e89d44447827695eca222/rust_audio_lib "Code for step2"