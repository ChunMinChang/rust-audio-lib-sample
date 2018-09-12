# Build scripts

(See the code [here][step8].)

## Closer look for ```#[link(...)]```

To get the *link* information, we use an experimental *rustc* option: ```print-link-args``` with ```-Z``` flag. It works only in **nightly** *rustc* at this moment. If you don't have nightly *rustc*, you can install nightly toolchain by ```$ rustup install nightly``` and switch default toolchain to it by ```$ rustup default nightly```. Otherwise, you will get an error like: [```error: the option `Z` is only accepted on the nightly compiler```][stkov].

To do the experiment, the first step is to create a file named *get_audio_id.rs* and put the following code into there:

```rust
use std::mem; // For mem::size_of
use std::os::raw::c_void;
use std::ptr; // For ptr::null()

//  Type Aliases
// ==============================================================================
// MacTypes.h
// -------------------------
// https://developer.apple.com/documentation/kernel/osstatus?language=objc
type OSStatus = i32;

// AudioHardwareBase.h
// -------------------------
// https://developer.apple.com/documentation/coreaudio/audioobjectid?language=objc
type AudioObjectID = u32;

// https://developer.apple.com/documentation/coreaudio/audioobjectpropertyaddress/1422175-mselector?language=objc
type AudioObjectPropertySelector = u32;
// https://developer.apple.com/documentation/coreaudio/audioobjectpropertyscope?language=objc
type AudioObjectPropertyScope = u32;
// https://developer.apple.com/documentation/coreaudio/audioobjectpropertyelement?language=objc
type AudioObjectPropertyElement = u32;

// https://developer.apple.com/documentation/coreaudio/audioobjectpropertyaddress?language=objc
#[repr(C)] // Specify data layout in the same way as C does.
struct AudioObjectPropertyAddress {
    pub mSelector: AudioObjectPropertySelector,
    pub mScope: AudioObjectPropertyScope,
    pub mElement: AudioObjectPropertyElement,
}

// https://developer.apple.com/documentation/coreaudio/1494531-anonymous/kaudiohardwarenoerror
const kAudioHardwareNoError: OSStatus = 0;

// https://developer.apple.com/documentation/coreaudio/1494461-anonymous/kaudioobjectunknown
const kAudioObjectUnknown: AudioObjectID = 0;

// https://developer.apple.com/documentation/coreaudio/1494464-anonymous/kaudioobjectpropertyscopeglobal
// 0x'glob' = 0x676C6F62 = 1735159650
const kAudioObjectPropertyScopeGlobal: AudioObjectPropertyScope = 1735159650;
// https://developer.apple.com/documentation/coreaudio/1494464-anonymous/kaudioobjectpropertyelementmaster
const kAudioObjectPropertyElementMaster: AudioObjectPropertyElement = 0;

// AudioHardware.h
// -------------------------
// https://developer.apple.com/documentation/coreaudio/1545873-anonymous/kaudioobjectsystemobject
const kAudioObjectSystemObject: AudioObjectID = 1;

// https://developer.apple.com/documentation/coreaudio/1545886-anonymous/kaudiohardwarepropertydefaultinputdevice
// 0x'dIn ' = 0x64496E20 = 1682533920
const kAudioHardwarePropertyDefaultInputDevice: AudioObjectPropertySelector = 1682533920;
// https://developer.apple.com/documentation/coreaudio/1545886-anonymous/kaudiohardwarepropertydefaultoutputdevice
// 0x'dOut' = 0x644F7574 = 1682929012
const kAudioHardwarePropertyDefaultOutputDevice: AudioObjectPropertySelector = 1682929012;

#[cfg(target_os = "macos")] // The function is only included in the build when compiling for macOS.
#[link(name = "CoreAudio", kind = "framework")] // Link to a dynamic library in CoreAudio framework.
extern "C" {
    // https://developer.apple.com/documentation/coreaudio/1422524-audioobjectgetpropertydata?language=objc
    fn AudioObjectGetPropertyData(
        inObjectID: AudioObjectID,
        inAddress: *const AudioObjectPropertyAddress,
        inQualifierDataSize: u32,
        inQualifierData: *const c_void,
        ioDataSize: *mut u32,
        outData: *mut c_void,
    ) -> OSStatus;
}

const DEFAULT_INPUT_DEVICE_PROPERTY_ADDRESS: AudioObjectPropertyAddress =
    AudioObjectPropertyAddress {
        mSelector: kAudioHardwarePropertyDefaultInputDevice,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMaster,
    };

const DEFAULT_OUTPUT_DEVICE_PROPERTY_ADDRESS: AudioObjectPropertyAddress =
    AudioObjectPropertyAddress {
        mSelector: kAudioHardwarePropertyDefaultOutputDevice,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMaster,
    };


#[derive(PartialEq)] // Enable comparison.
enum Scope {
    Input,
    Output,
}

fn get_default_device_id(scope: Scope) -> Result<AudioObjectID, OSStatus> {
    let id: AudioObjectID = kAudioObjectSystemObject;
    let address: &AudioObjectPropertyAddress = if scope == Scope::Input {
        &DEFAULT_INPUT_DEVICE_PROPERTY_ADDRESS
    } else {
        &DEFAULT_OUTPUT_DEVICE_PROPERTY_ADDRESS
    };
    let mut data: AudioObjectID = kAudioObjectUnknown;
    let mut size = mem::size_of::<AudioObjectID>() as u32; // Cast usize to u32.
    let status: OSStatus = unsafe {
        AudioObjectGetPropertyData(
            id,
            address, // as *const AudioObjectPropertyAddress
            0,
            ptr::null(),
            &mut size, // as *mut u32
            // Cast AudioObjectID ref to a raw AudioObjectID pointer first,
            // and then cast raw AudioObjectID pointer to void pointer.
            &mut data as *mut AudioObjectID as *mut c_void,
        )
    };
    if status == kAudioHardwareNoError {
        Ok(data)
    } else {
        Err(status)
    }
}

fn show_result(scope: Scope) {
    let side = if scope == Scope::Input { "input" } else { "output" };
    match get_default_device_id(scope) {
        Ok(id) => {
            println!("default {} device id: {}", side, id);
        },
        Err(error) => {
            println!("Failed to get {} device id. Error code: {}", side, error);
        }
    }
}

fn main() {
    show_result(Scope::Input);
    show_result(Scope::Output);
}
```

Next, running ```rustc -Z print-link-args get_audio_id.rs``` to get *linker* information:

```
$ rustc -Z print-link-args get_audio_id.rs
"cc" "-m64" "-L" "/<path>/<username>/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib" "get_audio_id.get_audio_id0.rcgu.o" "get_audio_id.get_audio_id1.rcgu.o" "get_audio_id.get_audio_id2.rcgu.o" "get_audio_id.get_audio_id3.rcgu.o" "get_audio_id.get_audio_id4.rcgu.o" "get_audio_id.get_audio_id5.rcgu.o" "get_audio_id.get_audio_id6.rcgu.o" "get_audio_id.get_audio_id7.rcgu.o" "-o" "get_audio_id" "get_audio_id.crate.allocator.rcgu.o" "-Wl,-dead_strip" "-nodefaultlibs" "-L" "/<path>/<username>/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib" "-framework" "CoreAudio" "/<path>/<username>/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/libstd-5426f7d812c33791.rlib" "/<path>/<username>/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/libpanic_unwind-2fc89f7407c2fdf1.rlib" "/<path>/<username>/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/liballoc_jemalloc-7973437d75e41551.rlib" "/<path>/<username>/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/libunwind-84afc50178e78273.rlib" "/<path>/<username>/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/liballoc_system-f2e8cf90553ff84f.rlib" "/<path>/<username>/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/liblibc-9afba5cb13b6de54.rlib" "/<path>/<username>/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/liballoc-8e2dd1e6e8b3dee0.rlib" "/<path>/<username>/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/libcore-7e32fa628631e8e6.rlib" "/<path>/<username>/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/libcompiler_builtins-a4054d9b44401c28.rlib" "-lSystem" "-lresolv" "-lpthread" "-lc" "-lm"
```

```
$ ./get_audio_id
default input device id: 39
default output device id: 39
```


There are some notable keywords: ```cc```, ```-L```, ```-framework```, ```CoreAudio``` shown when running ```rustc -Z print-link-args```. These keywords remind me of something like ```gcc ... -freamework CoreAudio```. And ```-L``` may be used to assign a path to search the linking libraries like [what *gcc* does][gcclinking].

More precisely, I mean, a file named ```get_audio_id.c``` with the following code:

```c
#include <stdio.h>
#include <CoreAudio/CoreAudio.h>

const AudioObjectPropertyAddress kDefaultInputDevicePropertyAddress = {
  kAudioHardwarePropertyDefaultInputDevice,
  kAudioObjectPropertyScopeGlobal,
  kAudioObjectPropertyElementMaster
};

const AudioObjectPropertyAddress kDefaultOutputDevicePropertyAddress = {
  kAudioHardwarePropertyDefaultOutputDevice,
  kAudioObjectPropertyScopeGlobal,
  kAudioObjectPropertyElementMaster
};

typedef enum {
  Input,
  Output
} Scope;

AudioObjectID get_default_device_id(Scope scope) {
  const AudioObjectPropertyAddress* address = scope == Input
    ? &kDefaultInputDevicePropertyAddress
    : &kDefaultOutputDevicePropertyAddress;
  AudioObjectID id = kAudioObjectUnknown;
  UInt32 size = (UInt32)sizeof(id);
  OSStatus s = AudioObjectGetPropertyData(kAudioObjectSystemObject, address,
                                          0, NULL, &size, &id);
  return s == kAudioHardwareNoError ? id : kAudioObjectUnknown;
}

int main() {
  printf("default input device id: %d\n", get_default_device_id(Input));
  printf("default output device id: %d\n", get_default_device_id(Output));
  return 0;
}
```

can be compiled by ```gcc -o get_audio_id get_audio_id.c -framework CoreAudio```:

```
$ gcc -o get_audio_id get_audio_id.c -framework CoreAudio
$ ./get_audio_id
default input device id: 39
default output device id: 39
```

I don't know how *Rust* works internally, but I guess what *linker* in *Rust* does for ```#[link(name = "CoreAudio", kind = "framework")]``` is something similar to what ```gcc -framework CoreAudio``` does on *Mac OS*.

## Shared Object Dependencies

We can use [```otool -L```][otool] (use [```ldd```][ldd] if you're on linux) to show the shared object dependencies. Let's take a look at what ```rustc``` does:

```
$ rustc get_audio_id.rs
$ otool -L get_audio_id
get_audio_id:
	/System/Library/Frameworks/CoreAudio.framework/Versions/A/CoreAudio (compatibility version 1.0.0, current version 1.0.0)
	/usr/lib/libSystem.B.dylib (compatibility version 1.0.0, current version 1252.50.4)
	/usr/lib/libresolv.9.dylib (compatibility version 1.0.0, current version 1.0.0)
```

Then take a look at what ```gcc``` does:

```
$ gcc -o get_audio_id get_audio_id.c -framework CoreAudio
$ otool -L get_audio_id
get_audio_id:
	/System/Library/Frameworks/CoreAudio.framework/Versions/A/CoreAudio (compatibility version 1.0.0, current version 1.0.0)
	/usr/lib/libSystem.B.dylib (compatibility version 1.0.0, current version 1252.50.4)
```

We can observe that the dependencies in the executable files compiled from ```rustc``` and ```gcc``` are almost same. But the one compiled in ```rustc``` has an additional dependency to ```libresolv```. (TODO: Find out why it exists.)

## Creating a Build Script

While using low-level link setting: ```#[link(name = "...")``` before the ```extern``` block explicitly indicates the replationship of the functions in the block and the libraries they will be linked to, it's harder to manage. If there are more modules using the native APIs, the ```#[link(name = "...")``` attributes will be marked everywhere. Things getting worse when the APIs are platform-dependent. The build will fail on linking stage on wrong platforms if there is no proper conditionally-compiled settings. Even we have proper configuration options, these information are separated among different code sections. The more attributes we have, the more distractions we have when reading code. The ideal case is that we can just write something as what we do in [Calling Native C APIs from Rust][callingC]:

```rust
extern "C" { // or just extern
    ...

    fn do_something(...) -> ... {
        ...
    }

    ...
}
```

and the ```do_something()``` will be automatically linked to the libraries we need.

In fact, we can remove ```#[link(name = "...")``` on the ```extern``` block if we specify the linking library to *rustc* compiler in advance by ```-l``` flag.

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
        --crate-type [bin|lib|rlib|dylib|cdylib|staticlib|proc-macro]
                        Comma separated list of types of crates for the
                        compiler to emit
        --crate-name NAME
                        Specify the name of the crate being built
        --emit [asm|llvm-bc|llvm-ir|obj|metadata|link|dep-info|mir]
                        Comma separated list of types of output for the
                        compiler to emit
        --print [crate-name|file-names|sysroot|cfg|target-list|target-cpus|target-features|relocation-models|code-models|tls-models|target-spec-json|native-static-libs]
                        Comma separated list of compiler information to print
                        on stdout
    ...
```

Remove the ```#[link(name = "CoreAudio", kind = "framework")]``` in the above *get_audio_id.rs* file:

```rust
...
const kAudioHardwarePropertyDefaultOutputDevice: AudioObjectPropertySelector = 1682929012;

// Delete #[link(name = "CoreAudio", kind = "framework")] here
extern "C" {
    // https://developer.apple.com/documentation/coreaudio/1422524-audioobjectgetpropertydata?language=objc
    fn AudioObjectGetPropertyData(
        inObjectID: AudioObjectID,
        inAddress: *const AudioObjectPropertyAddress,
        inQualifierDataSize: u32,
        inQualifierData: *const c_void,
        ioDataSize: *mut u32,
        outData: *mut c_void,
    ) -> OSStatus;
}

const DEFAULT_INPUT_DEVICE_PROPERTY_ADDRESS: AudioObjectPropertyAddress =
...
```

and then compile it by ```$ rustc -l framework=CoreAudio ...``` to see if it works:

```
$ rustc -l framework=CoreAudio get_audio_id.rs
...
$ otool -L get_audio_id
get_audio_id:
        /System/Library/Frameworks/CoreAudio.framework/Versions/A/CoreAudio (compatibility version 1.0.0, current version 1.0.0)
        /usr/lib/libSystem.B.dylib (compatibility version 1.0.0, current version 1252.50.4)
        /usr/lib/libresolv.9.dylib (compatibility version 1.0.0, current version 1.0.0)
$ ./get_audio_id
default input device id: 39
default output device id: 39
```

Cool! Now we know how to remove the annoying ```link``` attributes from one *Rust* file. How do we do the same thing if we want to build a library crate?

The answer is [*build script*][bs]! We can define all the linking libraries in the *build script*. In addition, it can perform platform-specific configuration by checking platform from [environment variables: ```TARGET```][envvar]. One example is [*servo/skia*][skia]. It's useful if the library is platform-dependent or cross-platform. For the cross-plaform library, we need to link to different libraries on different platforms. We use ```rustc-link-lib=[KIND=]NAME``` tp specify the linking library named ```NAME``` to *rustc* compiler and do what ```-l``` does.

Let's write a [*build script*][bs] for our *rust_audio_lib*. The first step is to create a *build.rs* in the root directory with one of the following code. Personally I like the second one, since it can warn users if they run the code on the wrong platforms.

```rust
fn main() {
    if std::env::var("TARGET").unwrap().contains("apple-darwin") {
        println!("cargo:rustc-link-lib=framework=CoreAudio");
    }
}
```

```rust
#[cfg(any(target_os = "macos", target_os = "ios"))]
fn main() {
    println!("cargo:rustc-link-lib=framework=CoreAudio");
}

#[cfg(not(any(target_os = "macos", target_os = "ios")))]
fn main() {
    eprintln!("This library requires macos or ios target");
}
```

*Cargo* will look up the file named *build.rs* in the package root by default when running ```cargo build```. We could also specify the *build script* in *Cargo.toml* with any name you like:

```toml
[package]
..
build = "any_name_you_like.rs"

```

After creating the *build script*, we can remove the ```#[link(name = "...")``` on the ```extern``` block in *src/sys.rs*:

```rust
// Delete #[cfg(target_os = "macos")] here
// Delete #[link(name = "CoreAudio", kind = "framework")] here
extern "C" {
    // https://developer.apple.com/documentation/coreaudio/1422524-audioobjectgetpropertydata?language=objc
    pub fn AudioObjectGetPropertyData(
        inObjectID: AudioObjectID,
        inAddress: *const AudioObjectPropertyAddress,
        inQualifierDataSize: u32,
        inQualifierData: *const c_void,
        ioDataSize: *mut u32,
        outData: *mut c_void,
    ) -> OSStatus;
}
```

Finally, we can build it by ```cargo build``` with ```-vv``` very-verbose output messages to see what happens:

```
$ cargo clean
$ cargo build -vv
   Compiling rust_audio_lib v0.1.0 (file:///<path>/<to>/<projects>/rust_audio_lib)
     Running `rustc --crate-name build_script_build build.rs --crate-type bin --emit=dep-info,link -C debuginfo=2 -C metadata=d12269031ab7bb37 -C extra-filename=-d12269031ab7bb37 --out-dir /<path>/<to>/<projects>/rust_audio_lib/target/debug/build/rust_audio_lib-d12269031ab7bb37 -C incremental=/<path>/<to>/<projects>/rust_audio_lib/target/debug/incremental -L dependency=/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps`
     Running `/<path>/<to>/<projects>/rust_audio_lib/target/debug/build/rust_audio_lib-d12269031ab7bb37/build-script-build`
cargo:rustc-link-lib=framework=CoreAudio
     Running `rustc --crate-name rust_audio_lib src/lib.rs --crate-type lib --emit=dep-info,link -C debuginfo=2 -C metadata=39ed89820159b5ee -C extra-filename=-39ed89820159b5ee --out-dir /<path>/<to>/<projects>/rust_audio_lib/target/debug/deps -C incremental=/<path>/<to>/<projects>/rust_audio_lib/target/debug/incremental -L dependency=/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps -l framework=CoreAudio`
     Running `rustc --crate-name rust_audio_lib src/main.rs --crate-type bin --emit=dep-info,link -C debuginfo=2 -C metadata=a9b5f18e0a301b88 -C extra-filename=-a9b5f18e0a301b88 --out-dir /<path>/<to>/<projects>/rust_audio_lib/target/debug/deps -C incremental=/<path>/<to>/<projects>/rust_audio_lib/target/debug/incremental -L dependency=/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps --extern rust_audio_lib=/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps/librust_audio_lib-39ed89820159b5ee.rlib`
    Finished dev [unoptimized + debuginfo] target(s) in 1.02s
$ otool -L target/debug/rust_audio_lib
target/debug/rust_audio_lib:
        /System/Library/Frameworks/CoreAudio.framework/Versions/A/CoreAudio (compatibility version 1.0.0, current version 1.0.0)
        /usr/lib/libSystem.B.dylib (compatibility version 1.0.0, current version 1252.50.4)
        /usr/lib/libresolv.9.dylib (compatibility version 1.0.0, current version 1.0.0)
```

## What *Build Script* Does

The fisrt ```Running```:
```
Running `rustc --crate-name build_script_build build.rs --crate-type bin --emit=dep-info,link -C debuginfo=2 -C metadata=d12269031ab7bb37 -C extra-filename=-d12269031ab7bb37 --out-dir /<path>/<to>/<projects>/rust_audio_lib/target/debug/build/rust_audio_lib-d12269031ab7bb37 -C incremental=/<path>/<to>/<projects>/rust_audio_lib/target/debug/incremental -L dependency=/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps`
```
is for ```build.rs```.

Then the second ```Running```:
```
Running `/<path>/<to>/<projects>/rust_audio_lib/target/debug/build/rust_audio_lib-d12269031ab7bb37/build-script-build`
cargo:rustc-link-lib=framework=CoreAudio
```
executes what we set in *buils.rs*: ```cargo:rustc-link-lib=framework=CoreAudio```.

Next, the third ```Running```:
```
Running `rustc --crate-name rust_audio_lib src/lib.rs --crate-type lib --emit=dep-info,link -C debuginfo=2 -C metadata=39ed89820159b5ee -C extra-filename=-39ed89820159b5ee --out-dir /<path>/<to>/<projects>/rust_audio_lib/target/debug/deps -C incremental=/<path>/<to>/<projects>/rust_audio_lib/target/debug/incremental -L dependency=/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps -l framework=CoreAudio`
```
build ```src/lib.rs``` with ```-L dependency=/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps -l framework=CoreAudio```, that is defined in the previous(second) ```Running```.

Finally, the last ```Running```:
```
Running `rustc --crate-name rust_audio_lib src/main.rs --crate-type bin --emit=dep-info,link -C debuginfo=2 -C metadata=a9b5f18e0a301b88 -C extra-filename=-a9b5f18e0a301b88 --out-dir /<path>/<to>/<projects>/rust_audio_lib/target/debug/deps -C incremental=/<path>/<to>/<projects>/rust_audio_lib/target/debug/incremental -L dependency=/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps --extern rust_audio_lib=/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps/librust_audio_lib-39ed89820159b5ee.rlib`
```
build ```src/main.rs``` with the ```rlib``` built by previous(third) ```Running```: ```librust_audio_lib-39ed89820159b5ee.rlib``` (check ```39ed89820159b5ee```).


### Difference: With or Without *Build Script*

To see what the differences with or without *build.rs* are, we could reverse all the changes in this chapter and use ```$ cargo build -vv``` to get the original output messages with the ```#[link(name = "...")``` attribute, without ```build.rs```:
```
$ cargo clean
$ cargo build -vv
   Compiling rust_audio_lib v0.1.0 (file:///<path>/<to>/<projects>/rust_audio_lib)
     Running `rustc --crate-name rust_audio_lib src/lib.rs --crate-type lib --emit=dep-info,link -C debuginfo=2 -C metadata=2aed894fd2c37dbb -C extra-filename=-2aed894fd2c37dbb --out-dir /<path>/<to>/<projects>/rust_audio_lib/target/debug/deps -C incremental=/<path>/<to>/<projects>/rust_audio_lib/target/debug/incremental -L dependency=/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps`
     Running `rustc --crate-name rust_audio_lib src/main.rs --crate-type bin --emit=dep-info,link -C debuginfo=2 -C metadata=94b661cdcadabf3d -C extra-filename=-94b661cdcadabf3d --out-dir /<path>/<to>/<projects>/rust_audio_lib/target/debug/deps -C incremental=/<path>/<to>/<projects>/rust_audio_lib/target/debug/incremental -L dependency=/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps --extern rust_audio_lib=/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps/librust_audio_lib-2aed894fd2c37dbb.rlib`
    Finished dev [unoptimized + debuginfo] target(s) in 0.60s
$ otool -L target/debug/rust_audio_lib
target/debug/rust_audio_lib:
        /System/Library/Frameworks/CoreAudio.framework/Versions/A/CoreAudio (compatibility version 1.0.0, current version 1.0.0)
        /usr/lib/libSystem.B.dylib (compatibility version 1.0.0, current version 1252.50.4)
        /usr/lib/libresolv.9.dylib (compatibility version 1.0.0, current version 1.0.0)
```

The **first** ```Running``` without *build.rs*:
```
Running `rustc --crate-name rust_audio_lib src/lib.rs --crate-type lib --emit=dep-info,link -C debuginfo=2 -C metadata=2aed894fd2c37dbb -C extra-filename=-2aed894fd2c37dbb --out-dir /<path>/<to>/<projects>/rust_audio_lib/target/debug/deps -C incremental=/<path>/<to>/<projects>/rust_audio_lib/target/debug/incremental -L dependency=/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps`
```
is almost same with the **third** ```Running``` with *build.rs*:
```
Running `rustc --crate-name rust_audio_lib src/lib.rs --crate-type lib --emit=dep-info,link -C debuginfo=2 -C metadata=39ed89820159b5ee -C extra-filename=-39ed89820159b5ee --out-dir /<path>/<to>/<projects>/rust_audio_lib/target/debug/deps -C incremental=/<path>/<to>/<projects>/rust_audio_lib/target/debug/incremental -L dependency=/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps -l framework=CoreAudio`
```
They both are for ```src/lib.rs```, but the additional link message: ```-l framework=CoreAudio``` will be shown if using *build.rs*.


The **second** ```Running``` without *build.rs*:
```
Running `rustc --crate-name rust_audio_lib src/main.rs --crate-type bin --emit=dep-info,link -C debuginfo=2 -C metadata=94b661cdcadabf3d -C extra-filename=-94b661cdcadabf3d --out-dir /<path>/<to>/<projects>/rust_audio_lib/target/debug/deps -C incremental=/<path>/<to>/<projects>/rust_audio_lib/target/debug/incremental -L dependency=/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps --extern rust_audio_lib=/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps/librust_audio_lib-2aed894fd2c37dbb.rlib`
```
is basically same with the **last** ```Running``` with *build.rs*:
```
Running `rustc --crate-name rust_audio_lib src/main.rs --crate-type bin --emit=dep-info,link -C debuginfo=2 -C metadata=a9b5f18e0a301b88 -C extra-filename=-a9b5f18e0a301b88 --out-dir /<path>/<to>/<projects>/rust_audio_lib/target/debug/deps -C incremental=/<path>/<to>/<projects>/rust_audio_lib/target/debug/incremental -L dependency=/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps --extern rust_audio_lib=/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps/librust_audio_lib-39ed89820159b5ee.rlib`
```
They both are for ```src/main.rs```, compiled based on ```rlib``` built by its previous ```Running```(check ```2aed894fd2c37dbb``` and ```39ed89820159b5ee```).

### Summary

To sum up the comparisons, if *build script* is defined:
- We will have two additional steps before building ```src/lib.rs``` and ```src/main.rs```:
  1. Compile *build script*.
  2. Invoke settings in *build script* before following steps.
- The last two steps are basically same with the buiding process without *build script*:
  1. Compile ```src/lib.rs``` with an additional link message: ```-l framework=CoreAudio```.
  2. Compile ```src/main.rs``` based on the *library* that is built by previous step.

There are more advanced usages in *build script*. Please read the [*Build Scripts*][bs] chapter in *The Cargo Book* for more details.

## Passing Linker Arguments to *rustc* in Cargo

To show the hidden linking information in *rustc*, we need to pass the ```-Z print-link-args``` arguments. The arguments for *rustc* can be set to ```rustflags``` in *Cargo* config: *~/.cargo/config*. For what we need, we can set:

```
[build]
rustflags = [ "-Z", "print-link-args" ]
```

To check if it works, run ```$ cargo build -vv``` in the original *rust_audio_lib* without *build.rs*:

```
$ cargo build -vv
   Compiling rust_audio_lib v0.1.0 (file:///<path>/<to>/<projects>/rust_audio_lib)
     Running `rustc --crate-name rust_audio_lib src/lib.rs --crate-type lib --emit=dep-info,link -C debuginfo=2 -C metadata=2aed894fd2c37dbb -C extra-filename=-2aed894fd2c37dbb --out-dir /<path>/<to>/<projects>/rust_audio_lib/target/debug/deps -C incremental=/<path>/<to>/<projects>/rust_audio_lib/target/debug/incremental -L dependency=/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps -Z print-link-args`
     Running `rustc --crate-name rust_audio_lib src/main.rs --crate-type bin --emit=dep-info,link -C debuginfo=2 -C metadata=94b661cdcadabf3d -C extra-filename=-94b661cdcadabf3d --out-dir /<path>/<to>/<projects>/rust_audio_lib/target/debug/deps -C incremental=/<path>/<to>/<projects>/rust_audio_lib/target/debug/incremental -L dependency=/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps --extern rust_audio_lib=/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps/librust_audio_lib-2aed894fd2c37dbb.rlib -Z print-link-args`
"cc" "-m64" "-L" "/<some_path>/<user_name>/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib" "/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps/rust_audio_lib-94b661cdcadabf3d.1y16o1qfye96o7m0.rcgu.o" "/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps/rust_audio_lib-94b661cdcadabf3d.3rngp6bm2u2q5z0y.rcgu.o" "/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps/rust_audio_lib-94b661cdcadabf3d.4oc10dk278mpk1vy.rcgu.o" "/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps/rust_audio_lib-94b661cdcadabf3d.4xq48u46a1pwiqn7.rcgu.o" "/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps/rust_audio_lib-94b661cdcadabf3d.7l0c3dhp9ogw9hz.rcgu.o" "/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps/rust_audio_lib-94b661cdcadabf3d.8xzrsc1ux72v29j.rcgu.o" "/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps/rust_audio_lib-94b661cdcadabf3d.oa3rad818d8sgn4.rcgu.o" "-o" "/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps/rust_audio_lib-94b661cdcadabf3d" "/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps/rust_audio_lib-94b661cdcadabf3d.crate.allocator.rcgu.o" "-Wl,-dead_strip" "-nodefaultlibs" "-L" "/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps" "-L" "/<some_path>/<user_name>/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib" "/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps/librust_audio_lib-2aed894fd2c37dbb.rlib" "/<some_path>/<user_name>/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/libstd-5426f7d812c33791.rlib" "/<some_path>/<user_name>/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/libpanic_unwind-2fc89f7407c2fdf1.rlib" "/<some_path>/<user_name>/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/liballoc_jemalloc-7973437d75e41551.rlib" "/<some_path>/<user_name>/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/libunwind-84afc50178e78273.rlib" "/<some_path>/<user_name>/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/liballoc_system-f2e8cf90553ff84f.rlib" "/<some_path>/<user_name>/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/liblibc-9afba5cb13b6de54.rlib" "/<some_path>/<user_name>/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/liballoc-8e2dd1e6e8b3dee0.rlib" "/<some_path>/<user_name>/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/libcore-7e32fa628631e8e6.rlib" "/<some_path>/<user_name>/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/libcompiler_builtins-a4054d9b44401c28.rlib" "-framework" "CoreAudio" "-lSystem" "-lresolv" "-lpthread" "-lc" "-lm"
    Finished dev [unoptimized + debuginfo] target(s) in 0.66s
```

### What it does

```
Running `rustc --crate-name rust_audio_lib src/lib.rs --crate-type lib --emit=dep-info,link -C debuginfo=2 -C metadata=2aed894fd2c37dbb -C extra-filename=-2aed894fd2c37dbb --out-dir /<path>/<to>/<projects>/rust_audio_lib/target/debug/deps -C incremental=/<path>/<to>/<projects>/rust_audio_lib/target/debug/incremental -L dependency=/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps -Z print-link-args`
```
The first ```Running``` is almost same as what we see in the first ```Running``` message without ```build.rs``` above. It just add a ```-Z print-link-args``` at the end.

```
Running `rustc --crate-name rust_audio_lib src/main.rs --crate-type bin --emit=dep-info,link -C debuginfo=2 -C metadata=94b661cdcadabf3d -C extra-filename=-94b661cdcadabf3d --out-dir /<path>/<to>/<projects>/rust_audio_lib/target/debug/deps -C incremental=/<path>/<to>/<projects>/rust_audio_lib/target/debug/incremental -L dependency=/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps --extern rust_audio_lib=/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps/librust_audio_lib-2aed894fd2c37dbb.rlib -Z print-link-args`
```
Similarly, the second ```Running``` is almost same as what we see in the second ```Running``` message without ```build.rs``` above. It just add a ```-Z print-link-args``` at the end.

```
"cc" "-m64" "-L" "/<some_path>/<user_name>/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib" "/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps/rust_audio_lib-94b661cdcadabf3d.1y16o1qfye96o7m0.rcgu.o" "/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps/rust_audio_lib-94b661cdcadabf3d.3rngp6bm2u2q5z0y.rcgu.o" "/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps/rust_audio_lib-94b661cdcadabf3d.4oc10dk278mpk1vy.rcgu.o" "/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps/rust_audio_lib-94b661cdcadabf3d.4xq48u46a1pwiqn7.rcgu.o" "/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps/rust_audio_lib-94b661cdcadabf3d.7l0c3dhp9ogw9hz.rcgu.o" "/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps/rust_audio_lib-94b661cdcadabf3d.8xzrsc1ux72v29j.rcgu.o" "/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps/rust_audio_lib-94b661cdcadabf3d.oa3rad818d8sgn4.rcgu.o" "-o" "/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps/rust_audio_lib-94b661cdcadabf3d" "/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps/rust_audio_lib-94b661cdcadabf3d.crate.allocator.rcgu.o" "-Wl,-dead_strip" "-nodefaultlibs" "-L" "/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps" "-L" "/<some_path>/<user_name>/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib" "/<path>/<to>/<projects>/rust_audio_lib/target/debug/deps/librust_audio_lib-2aed894fd2c37dbb.rlib" "/<some_path>/<user_name>/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/libstd-5426f7d812c33791.rlib" "/<some_path>/<user_name>/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/libpanic_unwind-2fc89f7407c2fdf1.rlib" "/<some_path>/<user_name>/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/liballoc_jemalloc-7973437d75e41551.rlib" "/<some_path>/<user_name>/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/libunwind-84afc50178e78273.rlib" "/<some_path>/<user_name>/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/liballoc_system-f2e8cf90553ff84f.rlib" "/<some_path>/<user_name>/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/liblibc-9afba5cb13b6de54.rlib" "/<some_path>/<user_name>/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/liballoc-8e2dd1e6e8b3dee0.rlib" "/<some_path>/<user_name>/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/libcore-7e32fa628631e8e6.rlib" "/<some_path>/<user_name>/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/libcompiler_builtins-a4054d9b44401c28.rlib" "-framework" "CoreAudio" "-lSystem" "-lresolv" "-lpthread" "-lc" "-lm"
```
The last log message shows a lot of libraries we are linking to: ```libstd```, ```liballoc```, ... etc.
At the end, the link defined in ```#[link(name = "CoreAudio", kind = "framework")]``` attribute is translated into ```"-framework" "CoreAudio"``` here.

## References
- Display shared object dependencies
  - [```otool -L```][otool]
  - [```ldd```][ldd]
- [GCC: Options for Linking][gcclinking]
- [servo/skia][skia]
- [The Cargo Book: Build Scripts][bs]
- [Environment variables Cargo sets for build scripts][envvar]
- [Linking Rust crates to native libraries][Rufflewind]
- [How to write build.rs scripts properly][kazlauskas]

[callingC]: step2.md "Calling Native C APIs from Rust"

[stkov]: https://stackoverflow.com/questions/48675235/error-the-option-z-is-only-accepted-on-the-nightly-compiler "error: the option `Z` is only accepted on the nightly compiler"

[advlink]: https://doc.rust-lang.org/1.9.0/book/advanced-linking.html "The Rust Programming Language(first edition): Advanced Linking"
[otool]: http://www.manpagez.com/man/1/otool/ "man page: otool"
[ldd]: http://man7.org/linux/man-pages/man1/ldd.1.html "man page: ldd"

[gcclinking]: https://gcc.gnu.org/onlinedocs/gcc/Link-Options.html "GCC: Options for Linking"

[bs]: https://doc.rust-lang.org/cargo/reference/build-scripts.html "The Cargo Book: Build Scripts"
[envvar]: https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-build-scripts "The Cargo Book: Environment variables Cargo sets for build scripts"
[skia]: https://github.com/servo/skia/blob/b46cc4a783cce394f62299269eeb7f88854f58f0/build.rs#L15 "build.rs in servo/skia"
[Rufflewind]: https://rufflewind.com/2017-11-19/linking-in-rust "Linking Rust crates to native libraries"
[kazlauskas]: https://kazlauskas.me/entries/writing-proper-buildrs-scripts.html "How to write build.rs scripts properly"

[step8]: https://github.com/ChunMinChang/rust-audio-lib-sample/tree/2369a06d3e0807c9b0f4f2d75354c2f17ea3d8d4/rust_audio_lib "Code for step 8"