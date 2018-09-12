# Calling Rust APIs in C

(See the code [here][step9-static-lib].)

To build a library that can be called in *C*, we need to create a *C*-compatible interface. The only one API ```get_default_device_id``` in our library returns a ```Result<...>```, which is not what *C* can understand. Hence, we need to create an interface like:

```c
Error get_default_device_id(scope s, device_id* id);
```

- The ```Error``` indicates the calling is successful or not. If the calling fails, it should be able to state what error is.
- The ```scope``` indicates it's input or output device
- The ```device_id``` is a pointer pointing to an address where the value of *device id* will be stored in.

To do that, we can rewrite *src/lib.rs* into the following code:

```rust
...

pub mod utils {
    ...

    #[repr(C)]
    #[derive(PartialEq)] // Enable comparison
    pub enum Scope {
        Input,
        Output,
    }

    #[repr(C)]
    #[derive(Debug, PartialEq)] // Using Debug for std::fmt::Debug
    pub enum Error {
        Ok,
        NoDevice,
        InvalidParameters,
    }

    ...
}

#[no_mangle] // Tell the Rust compiler not to mangle the name of this function.
pub extern fn get_default_device_id(
    scope: utils::Scope,
    id: *mut utils::DeviceId
) -> utils::Error {
    if id.is_null() {
        return utils::Error::InvalidParameters;
    }
    match utils::get_default_device_id(scope) {
        Ok(device_id) => {
            unsafe { *id = device_id };
            utils::Error::Ok
        },
        Err(error) => { error }
    }
}
```

To add the *C*-compatible interface ```fn get_default_device_id(...) -> utils::Error```, we add a ```Ok``` into ```enum Error``` so *C* user can know the calling is successful. Since the ```enum Error``` and ```enum Scope``` will be exposed to *C*, so we add ```#[repr(C)]``` to make sure their data layout is same as what *enum* in *C* is. On the line of the interface API ```get_default_device_id```, the [```#[no_mangle]```][unsafe] is used to prevent the function name from being modified when it's compiled, so it can be found in the library.

## Building Libraries

We can build our library into different types by specifying what ```crate-type``` is. The common options are *dylib*, *cdylib*, *staticlib*, *rlib*.
- *dylib*: Build a *dynamic* library that will be a dependency of other code.
- *cdylib*: Build a *dynamic* library that will be loaded from other languages.
- *staticlib*: Build a *static* library that will be compiled with other languages.
- *rlib*: Build a *Rust* library that can produce statically linked executables.

For more detail about ```crate-type```, please see [The Rust Reference: Linkage][linkage].

To build the library with specified type, we need to add ```lib``` settings in *Cargo.toml*:
```toml

...

[lib]
name = "<YOUR_LIBRARY_NAME>"
crate-type = ["<TYPE_OF_LIBRARY>"] # dylib, cdylib, staticlib, rib, ...

...

```

### Creating a [Shared Library][shrlib]

(See the code [here][step9-dynamic-lib].)

To build a **dynamic** library, we need to set ```cdylib``` to ```crate-type``` in *Cargo.toml* as follow:

```toml

...

[lib]
name = "rust_audio_lib"
# cdylib: For dynamic library to be loaded from another language.
crate-type = ["cdylib"] # Run with `$ cargo build --lib`
# To make `$ cargo build` work, using `crate-type = ["cdylib", "rlib"]` instead.
# `rlib` is for static Rust library to be used internally (src/main.rs here).

...

```

Then, run ```$ cargo build --lib``` to build the library only, without *bin*.

```
$ cargo clean
$ cargo build --lib
   Compiling rust_audio_lib v0.1.0 (file:///<path>/<to>/<projects>/rust_audio_lib)
    Finished dev [unoptimized + debuginfo] target(s) in 1.32s
```

There is a new *librust_audio_lib.dylib* file created, under *target/debug* of the package directory, after running ```$ cargo build --lib```.

```
/<path>/<to>/<projects>/rust_audio_lib/target/debug/librust_audio_lib.dylib
```

We can use [```nm```][nm] tool to check if ```get_default_device_id``` is included in *librust_audio_lib.dylib*:

<!-- ```
$ nm target/debug/librust_audio_lib.dylib | grep get_default_device_id
00000000000013a0 t __ZN14rust_audio_lib5utils21get_default_device_id17h78721057d05d8cf4E
0000000000001cc0 T _get_default_device_id
$ c++filt __ZN14rust_audio_lib5utils21get_default_device_id17h78721057d05d8cf4E
rust_audio_lib::utils::get_default_device_id::h78721057d05d8cf4
```
or
```
$ nm --demangle target/debug/librust_audio_lib.dylib | grep get_default_device_id
00000000000013a0 t rust_audio_lib::utils::get_default_device_id::h78721057d05d8cf4
0000000000001cc0 T _get_default_device_id
``` -->

```
$ nm -gU target/debug/librust_audio_lib.dylib | grep get_default_device_id
0000000000001ea0 T _get_default_device_id
```

```
$ man nm
```

```
NM(1)                                                                                                                  NM(1)

NAME
       nm - display name list (symbol table)
       ...
       ...
DESCRIPTION
       ...
       -g     Display only global (external) symbols.
       ...
       -U     Don't display undefined symbols.
       ...
```

From ```$ man nm```, we can know ```-gU``` is to find global (external) defined symbols. The ```T``` before ```_get_default_device_id``` means it's a [global text symbol][nm]. The ```T _get_default_device_id``` means ```_get_default_device_id``` is a global defined symbols in [text/code section][codeseg]. In fact, we can replace ```man -gU``` by ```nm --extern-only --defined-only```.

After confirming the function is exposed, we could create a *C* file to test it. New a file name *get_default_device_id.c* under *src* and put the following code into *src/get_default_device_id.c*:

```c
#include <stdbool.h>  // for bool
#include <stdio.h>    // for printf
#include <stdint.h>   // for uint32_t

// Interface with rust_audio_lib
typedef enum {
  Input,
  Output,
} Scope; // Map to rust_audio_lib::utils::Scope

typedef enum {
  Ok,
  No_Device,
  Invalid_Parameters,
} Error; // Map to rust_audio_lib::utils::Error

const unsigned int TOTAL_ERRORS = Invalid_Parameters + 1;

typedef uint32_t device_id; // Map to rust_audio_lib::utils::DeviceId

extern Error get_default_device_id(Scope scope, device_id* id);

bool valid(Error e) {
  return e == OK;
}

const char* error_message(Error e) {
  const char* messages[TOTAL_ERRORS] = {
    "no error",
    "device not found",
    "invalid parameters"
  };
  return messages[e];
}

void show_result(Scope scope) {
  device_id id;
  const char* side = scope == Input ? "input" : "output";
  Error error = get_default_device_id(scope, &id);
  if (valid(error)) {
    printf("default %s device id: %d\n", side, id);
  } else {
    printf("Error on getting %s device: %s\n", side, error_message(error));
  }
}

int main() {
  show_result(Input);
  show_result(Output);
  return 0;
}
```

To link to *librust_audio_lib* with *src/get_default_device_id.c*, we can compile them by [*gcc* with ```-L``` and ```-l``` flags][gcclinking]:

```
$ gcc -o get_default_device_id src/get_default_device_id.c -L target/debug/ -lrust_audio_lib
$ ./get_default_device_id
default input device id: 39
default output device id: 39
```

In the similary way, we can create a *C++* file to call *librust_audio_lib*.
As what ```#[no_mangle]``` does in *Rust*, to call the API in *librust_audio_lib* in *C++*, we need to use [```extern "C"```][externc] to prevent the function-name in *C++* from being mangled. Create A file named *src/get_default_device_id.cpp* as follow:

```cpp
#include <iostream> // for std::cout, std::endl
#include <stdint.h> // for uint32_t

// Interface with rust_audio_lib
typedef enum {
  Input,
  Output,
} Scope; // Map to rust_audio_lib::utils::Scope

typedef enum {
  Ok,
  No_Device,
  Invalid_Parameters,
} Error; // Map to rust_audio_lib::utils::Error

const unsigned int TOTAL_ERRORS = Invalid_Parameters + 1;

typedef uint32_t device_id; // Map to rust_audio_lib::utils::DeviceId

extern "C" Error get_default_device_id(Scope scope, device_id* id);

bool valid(Error e) {
  return e == OK;
}

const char* error_message(Error e) {
  const char* messages[TOTAL_ERRORS] = {
    "no error",
    "device not found",
    "invalid parameters"
  };
  return messages[e];
}

void show_result(Scope scope) {
  device_id id;
  const char* side = scope == Input ? "input" : "output";
  Error error = get_default_device_id(scope, &id);
  if (valid(error)) {
    std::cout << "default " << side << " device id: " << id << std::endl;
  } else {
    std::cout << "Error on getting " << side << " device: " << error_message(error) << std::endl;
  }
}

int main() {
  show_result(Input);
  show_result(Output);
  return 0;
}
```

It can be compiled by *g++* with ```-l``` and ```-L``` flags, as what we does for *src/get_default_device_id.c*:

```
$ g++ -o get_default_device_id src/get_default_device_id.cpp -L target/debug/ -lrust_audio_lib
$ ./get_default_device_id
default input device id: 39
default output device id: 39
```

A better way to organize code is to create an interface with *librust_audio_lib*. We can create a header file *src/rust_audio_lib.h* as follow:

```c
#if !defined(RUST_AUDIO_LIB)
#define RUST_AUDIO_LIB

#include <stdint.h> // for uint32_t

// Interface with rust_audio_lib
typedef enum {
  Input,
  Output,
} Scope; // Map to rust_audio_lib::utils::Scope

typedef enum {
  Ok,
  No_Device,
  Invalid_Parameters,
} Error; // Map to rust_audio_lib::utils::Error

const unsigned int TOTAL_ERRORS = Invalid_Parameters + 1;

typedef uint32_t device_id; // Map to rust_audio_lib::utils::DeviceId

#if defined(__cplusplus)
extern "C" {
#endif

extern Error get_default_device_id(Scope scope, device_id* id);

#if defined(__cplusplus)
}
#endif

// Utilities
bool valid(Error e) {
  return e == Ok;
}

const char* error_message(Error e) {
  const char* messages[TOTAL_ERRORS] = {
    "no error",
    "device not found",
    "invalid parameters"
  };
  return messages[e];
}

#endif /* RUST_AUDIO_LIB */
```

and then include *src/rust_audio_lib.h* in *src/get_default_device_id.c*:

```c
#include <stdbool.h>  // for bool
#include <stdio.h>    // for printf
#include "rust_audio_lib.h"

void show_result(Scope scope) {
  device_id id;
  const char* side = scope == Input ? "input" : "output";
  Error error = get_default_device_id(scope, &id);
  if (valid(error)) {
    printf("default %s device id: %d\n", side, id);
  } else {
    printf("Error on getting %s device: %s\n", side, error_message(error));
  }
}

int main() {
  show_result(Input);
  show_result(Output);
  return 0;
}
```

Similarly, the *src/get_default_device_id.cpp* can be shorten to:

```cpp
#include <iostream> // for std::cout, std::endl
#include "rust_audio_lib.h"

void show_result(Scope scope) {
  device_id id;
  const char* side = scope == Input ? "input" : "output";
  Error error = get_default_device_id(scope, &id);
  if (valid(error)) {
    std::cout << "default " << side << " device id: " << id << std::endl;
  } else {
    std::cout << "Error on getting " << side << " device: " << error_message(error) << std::endl;
  }
}

int main() {
  show_result(Input);
  show_result(Output);
  return 0;
}
```

Finally, we can test them:

```
$ gcc -o get_default_device_id src/get_default_device_id.c -L target/debug/ -lrust_audio_lib
$ ./get_default_device_id
default input device id: 39
default output device id: 39
$ g++ -o get_default_device_id src/get_default_device_id.cpp -L target/debug/ -lrust_audio_lib
$ ./get_default_device_id
default input device id: 39
default output device id: 39
```

One benefit of dynamic linking is that we can change the library any time without recompiling the *C* and *C++* code. For example, if we modify *src/lib.rs* as follows:
```rust

...

#[no_mangle] // Tell the Rust compiler not to mangle the name of this function.
pub extern fn get_default_device_id(
    scope: utils::Scope,
    id: *mut utils::DeviceId
) -> utils::Error {
    if id.is_null() {
        return utils::Error::InvalidParameters;
    }
    match utils::get_default_device_id(scope) {
        Ok(device_id) => {
            println!("hello world!"); // Add this line!
            unsafe { *id = device_id };
            utils::Error::Ok
        },
        Err(error) => { error }
    }
}

...

```

the results of *C* or *C++* executable files will change without re-compiling them:

```
$ cargo clean
$ cargo build --lib
   Compiling rust_audio_lib v0.1.0 (file:///<path>/<to>/<projects>/rust_audio_lib)
    Finished dev [unoptimized + debuginfo] target(s) in 0.91s
$ ./get_default_device_id
hello world!
default input device id: 39
hello world!
default output device id: 39
```

#### Writing a *Shell Script* to Compile *C* and *C++* Files

If we will test the library with different settings multiple times, it's better to write a script to compile them automatically. We could create a *shell script* named *compile.sh* like:

```sh

#!/bin/sh

LIBRARY_PATH="target/debug/"
LIBRARY_NAME="rust_audio_lib"

C_SOURCE="src/get_default_device_id.c"
CPP_SOURCE="src/get_default_device_id.cpp"

C_EXE="get_default_device_id-c"
CPP_EXE="get_default_device_id-cpp"

run_and_clean()
{
  echo "Run c executable file:"
  ./$C_EXE

  echo "Run c++ executable file:"
  ./$CPP_EXE

  echo "Clean executable files."
  rm $C_EXE $CPP_EXE
}

compile_to_exe()
{
  compiler=$1
  source=$2
  exe=$3
  $compiler $source -L $LIBRARY_PATH -l$LIBRARY_NAME -o $exe
}

echo "Build Rust library."
cargo build --lib

echo "Build executable files."
compile_to_exe gcc $C_SOURCE $C_EXE
compile_to_exe g++ $CPP_SOURCE $CPP_EXE

echo "Run executable files."
run_and_clean

echo "Clean Rust library."
cargo clean

```

```
$ sh compile.sh
Build Rust library.
   Compiling rust_audio_lib v0.1.0 (file:///Users/cchang/Work/playground/rust-audio-lib-sample/rust_audio_lib)
    Finished dev [unoptimized + debuginfo] target(s) in 0.77s
Build executable files.
Run executable files.
Run c executable file:
default input device id: 39
default output device id: 39
Run c++ executable file:
default input device id: 39
default output device id: 39
Clean executable files.
Clean Rust library.
```

### Creating a [Static Library][stalib]

(See the code [here][step9-static-lib].)

The process to build a **static** library is almost same as what we do to build a **dynamic** library. The differences between them are ```crate-type``` and linking options of compiler.

To build a **static** library, we need to set ```staticlib``` to ```crate-type``` in *Cargo.toml*:
```toml

...

[lib]
name = "rust_audio_lib"
# cdylib: For dynamic library to be loaded from another language.
# staticlib: For static library to be compiled with another code.
crate-type = ["staticlib"] # Run with `$ cargo build --lib`
# To make `$ cargo build` work, using `crate-type = ["staticlib", "rlib"]` instead.
# `rlib` is for static Rust library to be used internally (src/main.rs here).

...

```

```
$ cargo clean
$ cargo build --lib
   Compiling rust_audio_lib v0.1.0 (file:///<path>/<to>/<projects>/rust_audio_lib)
    Finished dev [unoptimized + debuginfo] target(s) in 0.79s
```

There is a new *librust_audio_lib.a* built under *target/debug* after running ```$ cargo build --lib```:

```
/<path>/<to>/<projects>/rust_audio_lib/target/debug/librust_audio_lib.a
```

```.a``` files are static libraries. They are usually a collection of ```.o``` object files. We can use ```ar``` command to see the contents:

```
$ ar tv target/debug/librust_audio_lib.a
---------       0/0        377264 Dec 31 16:00 1969 __.SYMDEF
rw-r--r--       0/0          4312 Dec 31 16:00 1969 rust_audio_lib-653aff045d7a1c85.181cuta0v63atwcm.rcgu.o
...
rw-r--r--       0/0        144040 Dec 31 16:00 1969 std-5426f7d812c33791.std0-1bbf4d85ba0eadbddc45b0c2a2600fd6.rs.rcgu.o
...
rw-r--r--       0/0          6120 Dec 31 16:00 1969 panic_unwind-2fc89f7407c2fdf1.panic_unwind0-8a5a3a9c1207616b54ad0b55f7f329b5.rs.rcgu.o
...
rw-r--r--       0/0          3596 Dec 31 16:00 1969 libc-9afba5cb13b6de54.libc0-58a263f228f631494bc8eece3014a334.rs.rcgu.o
...
rw-r--r--       0/0         77800 Dec 31 16:00 1969 alloc-8e2dd1e6e8b3dee0.alloc0-fdb58353d8bd33b2dc988abe4d04b49f.rs.rcgu.o
...
rw-r--r--       0/0         82352 Dec 31 16:00 1969 core-7e32fa628631e8e6.core0-f4284668a62bd8135eaa7754e32bb81c.rs.rcgu.o
...

```

Then run the same command, as what we do above, to compile *src/get_default_device_id.c*:

```
$ gcc -o get_default_device_id src/get_default_device_id.c -L target/debug/ -lrust_audio_lib
Undefined symbols for architecture x86_64:
  "_AudioObjectGetPropertyData", referenced from:
      rust_audio_lib::utils::audio_object_get_property_data::hbce4fdef7b8f205f in librust_audio_lib.a(rust_audio_lib-653aff045d7a1c85.4poa11uuafp285be.rcgu.o)
ld: symbol(s) not found for architecture x86_64
clang: error: linker command failed with exit code 1 (use -v to see invocation)
```

We get an error here! The ```AudioObjectGetPropertyData``` is undefined at **compile time**!

The ```AudioObjectGetPropertyData``` is defined in the *CoreAudio* framwork in *Mac OS* instead of in *rust_audio_lib*. *rust_audio_lib* doesn't know it either. But why *rust_audio_lib* can be built successfully? The answer is in *build.rs*. We have a ```cargo:rustc-link-lib=framework=CoreAudio``` in *build.rs*, which is used to tell *rustc* compiler that the ```AudioObjectGetPropertyData``` is defined in *CoreAudio* framwork, and *rustc* will link it **dynamically** in **run time**.

To statically link to a library that dynamically links to other libraries, we need to tell our compiler where it can find the underlying dependencies. We need to tell *gcc* or *g++* where it can find ```AudioObjectGetPropertyData```, by adding ```-framework CoreAudio```:

```
$ gcc -o get_default_device_id src/get_default_device_id.c -L target/debug/ -lrust_audio_lib -framework CoreAudio
$ ./get_default_device_id
default input device id: 39
default output device id: 39
$ g++ -o get_default_device_id src/get_default_device_id.cpp -L target/debug/ -lrust_audio_lib -framework CoreAudio
$ ./get_default_device_id
default input device id: 39
default output device id: 39
```

#### Dynamic Linking v.s. Static Linking

One interesting quesiton here is: *why we don't get this error when dynamic linking*.

[Dynamic linking][dlink] allows the executables contains **undefined** symbols with a list indicating where they are defined. The undefined symbols will be bound to their references in the provided list when the program is loaded in **run time**. The executable may just have a unreferenced symbol from ```extern Error get_default_device_id(Scope scope, device_id* id)``` and a list indicating that ```get_default_device_id``` can be found in *librust_audio_lib.dylib*. The ```get_default_device_id``` will be linked when the executable is loaded to run. The executable has **no idea** about ```AudioObjectGetPropertyData```. It doesn't care the detail in ```get_default_device_id```. And that's why we can change the implementation in ```pub extern fn get_default_device_id``` anytime(recall that we add ```println!("hello world!")``` in ```get_default_device_id``` without recompiling *C* or *C++* code.).

[Static linking][sbuild] links all symbols statically when the executables are created. All the bindings of undefined symbols in the program have to be done in **compile time**. That's why we need to tell compiler where it can find ```AudioObjectGetPropertyData```.

#### Updating the Shell Script

To make the script work with static library, we could rewrite it as follow:

```sh
#!/bin/sh

LIBRARY_PATH="target/debug/"
LIBRARY_NAME="rust_audio_lib"

C_SOURCE="src/get_default_device_id.c"
CPP_SOURCE="src/get_default_device_id.cpp"

C_EXE="get_default_device_id-c"
CPP_EXE="get_default_device_id-cpp"

run_and_clean()
{
  echo "Run c executable file:"
  ./$C_EXE

  echo "Run c++ executable file:"
  ./$CPP_EXE

  echo "Clean executable files."
  rm $C_EXE $CPP_EXE
}

compile_to_exe()
{
  compiler=$1
  source=$2
  exe=$3
  append="$4"
  $compiler $source -L $LIBRARY_PATH -l$LIBRARY_NAME $append -o $exe
}

echo "Build Rust library."
cargo clean
cargo build --lib

echo "Build executable files."
FRAMEWORK_DEPENDENCY="-framework CoreAudio"
compile_to_exe gcc $C_SOURCE $C_EXE "$FRAMEWORK_DEPENDENCY"
compile_to_exe g++ $CPP_SOURCE $CPP_EXE "$FRAMEWORK_DEPENDENCY"

# echo "Run executable files."
run_and_clean
```

We add ```append="$4"``` into ```compile_to_exe``` so it can read appended flags we need. It gives some flexibility to change the type of library we are linking. We could still compile executable files with **dynamic** libraries above without ```append```:
```
echo "Build executable files."
compile_to_exe gcc $C_SOURCE $C_EXE
compile_to_exe g++ $CPP_SOURCE $CPP_EXE
```
It works for *librust_audio_lib.dylib*.

## Reference
- [Code segment][codeseg]
- [Library (computing)][lib]
- [Static build][sbuild]
- [The Cargo Book: The Manifest Format][manifest]
- [The Rust Reference: Linkage][linkage]
- [The Rust Programming Language: Unsafe Rust][unsafe]
- [How to print a list of symbols exported from a dynamic library][stkovflw]
- [What is the effect of extern “C” in C++?][externc]
- [What is a file with extension .a?][aext]
- [GCC: Options for Linking][gcclinking]
- [man page: nm(1)][nm]
- [Exposing a Rust Library to C][sergey]
- [The Rust FFI Omnibus][shepmaster]

[codeseg]: https://en.wikipedia.org/wiki/Code_segment "Code segment"
[lib]: https://en.wikipedia.org/wiki/Library_(computing) "Library (computing)"
[stalib]: https://en.wikipedia.org/wiki/Library_(computing)#Static_libraries "Static libraries"
[shrlib]: https://en.wikipedia.org/wiki/Library_(computing)#Shared_libraries "Shared libraries"
[dlink]: https://en.wikipedia.org/wiki/Static_build#Dynamic_linking "Dynamic linking"
[sbuild]: https://en.wikipedia.org/wiki/Static_build "Static build"

[manifest]: https://doc.rust-lang.org/cargo/reference/manifest.html#building-dynamic-or-static-libraries "The Cargo Book: The Manifest Format"
[linkage]: https://doc.rust-lang.org/reference/linkage.html "The Rust Reference: Linkage"
[unsafe]: https://doc.rust-lang.org/book/second-edition/ch19-01-unsafe-rust.html#using-extern-functions-to-call-external-code "The Rust Programming Language: Unsafe Rust"

[nm]: https://linux.die.net/man/1/nm "man page: nm(1)"
[gcclinking]: https://gcc.gnu.org/onlinedocs/gcc/Link-Options.html "GCC: Options for Linking"

[externc]: https://stackoverflow.com/questions/1041866/what-is-the-effect-of-extern-c-in-c "What is the effect of extern “C” in C++?"
[aext]: https://stackoverflow.com/questions/5965171/what-is-a-file-with-extension-a "What is a file with extension .a?"
[stkovflw]: https://stackoverflow.com/questions/4506121/how-to-print-a-list-of-symbols-exported-from-a-dynamic-library "How to print a list of symbols exported from a dynamic library"

[sergey]: http://greyblake.com/blog/2017/08/10/exposing-rust-library-to-c/ "Exposing a Rust Library to C"
[shepmaster]: http://jakegoulding.com/rust-ffi-omnibus/ "The Rust FFI Omnibus"

[step9-dynamic-lib]: https://github.com/ChunMinChang/rust-audio-lib-sample/tree/7b3aa7bbeec71f707e80c388978ebc8ab22412a9/rust_audio_lib "Code for step 9: Creating a Shared Library"
[step9-static-lib]: https://github.com/ChunMinChang/rust-audio-lib-sample/tree/96be1587ddd3c8e23918dcfcd5216f56a4f7764f/rust_audio_lib "Code for step 9: Creating a Static Library"
