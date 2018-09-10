# Library Based on Platform C API

Now, it's time to build our sample library!

In this sample, we will create an audio library based on *OS X*'s *CoreAudio* framework. To make it simple, this library only contain one API that returns the [```AudioObjectID```][aoid] of the default input or output device if the calling is successful, or an error otherwise. The API will use [```AudioObjectGetPropertyData```][gpd] to query the system information. If you're not familiar with this API, it's fine. We have a *C/C++* [example here][abc]. The mission is to rewrite it into *Rust*.

## Practicing TDD with Mock API
Again, to practice *TDD(test-driven development)*, we should write tests before implementing functions. Since there is no API for now, we can simply create a mock API and use it to write tests. On ther other hand, we can also build a rough outline of the library by the mock API.

Put the following code in *src/lib.rs*:
```rust
pub mod utils {
    #[derive(PartialEq)] // Enable comparison.
    pub enum Scope {
        Input,
        Output,
    }

    pub fn get_default_device_id(scope: Scope) -> Result<u32, i32> {
        let id: u32 = if scope == Scope::Input { 1 } else { 2 };
        get_property_data(id)
    }

    // Mock API
    fn get_property_data(id: u32) -> Result<u32, i32> {
        Err(id as i32)
    }

    #[cfg(test)] // Indicates this is only included when running `cargo test`
    mod tests { // A private internal submodule in utils
        use super::*; // To use the functions in utils

        #[test] // Built only within `cargo test`.
        fn utils_get_property_data_invalid_id() {
            let invalid_id: u32 = 0;
            assert!(get_property_data(invalid_id).is_err());
        }

        #[test] // Built only within `cargo test`.
        fn utils_get_property_data() {
            let id: u32 = 10;
            assert!(get_property_data(id).is_ok());
        }
    }
}
```

The ```get_property_data``` is a mock API that should call the native [```AudioObjectGetPropertyData```][gpd] API. [```AudioObjectGetPropertyData```][gpd] takes [```AudioObjectID```][aoid] whose type is a *32-bit unsigned int* as one of its parameter and returns an [OSStatus][oss] whose type is a *32-bit signed int*, so we take a ```u32``` parameter and return a ```i32``` when there is an error. Returning a [```Result```][result] is a common error handling pattern in *Rust*, so we adopt this idiom.

After having a mock API, we write two simple unit tests:
- calling the internal API with invalid parameters: ```utils_get_property_data_invalid_id```
- calling the internal API with valid parameters: ```utils_get_property_data```

Next, we write a simple integration test in *tests/integration.rs*:

```rust
extern crate rust_audio_lib; // Introduce the `rust_audio_lib` library crate.
use rust_audio_lib::utils; // Refer to `utils` module

#[test]
fn utils_get_default_device_id() {
    assert!(utils::get_default_device_id(utils::Scope::Input).is_ok());
    assert!(utils::get_default_device_id(utils::Scope::Output).is_ok());
}
```

It verifies the API works for default input and output devices.

Finally, we can use the API to show the results in *src/main.rs*:

```rust
extern crate rust_audio_lib;
use rust_audio_lib::utils; // Refer to `utils` module

fn show_result(scope: utils::Scope) {
    let side = if scope == utils::Scope::Input { "input" } else { "output" };
    match utils::get_default_device_id(scope) {
        Ok(id) => {
            println!("default {} device id: {}", side, id);
        },
        Err(error) => {
            println!("Failed to get {} device id. Error: {}", side, error);
        }
    }
}

fn main() {
    show_result(utils::Scope::Input);
    show_result(utils::Scope::Output);
}
```

## Foreign Function Interface: Linking to Platform Library

Recall what we did in [Calling Native C APIs from Rust][callingC]: To call native APIs, *Rust* provides a *FFI* mechanism by keyword ```extern```. We can call the native platform-dependent C APIs in the similar way, with different *link* settings. To make it simple, let's assume we have a ```sys``` module that wraps all the the native types and APIs we need. This is a common way to organize *Rust* code. See more details in [modules chapter][mod] of *The Rust Programming Language*.

Based on the assumption we set, we can rewrite *src/lib.rs* as follows:

```rust
mod sys; // Module contains types and functions of theexternal libraries.

pub mod utils {
    use super::sys::*; // Bring public types and functions in sys into scope.

    #[derive(PartialEq)] // Enable comparison.
    pub enum Scope {
        Input,
        Output,
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

    pub fn get_default_device_id(scope: Scope) -> Result<AudioObjectID, OSStatus> {
        let id: AudioObjectID = kAudioObjectSystemObject;
        let address: &AudioObjectPropertyAddress = if scope == Scope::Input {
            &DEFAULT_INPUT_DEVICE_PROPERTY_ADDRESS
        } else {
            &DEFAULT_OUTPUT_DEVICE_PROPERTY_ADDRESS
        };
        get_property_data::<AudioObjectID>(id, address)
    }

    #[cfg(test)] // Indicates this is only included when running `cargo test`
    mod tests { // A private internal submodule in utils
        use super::*; // To use the functions in utils

        #[test] // Built only within `cargo test`.
        fn utils_get_property_data_invalid_id() {

            // Invalid AudioObjectID with valid input adrress.
            assert_eq!(get_property_data::<AudioObjectID>(
                        kAudioObjectUnknown,
                        &DEFAULT_INPUT_DEVICE_PROPERTY_ADDRESS
                    ).unwrap_err(), kAudioHardwareBadObjectError);
            // Invalid AudioObjectID with valid output adrress.
            assert_eq!(get_property_data::<AudioObjectID>(
                        kAudioObjectUnknown,
                        &DEFAULT_OUTPUT_DEVICE_PROPERTY_ADDRESS
                    ).unwrap_err(), kAudioHardwareBadObjectError);
        }

        #[test] // Built only within `cargo test`.
        fn utils_get_property_data() {
            // Check the default input device id is valid.
            assert_ne!(get_property_data::<AudioObjectID>(
                        kAudioObjectSystemObject,
                        &DEFAULT_INPUT_DEVICE_PROPERTY_ADDRESS
                    ).unwrap(), kAudioObjectUnknown);
            // Check the default output device id is valid.
            assert_ne!(get_property_data::<AudioObjectID>(
                        kAudioObjectSystemObject,
                        &DEFAULT_OUTPUT_DEVICE_PROPERTY_ADDRESS
                    ).unwrap(), kAudioObjectUnknown);
        }
    }
}
```

Beyond the language level, the process to get the ```AudioObjectID``` is pretty similar to the [example][abc].

All the native types like ```AudioObjectPropertyAddress```, ```kAudioObjectUnknown```,... , etc are all defined in the ```sys``` module and are brought into scope of ```utils``` by ```use super::sys::*;```. The key API: ```get_property_data``` is a wrapper for ```AudioObjectGetPropertyData``` that returns a ```Result<AudioObjectID, OSStatus>```. That is, if the calling is successful, then we will get a ```AudioObjectID```. Otherwise, we will get an error with ```OSStatus``` value indicating what is wrong. It's better to throw an error with clear message instead of the native error code from the framework. However, to make it simple, we just leave it for now.

The mock API is replaced by ```get_property_data<T>```. It's a *generic function* that returns a data with type ```T```. See more details about *generic* in [*Rust by Example*][generics] and [*The Rust Programming Language*][gdt].

There are two *unit tests* here. One is to check if the error throwns as expected. The other is to check if the returned ```AudioObjectID``` is valid when all parameters are valid. Both tests exercise in input and output scopes.

Next, let's implement what we defined above in ```sys``` module. Create a *sys.rs* under *src* and put the following code into *src/sys.rs*:

```rust
use std::mem; // For mem::uninitialized(), mem::size_of
use std::os::raw::c_void;
use std::ptr; // For ptr::null()

//  Type Aliases
// ==============================================================================
// MacTypes.h
// -------------------------
// https://developer.apple.com/documentation/kernel/osstatus?language=objc
pub type OSStatus = i32;

// AudioHardwareBase.h
// -------------------------
// https://developer.apple.com/documentation/coreaudio/audioobjectid?language=objc
pub type AudioObjectID = u32;

// https://developer.apple.com/documentation/coreaudio/audioobjectpropertyaddress/1422175-mselector?language=objc
type AudioObjectPropertySelector = u32;
// https://developer.apple.com/documentation/coreaudio/audioobjectpropertyscope?language=objc
type AudioObjectPropertyScope = u32;
// https://developer.apple.com/documentation/coreaudio/audioobjectpropertyelement?language=objc
type AudioObjectPropertyElement = u32;

// https://developer.apple.com/documentation/coreaudio/audioobjectpropertyaddress?language=objc
#[repr(C)] // Specify data layout in the same way as C does.
pub struct AudioObjectPropertyAddress {
    pub mSelector: AudioObjectPropertySelector,
    pub mScope: AudioObjectPropertyScope,
    pub mElement: AudioObjectPropertyElement,
}

// https://developer.apple.com/documentation/coreaudio/1494531-anonymous/kaudiohardwarenoerror
const kAudioHardwareNoError: OSStatus = 0;
// https://developer.apple.com/documentation/coreaudio/1494531-anonymous/kaudiohardwarebadobjecterror
// 0x'!obj' = 0x216F626A = 560947818
pub const kAudioHardwareBadObjectError: OSStatus = 560947818;

// https://developer.apple.com/documentation/coreaudio/1494461-anonymous/kaudioobjectunknown
pub const kAudioObjectUnknown: AudioObjectID = 0;

// https://developer.apple.com/documentation/coreaudio/1494464-anonymous/kaudioobjectpropertyscopeglobal
// 0x'glob' = 0x676C6F62 = 1735159650
pub const kAudioObjectPropertyScopeGlobal: AudioObjectPropertyScope = 1735159650;
// https://developer.apple.com/documentation/coreaudio/1494464-anonymous/kaudioobjectpropertyelementmaster
pub const kAudioObjectPropertyElementMaster: AudioObjectPropertyElement = 0;

// AudioHardware.h
// -------------------------
// https://developer.apple.com/documentation/coreaudio/1545873-anonymous/kaudioobjectsystemobject
pub const kAudioObjectSystemObject: AudioObjectID = 1;

// https://developer.apple.com/documentation/coreaudio/1545886-anonymous/kaudiohardwarepropertydefaultinputdevice
// 0x'dIn ' = 0x64496E20 = 1682533920
pub const kAudioHardwarePropertyDefaultInputDevice: AudioObjectPropertySelector = 1682533920;
// https://developer.apple.com/documentation/coreaudio/1545886-anonymous/kaudiohardwarepropertydefaultoutputdevice
// 0x'dOut' = 0x644F7574 = 1682929012
pub const kAudioHardwarePropertyDefaultOutputDevice: AudioObjectPropertySelector = 1682929012;

#[cfg(target_os = "macos")] // The function is only included on macOS.
#[link(name = "CoreAudio", kind = "framework")] // Link dynamically to CoreAudio.
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

pub fn get_property_data<T> (
    id: AudioObjectID,
    address: &AudioObjectPropertyAddress,
) -> Result<T, OSStatus> {
    // Using `mem::uninitialized()` to bypasses memory-initialization checks.
    let mut data: T = unsafe { mem::uninitialized() };
    let mut size = mem::size_of_val(&data) as u32; // Cast usize to u32.
    let status: OSStatus = unsafe {
        AudioObjectGetPropertyData(
            id,
            // Cast AudioObjectPropertyAddress ref to
            // raw AudioObjectPropertyAddress pointer
            address as *const AudioObjectPropertyAddress,
            0,
            ptr::null(),
            // Cast u32 ref to a raw u32 pointer.
            &mut size as *mut u32,
            // Cast T ref to a raw T pointer first,
            // and then cast raw T pointer to void pointer.
            &mut data as *mut T as *mut c_void,
        )
    };
    if status == kAudioHardwareNoError {
      Ok(data)
    } else {
      Err(status)
    }
}
```

In *src/sys.rs*, we bind all types such as ```OSStatus```, ```AudioObjectID``` manually. We declare type aliases to give existing types another names.
For example, ```type OSStatus = i32``` gives ```i32``` another name ```OSStatus``` and ```type AudioObjectID = u32``` gives ```u32``` another name ```AudioObjectID```. Check [Type Alias][ta] for more details.

In fact, ```OSStatus``` or ```AudioObjectID``` are also just type aliases defined by ```typedef``` in framework's header files, such as ```typedef UInt32 AudioObjectID;``` or ```typedef SInt32 OSStatus;```. Check them in [MacTypes.h][mt], [AudioHardwareBase.h][ahb], and [AudioHardware.h][ah].

The enum variables defined in framework's header files like ```kAudioHardwareNoError``` or ```kAudioObjectUnknown``` are defined as [constants][const] here.

For the ```struct``` that is interoperable with *C* API, we need to add ```#[repr(C)]``` *attribute*. It specifies the data layout should be aligned in *C*'s style and prevents its memory layout from being mangled by *Rust* compiler for optimization. See more details in [*The Rust Reference*][layout] and [*The Rustonomicon*][reprc].

The key API ```get_property_data<T>``` is a *generic function*, that returns a ```data``` with type ```T```. It's implemented based on the native C API: [```AudioObjectGetPropertyData```][gpd]. Given necessary parameters, we can query a ```data``` in any types with [```AudioObjectGetPropertyData```][gpd] since the ```data``` is reinterpreted as a ```void *``` pointer. That's why we use a *generic function* to wrap it. We should be able to query a ```data``` in any types.

Declaring a variable ```data``` in type *T* without *initialized* value is illegal in *Rust*, so we use [```std::mem::uninitialized()```][memuninit] to bypass the check. The ```data``` is used to store the queried data. The byte size of ```data``` is can be calculated by [std::mem::size_of_val][memsize].

To call C API with *pointer* parameters, we need to cast types by ourselves. Keyword ```as``` is for safe casts. [```AudioObjectGetPropertyData```][gpd] takes the following pointers as parameters (in *C* style):
- ```AudioObjectID inObjectID```
- ```const AudioObjectPropertyAddress* inAddress```
- ```const void* inQualifierData``` (usually ```nullptr```)
- ```UInt32* ioDataSize```
- ```void* outData```

The ```*const T``` and ```*mut T``` in *Rust* are similar to *C*'s ```const T*```(a **variable** pointer pointing to a **constant** ```T```) and ```T*```(a **variable** pointer pointing to a **variable** ```T```), respectively.

The second parameter of [```AudioObjectGetPropertyData```][gpd] is a pointer pointing to a constant ```AudioObjectPropertyAddress```, so we apply ```as``` to cast ```AudioObjectPropertyAddress``` reference: ```&AudioObjectPropertyAddress``` to a raw pointer pointing to a ```const AudioObjectPropertyAddress```: ```*const AudioObjectPropertyAddress``` directly by ```address as *const AudioObjectPropertyAddress```.

The fourth parameter of [```AudioObjectGetPropertyData```][gpd] is usually a *NULL* pointer, so we can just pass [```std::ptr::null()```][nullptr].

The last second parameter of [```AudioObjectGetPropertyData```][gpd] is a *32-bit unsigned int* pointer, indicating the size of ```outData``` we have. The returned type of [std::mem::size_of_val][memsize] is ```usize```, so we cast ```size``` from ```usize``` to ```u32``` in advance and pass a raw pointer ```*mut u32``` that is casted from ```&mut size```.

The last parameter of [```AudioObjectGetPropertyData```][gpd] is a *void* pointer and our data will be put in that address. [```std::os::raw::c_void```][void] is used to construct *C* *void* pointers and it is only useful as a pointer target. It's illegal to directly cast from a reference to a *void* pointer in *Rust*. To get the *void* pointer from ```T``` reference, we need:
1. Cast from a ```T``` reference: ```&mut data``` to a raw ```T``` pointer by ```&mut data as *mut T```
2. Cast from a raw ```T``` pointer to a *void* pointer by ```*mut T as *mut c_void```

Finally, let's look how to link the function ```fn AudioObjectGetPropertyData(...)``` to native API [```AudioObjectGetPropertyData```][gpd].

The ```#[cfg(target_os = "macos")]``` is a [configuration option][cfg] that compiles code only when the operating system is *Mac OS*. The ```AudioObjectGetPropertyData``` is platform-dependent API on *Mac OS*, so we use this attribute to mark it. Another common option is ```#[cfg(test)]```. That's what we used in [previous post][prev]. It indicates that the following code is only compiled for the test harness.

The ```#[link(name = "CoreAudio", kind = "framework")]``` is a [*FFI attribute*][ffiattr]. The ```link``` attribute on ```extern``` blocks define how the functions in block link to the native libraries. The ```CoreAudio```, assigned to ```name```, is the name of the native library that we're linking to. The ```framework```, assigned to ```kind```, is the type of native library that the compiler is linking to. The different ```kind``` values are meant to differentiate how the native library participates in linkage. Note that ```framework``` are only available on macOS targets. For more details about *linking*, read [here][linking].

The rest of code in the ```extern``` block is similar to what we did in [Calling Native C APIs from Rust][callingC].

We don't change the public interface in *src/lib.rs*, so we can leave *tests/integration.rs* and *src/main.rs* as they are.

*tests/integration.rs*:
```rust
extern crate rust_audio_lib; // Introduce the `rust_audio_lib` library crate.
use rust_audio_lib::utils; // Refer to `utils` module

#[test]
fn utils_get_default_device_id() {
    assert!(utils::get_default_device_id(utils::Scope::Input).is_ok());
    assert!(utils::get_default_device_id(utils::Scope::Output).is_ok());
}
```

*src/main.rs*:
```rust
extern crate rust_audio_lib;
use rust_audio_lib::utils; // Refer to `utils` module

fn show_result(scope: utils::Scope) {
    let side = if scope == utils::Scope::Input { "input" } else { "output" };
    match utils::get_default_device_id(scope) {
        Ok(id) => {
            println!("default {} device id: {}", side, id);
        },
        Err(error) => {
            println!("Failed to get {} device id. Error: {}", side, error);
        }
    }
}

fn main() {
    show_result(utils::Scope::Input);
    show_result(utils::Scope::Output);
}
```

Finnaly, it's time to run them:

```
$ cargo test
   Compiling rust_audio_lib v0.1.0 (file:///<path>/<to>/<projects>/rust_audio_lib)
warning: constant item is never used: `kAudioHardwareBadObjectError`
  --> src/sys.rs:36:1
   |
36 | pub const kAudioHardwareBadObjectError: OSStatus = 560947818;
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: #[warn(dead_code)] on by default

warning: constant item is never used: `kAudioObjectUnknown`
  --> src/sys.rs:39:1
   |
39 | pub const kAudioObjectUnknown: AudioObjectID = 0;
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: structure field `mSelector` should have a snake case name such as `m_selector`
  --> src/sys.rs:27:5
   |
27 |     pub mSelector: AudioObjectPropertySelector,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: #[warn(non_snake_case)] on by default

warning: structure field `mScope` should have a snake case name such as `m_scope`
  --> src/sys.rs:28:5
   |
28 |     pub mScope: AudioObjectPropertyScope,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: structure field `mElement` should have a snake case name such as `m_element`
  --> src/sys.rs:29:5
   |
29 |     pub mElement: AudioObjectPropertyElement,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `kAudioHardwareNoError` should have an upper case name such as `K_AUDIO_HARDWARE_NO_ERROR`
  --> src/sys.rs:33:1
   |
33 | const kAudioHardwareNoError: OSStatus = 0;
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: #[warn(non_upper_case_globals)] on by default

warning: constant `kAudioHardwareBadObjectError` should have an upper case name such as `K_AUDIO_HARDWARE_BAD_OBJECT_ERROR`
  --> src/sys.rs:36:1
   |
36 | pub const kAudioHardwareBadObjectError: OSStatus = 560947818;
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `kAudioObjectUnknown` should have an upper case name such as `K_AUDIO_OBJECT_UNKNOWN`
  --> src/sys.rs:39:1
   |
39 | pub const kAudioObjectUnknown: AudioObjectID = 0;
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `kAudioObjectPropertyScopeGlobal` should have an upper case name such as `K_AUDIO_OBJECT_PROPERTY_SCOPE_GLOBAL`
  --> src/sys.rs:43:1
   |
43 | pub const kAudioObjectPropertyScopeGlobal: AudioObjectPropertyScope = 1735159650;
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `kAudioObjectPropertyElementMaster` should have an upper case name such as `K_AUDIO_OBJECT_PROPERTY_ELEMENT_MASTER`
  --> src/sys.rs:45:1
   |
45 | pub const kAudioObjectPropertyElementMaster: AudioObjectPropertyElement = 0;
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `kAudioObjectSystemObject` should have an upper case name such as `K_AUDIO_OBJECT_SYSTEM_OBJECT`
  --> src/sys.rs:50:1
   |
50 | pub const kAudioObjectSystemObject: AudioObjectID = 1;
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `kAudioHardwarePropertyDefaultInputDevice` should have an upper case name such as `K_AUDIO_HARDWARE_PROPERTY_DEFAULT_INPUT_DEVICE`
  --> src/sys.rs:54:1
   |
54 | pub const kAudioHardwarePropertyDefaultInputDevice: AudioObjectPropertySelector = 1682533920;
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `kAudioHardwarePropertyDefaultOutputDevice` should have an upper case name such as `K_AUDIO_HARDWARE_PROPERTY_DEFAULT_OUTPUT_DEVICE`
  --> src/sys.rs:57:1
   |
57 | pub const kAudioHardwarePropertyDefaultOutputDevice: AudioObjectPropertySelector = 1682929012;
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: structure field `mSelector` should have a snake case name such as `m_selector`
  --> src/sys.rs:27:5
   |
27 |     pub mSelector: AudioObjectPropertySelector,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: #[warn(non_snake_case)] on by default

warning: structure field `mScope` should have a snake case name such as `m_scope`
  --> src/sys.rs:28:5
   |
28 |     pub mScope: AudioObjectPropertyScope,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: structure field `mElement` should have a snake case name such as `m_element`
  --> src/sys.rs:29:5
   |
29 |     pub mElement: AudioObjectPropertyElement,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `kAudioHardwareNoError` should have an upper case name such as `K_AUDIO_HARDWARE_NO_ERROR`
  --> src/sys.rs:33:1
   |
33 | const kAudioHardwareNoError: OSStatus = 0;
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: #[warn(non_upper_case_globals)] on by default

warning: constant `kAudioHardwareBadObjectError` should have an upper case name such as `K_AUDIO_HARDWARE_BAD_OBJECT_ERROR`
  --> src/sys.rs:36:1
   |
36 | pub const kAudioHardwareBadObjectError: OSStatus = 560947818;
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `kAudioObjectUnknown` should have an upper case name such as `K_AUDIO_OBJECT_UNKNOWN`
  --> src/sys.rs:39:1
   |
39 | pub const kAudioObjectUnknown: AudioObjectID = 0;
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `kAudioObjectPropertyScopeGlobal` should have an upper case name such as `K_AUDIO_OBJECT_PROPERTY_SCOPE_GLOBAL`
  --> src/sys.rs:43:1
   |
43 | pub const kAudioObjectPropertyScopeGlobal: AudioObjectPropertyScope = 1735159650;
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `kAudioObjectPropertyElementMaster` should have an upper case name such as `K_AUDIO_OBJECT_PROPERTY_ELEMENT_MASTER`
  --> src/sys.rs:45:1
   |
45 | pub const kAudioObjectPropertyElementMaster: AudioObjectPropertyElement = 0;
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `kAudioObjectSystemObject` should have an upper case name such as `K_AUDIO_OBJECT_SYSTEM_OBJECT`
  --> src/sys.rs:50:1
   |
50 | pub const kAudioObjectSystemObject: AudioObjectID = 1;
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `kAudioHardwarePropertyDefaultInputDevice` should have an upper case name such as `K_AUDIO_HARDWARE_PROPERTY_DEFAULT_INPUT_DEVICE`
  --> src/sys.rs:54:1
   |
54 | pub const kAudioHardwarePropertyDefaultInputDevice: AudioObjectPropertySelector = 1682533920;
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `kAudioHardwarePropertyDefaultOutputDevice` should have an upper case name such as `K_AUDIO_HARDWARE_PROPERTY_DEFAULT_OUTPUT_DEVICE`
  --> src/sys.rs:57:1
   |
57 | pub const kAudioHardwarePropertyDefaultOutputDevice: AudioObjectPropertySelector = 1682929012;
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

    Finished dev [unoptimized + debuginfo] target(s) in 0.89s
     Running target/debug/deps/rust_audio_lib-e7d8afc4518ed66d

running 2 tests
test utils::utils_get_property_data_invalid_id ... ok
test utils::utils_get_property_data ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

     Running target/debug/deps/rust_audio_lib-181ddafe8a1703f3

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

     Running target/debug/deps/integration-c1549a32ceba0911

running 1 test
test utils_get_default_device_id ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

   Doc-tests rust_audio_lib

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

```

All the tests are passed!

```
$ cargo clean
$ cargo build
   Compiling rust_audio_lib v0.1.0 (file:///<path>/<to>/<projects>/rust_audio_lib)
warning: constant item is never used: `kAudioHardwareBadObjectError`
  --> src/sys.rs:36:1
   |
36 | pub const kAudioHardwareBadObjectError: OSStatus = 560947818;
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: #[warn(dead_code)] on by default

warning: constant item is never used: `kAudioObjectUnknown`
  --> src/sys.rs:39:1
   |
39 | pub const kAudioObjectUnknown: AudioObjectID = 0;
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: structure field `mSelector` should have a snake case name such as `m_selector`
  --> src/sys.rs:27:5
   |
27 |     pub mSelector: AudioObjectPropertySelector,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: #[warn(non_snake_case)] on by default

warning: structure field `mScope` should have a snake case name such as `m_scope`
  --> src/sys.rs:28:5
   |
28 |     pub mScope: AudioObjectPropertyScope,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: structure field `mElement` should have a snake case name such as `m_element`
  --> src/sys.rs:29:5
   |
29 |     pub mElement: AudioObjectPropertyElement,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `kAudioHardwareNoError` should have an upper case name such as `K_AUDIO_HARDWARE_NO_ERROR`
  --> src/sys.rs:33:1
   |
33 | const kAudioHardwareNoError: OSStatus = 0;
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: #[warn(non_upper_case_globals)] on by default

warning: constant `kAudioHardwareBadObjectError` should have an upper case name such as `K_AUDIO_HARDWARE_BAD_OBJECT_ERROR`
  --> src/sys.rs:36:1
   |
36 | pub const kAudioHardwareBadObjectError: OSStatus = 560947818;
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `kAudioObjectUnknown` should have an upper case name such as `K_AUDIO_OBJECT_UNKNOWN`
  --> src/sys.rs:39:1
   |
39 | pub const kAudioObjectUnknown: AudioObjectID = 0;
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `kAudioObjectPropertyScopeGlobal` should have an upper case name such as `K_AUDIO_OBJECT_PROPERTY_SCOPE_GLOBAL`
  --> src/sys.rs:43:1
   |
43 | pub const kAudioObjectPropertyScopeGlobal: AudioObjectPropertyScope = 1735159650;
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `kAudioObjectPropertyElementMaster` should have an upper case name such as `K_AUDIO_OBJECT_PROPERTY_ELEMENT_MASTER`
  --> src/sys.rs:45:1
   |
45 | pub const kAudioObjectPropertyElementMaster: AudioObjectPropertyElement = 0;
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `kAudioObjectSystemObject` should have an upper case name such as `K_AUDIO_OBJECT_SYSTEM_OBJECT`
  --> src/sys.rs:50:1
   |
50 | pub const kAudioObjectSystemObject: AudioObjectID = 1;
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `kAudioHardwarePropertyDefaultInputDevice` should have an upper case name such as `K_AUDIO_HARDWARE_PROPERTY_DEFAULT_INPUT_DEVICE`
  --> src/sys.rs:54:1
   |
54 | pub const kAudioHardwarePropertyDefaultInputDevice: AudioObjectPropertySelector = 1682533920;
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `kAudioHardwarePropertyDefaultOutputDevice` should have an upper case name such as `K_AUDIO_HARDWARE_PROPERTY_DEFAULT_OUTPUT_DEVICE`
  --> src/sys.rs:57:1
   |
57 | pub const kAudioHardwarePropertyDefaultOutputDevice: AudioObjectPropertySelector = 1682929012;
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

    Finished dev [unoptimized + debuginfo] target(s) in 0.59s
$ cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.02s
     Running `target/debug/rust_audio_lib`
default device id: 48
default device id: 55
```

And we get the ids of default input and output device successfully!

However, there are lots of warnings when we run it. Most of them are naming issues like: Considering renaming `mSelector` to `m_selector` or `kAudioObjectUnknown` to `K_AUDIO_OBJECT_UNKNOWN`. This is a conflict of preferred naming between *C* and *Rust*. I prefer to remain the *C* style naming of the corresponding type alias because I can recognize where they are from. To disable the warnings, we could add ```#![allow(non_snake_case, non_upper_case_globals)]``` at the beginning in *sys.rs*.

```rust
#![allow(non_snake_case, non_upper_case_globals)]

use std::mem; // For mem::uninitialized(), mem::size_of
use std::os::raw::c_void;
use std::ptr; // For ptr::null()

...
...
```

Another type of warnings are unused variables: ```kAudioHardwareBadObjectError``` and ```kAudioObjectUnknown```. They are just used in *unit tests*, so we can mark ```#[cfg(test)]``` on the line before their declarations.

```rust
...
...

// https://developer.apple.com/documentation/coreaudio/1494531-anonymous/kaudiohardwarenoerror
const kAudioHardwareNoError: OSStatus = 0;
// https://developer.apple.com/documentation/coreaudio/1494531-anonymous/kaudiohardwarebadobjecterror
// 0x'!obj' = 0x216F626A = 560947818
#[cfg(test)]
pub const kAudioHardwareBadObjectError: OSStatus = 560947818;

// https://developer.apple.com/documentation/coreaudio/1494461-anonymous/kaudioobjectunknown
#[cfg(test)]
pub const kAudioObjectUnknown: AudioObjectID = 0;

...
...
```

All the warnings will be disappeared when running it again:
```
$ cargo clean
$ cargo test
   Compiling rust_audio_lib v0.1.0 (file:///<path>/<to>/<projects>/rust_audio_lib)
    Finished dev [unoptimized + debuginfo] target(s) in 0.87s
     Running target/debug/deps/rust_audio_lib-e7d8afc4518ed66d

running 2 tests
test utils::utils_get_property_data_invalid_id ... ok
test utils::utils_get_property_data ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

     Running target/debug/deps/rust_audio_lib-181ddafe8a1703f3

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

     Running target/debug/deps/integration-c1549a32ceba0911

running 1 test
test utils_get_default_device_id ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

   Doc-tests rust_audio_lib

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

```
$ cargo clean
$ cargo build
   Compiling rust_audio_lib v0.1.0 (file:///<path>/<to>/<projects>/rust_audio_lib)
    Finished dev [unoptimized + debuginfo] target(s) in 0.60s
$ cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.02s
     Running `target/debug/rust_audio_lib`
default device id: 48
default device id: 55
```

## Refactoring

There are few points I'd like to improve:
1. The type alias from native library in ```sys``` module may be reused by other modules, but ```get_property_data``` may be used only in ```utils``` module. It makes sense to me to move ```get_property_data``` from ```sys``` module to ```utils``` module.
2. We should use a custom error type instead of using ```OSStatus```. The ```OSStatus``` is from native system and we should not expect library user understand what it means.
3. One error is **hiding** behind success state. The returned ```AudioObjectID``` in success state may be a ```kAudioObjectUnknown```. That's why we check ```assert_ne!(get_property_data::<AudioObjectID>(...).unwrap(), kAudioObjectUnknown)``` in ```utils_get_property_data()```. This is an error! We should throw an error instead of returning a value.
4. Return a custom *device id* instead of ```AudioObjectID```. ```AudioObjectID``` is native type from the system. We shouldn't use it as a returned type in a custom library. If this library will be called from *Python*, *python* doesn't know what ```AudioObjectID``` means. Furthermore, using a custom interface will give us a room to change the returned type anytime.(If ```utils``` module is only used internally, there is no need to do this.)
5. Don't bring all the functions and types in ```sys``` into the scope of ```utils``` without namespace. It makes us confused about which variable we are using. For example:
```rust
fn foo (status: OSStatus) {
    match status {
        kAudioHardwareBadObjectError => {
            println!("status is {}", kAudioHardwareBadObjectError)
        }
        _ => { /* never reach this match arm! */ }
    }
}
```
The ```kAudioHardwareBadObjectError``` is not the ```const``` variable defined in ```sys```. It's a **new** variable introduced in the ```kAudioHardwareBadObjectError => { ... }``` *match arm*, which hides the ```kAudioHardwareBadObjectError``` defined in ```sys```, just like declaring a variable in an inner scope(see [here][r4cppp].). We should replace ```use super::sys::*``` with ```use super::sys``` and use ```sys::kAudioHardwareBadObjectError``` instead. It's clearer to indicate what variables we are using.

To address what we mentioned, the code can be rewritten into:

*src/sys.rs*
```rust
#![allow(non_snake_case, non_upper_case_globals)]

use std::os::raw::c_void;

//  Type Aliases
// ==============================================================================
// MacTypes.h
// -------------------------
// https://developer.apple.com/documentation/kernel/osstatus?language=objc
pub type OSStatus = i32;

// AudioHardwareBase.h
// -------------------------
// https://developer.apple.com/documentation/coreaudio/audioobjectid?language=objc
pub type AudioObjectID = u32;

// https://developer.apple.com/documentation/coreaudio/audioobjectpropertyaddress/1422175-mselector?language=objc
type AudioObjectPropertySelector = u32;
// https://developer.apple.com/documentation/coreaudio/audioobjectpropertyscope?language=objc
type AudioObjectPropertyScope = u32;
// https://developer.apple.com/documentation/coreaudio/audioobjectpropertyelement?language=objc
type AudioObjectPropertyElement = u32;

// https://developer.apple.com/documentation/coreaudio/audioobjectpropertyaddress?language=objc
#[repr(C)] // Specify data layout in the same way as C does.
pub struct AudioObjectPropertyAddress {
    pub mSelector: AudioObjectPropertySelector,
    pub mScope: AudioObjectPropertyScope,
    pub mElement: AudioObjectPropertyElement,
}

// https://developer.apple.com/documentation/coreaudio/1494531-anonymous/kaudiohardwarenoerror
pub const kAudioHardwareNoError: OSStatus = 0;
// https://developer.apple.com/documentation/coreaudio/1494531-anonymous/kaudiohardwarebadobjecterror
// 0x'!obj' = 0x216F626A = 560947818
pub const kAudioHardwareBadObjectError: OSStatus = 560947818;

// https://developer.apple.com/documentation/coreaudio/1494461-anonymous/kaudioobjectunknown
pub const kAudioObjectUnknown: AudioObjectID = 0;

// https://developer.apple.com/documentation/coreaudio/1494464-anonymous/kaudioobjectpropertyscopeglobal
// 0x'glob' = 0x676C6F62 = 1735159650
pub const kAudioObjectPropertyScopeGlobal: AudioObjectPropertyScope = 1735159650;
// https://developer.apple.com/documentation/coreaudio/1494464-anonymous/kaudioobjectpropertyelementmaster
pub const kAudioObjectPropertyElementMaster: AudioObjectPropertyElement = 0;

// AudioHardware.h
// -------------------------
// https://developer.apple.com/documentation/coreaudio/1545873-anonymous/kaudioobjectsystemobject
pub const kAudioObjectSystemObject: AudioObjectID = 1;

// https://developer.apple.com/documentation/coreaudio/1545886-anonymous/kaudiohardwarepropertydefaultinputdevice
// 0x'dIn ' = 0x64496E20 = 1682533920
pub const kAudioHardwarePropertyDefaultInputDevice: AudioObjectPropertySelector = 1682533920;
// https://developer.apple.com/documentation/coreaudio/1545886-anonymous/kaudiohardwarepropertydefaultoutputdevice
// 0x'dOut' = 0x644F7574 = 1682929012
pub const kAudioHardwarePropertyDefaultOutputDevice: AudioObjectPropertySelector = 1682929012;

#[cfg(target_os = "macos")] // The function is only included on macOS.
#[link(name = "CoreAudio", kind = "framework")] // Link dynamically to CoreAudio.
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

*src/lib.rs*
```rust
mod sys; // Module contains types and functions of theexternal libraries

pub mod utils {
    use std::mem; // For mem::uninitialized(), mem::size_of_val()
    use std::os::raw::c_void;
    use std::ptr; // For ptr::null()
    use super::sys; // Bring `sys` module into scope

    #[derive(PartialEq)] // Enable comparison
    pub enum Scope {
        Input,
        Output,
    }

    #[derive(Debug, PartialEq)] // Using Debug for std::fmt::Debug
    pub enum Error {
        NoDevice,
        InvalidParameters,
    }

    impl From<sys::OSStatus> for Error {
        fn from(status: sys::OSStatus) -> Error {
            match status {
                sys::kAudioHardwareBadObjectError => Error::InvalidParameters,
                s => panic!("Unknown error status: {}", s),
            }
        }
    }

    pub type DeviceId = i32;

    const DEFAULT_INPUT_DEVICE_PROPERTY_ADDRESS: sys::AudioObjectPropertyAddress =
        sys::AudioObjectPropertyAddress {
            mSelector: sys::kAudioHardwarePropertyDefaultInputDevice,
            mScope: sys::kAudioObjectPropertyScopeGlobal,
            mElement: sys::kAudioObjectPropertyElementMaster,
        };

    const DEFAULT_OUTPUT_DEVICE_PROPERTY_ADDRESS: sys::AudioObjectPropertyAddress =
        sys::AudioObjectPropertyAddress {
            mSelector: sys::kAudioHardwarePropertyDefaultOutputDevice,
            mScope: sys::kAudioObjectPropertyScopeGlobal,
            mElement: sys::kAudioObjectPropertyElementMaster,
        };

    pub fn get_default_device_id(scope: Scope) -> Result<DeviceId, Error> {
        let address: &sys::AudioObjectPropertyAddress = if scope == Scope::Input {
            &DEFAULT_INPUT_DEVICE_PROPERTY_ADDRESS
        } else {
            &DEFAULT_OUTPUT_DEVICE_PROPERTY_ADDRESS
        };
        let id = get_property_data::<sys::AudioObjectID>(
            sys::kAudioObjectSystemObject,
            address
        )?;
        if id == sys::kAudioObjectUnknown {
            Err(Error::NoDevice)
        } else {
            Ok(to_device_id(id))
        }
    }

    fn to_device_id(id: sys::AudioObjectID) -> DeviceId {
        id as DeviceId
    }

    fn get_property_data<T> (
        id: sys::AudioObjectID,
        address: &sys::AudioObjectPropertyAddress,
    ) -> Result<T, Error> {
        // Use `mem::uninitialized()` to bypasses memory-initialization checks
        let mut data: T = unsafe { mem::uninitialized() };
        let mut size = mem::size_of_val(&data);
        let status = audio_object_get_property_data(
            id,
            address,
            &mut size,
            &mut data
        );
        convert_to_result(status)?;
        Ok(data)
    }

    fn audio_object_get_property_data<T>(
        id: sys::AudioObjectID,
        address: &sys::AudioObjectPropertyAddress,
        size: *mut usize,
        data: *mut T,
    ) -> sys::OSStatus {
        unsafe {
            sys::AudioObjectGetPropertyData(
                id,
                address, // as `*const sys::AudioObjectPropertyAddress` automatically
                0,
                ptr::null(),
                size as *mut u32, // Cast raw usize pointer to raw u32 pointer
                data as *mut c_void, // Cast raw T pointer to void pointer
            )
        }
    }

    fn convert_to_result(status: sys::OSStatus) -> Result<(), Error> {
        match status {
            sys::kAudioHardwareNoError => Ok(()),
            e => Err(e.into()),
        }
    }

    #[cfg(test)] // Indicates this is only included when running `cargo test`
    mod tests { // A private internal submodule in utils
        use super::*; // To use the functions in utils

        #[test] // Built only within `cargo test`.
        fn utils_get_property_data_invalid_id() {
            // Invalid AudioObjectID with valid input adrress.
            assert_eq!(get_property_data::<sys::AudioObjectID>(
                        sys::kAudioObjectUnknown,
                        &DEFAULT_INPUT_DEVICE_PROPERTY_ADDRESS
                    ).unwrap_err(), Error::InvalidParameters);
            // Invalid AudioObjectID with valid output adrress.
            assert_eq!(get_property_data::<sys::AudioObjectID>(
                        sys::kAudioObjectUnknown,
                        &DEFAULT_OUTPUT_DEVICE_PROPERTY_ADDRESS
                    ).unwrap_err(), Error::InvalidParameters);
        }

        #[test] // Built only within `cargo test`.
        fn utils_get_property_data() {
            // Check the default input device id is valid.
            assert!(get_property_data::<sys::AudioObjectID>(
                        sys::kAudioObjectSystemObject,
                        &DEFAULT_INPUT_DEVICE_PROPERTY_ADDRESS
                    ).is_ok());
            // Check the default output device id is valid.
            assert!(get_property_data::<sys::AudioObjectID>(
                        sys::kAudioObjectSystemObject,
                        &DEFAULT_OUTPUT_DEVICE_PROPERTY_ADDRESS
                    ).is_ok());
        }
    }
}
```

We use [```?``` operator to propagate errors][qop] here. If the ```Result``` value returned from the function before ```?``` operator is an ```Err```, then the ```Err``` will be returned immediately as what early return works. If the ```Result``` is an ```Ok```, then the value inside the ```Ok``` will be returned and the program will keep going.

To convert the ```OSStatus``` to our custom ```Error```, we implement [```std::convert::From```][conv-from] trait so we can get an ```Error``` from ```e.into()``` where ```e```'s type is ```OSStatus```.

On the other hand, we use automatic casting from reference to raw pointer (see [here][refandptr] for more details) to shorten our code:
- ```&sys::AudioObjectPropertyAddress``` to ```*const sys::AudioObjectPropertyAddress``` (by ```address: &sys::AudioObjectPropertyAddress```)
- ```&mut usize``` to ``` *mut usize``` (by ```&mut size```)
- ```&mut T``` to ```*mut T``` (by ```&mut data```)

To know more about raw pointer, please read the related chapters in [first edition][refandptr] and [second edition][unsafe] of *The Rust Programming Language* listed on [Raw Pointers][rawptr]. The details about casting between types can be found [here][cast].

After refactoring, we need to modify *src/main.rs* to use ```{:?}``` to show the debug message of ```utils::Error```:

```rust
...

fn show_result(scope: utils::Scope) {
    let side = if scope == utils::Scope::Input { "input" } else { "output" };
    match utils::get_default_device_id(scope) {
        Ok(id) => {
            println!("default {} device id: {}", side, id);
        },
        Err(error) => {
            println!("Failed to get {} device id. Error: {:?}", side, error);
        }
    }
}

...
```

If we want to use the normal display ```{}``` for ```utils::Error```, then we need to implement ```std::fmt::Display``` trait by our own.

Finnaly, we can run ```cargo test``` and ```cargo run``` to check if it works. The result will be same as what it shows above.

Further, instead of throwing errors, actually we could just *assert* parameters are valid by ```assert!(...)```. If the assertion fails, it will ```panic```. We could add an assertion in ```src/lib.rs``` like:

```rust

...

fn get_property_data<T> (
    id: sys::AudioObjectID,
    address: &sys::AudioObjectPropertyAddress,
) -> Result<T, Error> {
    assert!(id != sys::kAudioObjectUnknown, "Bad AudioObjectID!");
    // Use `mem::uninitialized()` to bypasses memory-initialization checks
    let mut data: T = unsafe { mem::uninitialized() };
    let mut size = mem::size_of_val(&data);
    let status = audio_object_get_property_data(
        id,
        address,
        &mut size,
        &mut data
    );
    convert_to_result(status)?;
    Ok(data)
}

...

```

and add an attribute: [```#[should_panic]```][testpanic] to the test to catch the ```panic```:

```rust

...

#[test] // Built only within `cargo test`.
#[should_panic(expected = "Bad")]
fn utils_get_property_data_invalid_id() {
    // Invalid AudioObjectID with valid input adrress.
    assert_eq!(get_property_data::<sys::AudioObjectID>(
                sys::kAudioObjectUnknown,
                &DEFAULT_INPUT_DEVICE_PROPERTY_ADDRESS
            ).unwrap_err(), Error::InvalidParameters);
    // Invalid AudioObjectID with valid output adrress.
    assert_eq!(get_property_data::<sys::AudioObjectID>(
                sys::kAudioObjectUnknown,
                &DEFAULT_OUTPUT_DEVICE_PROPERTY_ADDRESS
            ).unwrap_err(), Error::InvalidParameters);
}

...

```

It will let library users know they pass the wrong arguments.

## References
- [AudioBehaviorCheck][abc]
- OSX/CoreAudio
  - [OSStatus][oss]
  - [AudioObjectID][aoid]
  - [AudioObjectGetPropertyData][gpd]
  - [MacTypes.h][mt]
  - [AudioHardwareBase.h][ahb]
  - [AudioHardware.h][ah]
- [Result][result]
- [Modules][mod]
- Generics
  - [Rust by Example: Generics][generics]
  - [The Rust Programming Language: Generic Data Types][gdt]
- [Type Alias][ta]
- [Constants][const]
- Representation
  - [The Rust Reference: Representation][layout]
  - [The Rustonomicon: Alternative representations][reprc]
- [Conditional compilation][cfg]
- FFI Linking
  - [The Rust Programming Language(first edition): Linking][linking]
  - [The Rust Reference: FFI attributes][ffiattr]
- [The Rust Programming Language: Recoverable Errors with Result][qop]
- [Raw Pointers][rawptr]
  - [The Rust Programming Language(first edition): Raw Pointers][refandptr]
  - [The Rust Programming Language: Unsafe Rust][unsafe]
- [Casting between types][cast]
- [Checking for Panics with ```should_panic```][testpanic]
- Rust APIs
  - [std::mem::uninitialized][memuninit]
  - [std::mem::size_of_val][memsize]
  - [std::ptr::null][nullptr]
  - [std::os::raw::c_void][void]
- [std::convert::From][conv-from]

[callingC]: step2.md "Calling Native C APIs from Rust"
[prev]: step4.md "Testing"

[abc]: https://github.com/ChunMinChang/AudioBehaviorCheck/blob/3da870e4dfa5bd9b3c47ca6e658552dabd8217e4/AudioObjectUtils.cpp#L65-L75 "ABC: AudioObjectUtils::GetDefaultDeviceId"

[aoid]: https://developer.apple.com/documentation/coreaudio/audioobjectid?language=objc "AudioObjectID"
[oss]: https://developer.apple.com/documentation/kernel/osstatus?language=objc "OSStatus"
[gpd]: https://developer.apple.com/documentation/coreaudio/1422524-audioobjectgetpropertydata?language=objc "AudioObjectGetPropertyData"

[mt]: https://github.com/phracker/MacOSX-SDKs/blob/master/MacOSX10.7.sdk/System/Library/Frameworks/CoreServices.framework/Versions/A/Frameworks/CarbonCore.framework/Versions/A/Headers/MacTypes.h "MacTypes.h"
[ahb]: https://github.com/phracker/MacOSX-SDKs/blob/master/MacOSX10.13.sdk/System/Library/Frameworks/CoreAudio.framework/Versions/A/Headers/AudioHardwareBase.h "AudioHardwareBase.h"
[ah]: https://github.com/phracker/MacOSX-SDKs/blob/master/MacOSX10.13.sdk/System/Library/Frameworks/CoreAudio.framework/Versions/A/Headers/AudioHardware.h "AudioHardware.h"

[result]: https://doc.rust-lang.org/book/second-edition/ch09-02-recoverable-errors-with-result.html "The Rust Programming Language: Recoverable Errors with Result"

[mod]: https://doc.rust-lang.org/book/second-edition/ch07-01-mod-and-the-filesystem.html#moving-modules-to-other-files "The Rust Programming Language: Moving Modules to Other Files"

[generics]: https://doc.rust-lang.org/rust-by-example/generics.html "Rust by Example: Generics"
[gdt]: https://doc.rust-lang.org/book/second-edition/ch10-01-syntax.html "The Rust Programming Language: Generic Data Types"

[ta]: https://doc.rust-lang.org/book/second-edition/ch19-04-advanced-types.html#type-aliases-create-type-synonyms "The Rust Programming Language: Type Alias"

[const]: https://doc.rust-lang.org/book/second-edition/ch03-01-variables-and-mutability.html#differences-between-variables-and-constants "The Rust Programming Language: Constants"

[layout]: https://doc.rust-lang.org/reference/type-layout.html#representations "The Rust Reference: Representation"
[reprc]: https://doc.rust-lang.org/nomicon/other-reprs.html "The Rustonomicon: Alternative representations"

[cfg]: https://doc.rust-lang.org/reference/attributes.html#conditional-compilation "The Rust Reference: Conditional compilation"

[linking]: https://doc.rust-lang.org/book/first-edition/ffi.html#linking "The Rust Programming Language(first edition): Linking"
[ffiattr]: https://doc.rust-lang.org/reference/attributes.html#ffi-attributes "The Rust Reference: FFI attributes"

[qop]: https://doc.rust-lang.org/book/second-edition/ch09-02-recoverable-errors-with-result.html#a-shortcut-for-propagating-errors-the--operator "The Rust Programming Language: Recoverable Errors with Result"

[rawptr]: https://doc.rust-lang.org/book/raw-pointers.html "Raw Pointers"
[refandptr]: https://doc.rust-lang.org/book/first-edition/raw-pointers.html#references-and-raw-pointers "The Rust Programming Language(first edition): Raw Pointers"
[unsafe]: https://doc.rust-lang.org/book/second-edition/ch19-01-unsafe-rust.html "The Rust Programming Language: Unsafe Rust"

[cast]: https://doc.rust-lang.org/book/casting-between-types.html "Casting between types"

[testpanic]: https://doc.rust-lang.org/book/second-edition/ch11-01-writing-tests.html#checking-for-panics-with-should_panic "Checking for Panics with should_panic"

[memuninit]: https://doc.rust-lang.org/std/mem/fn.uninitialized.html "std::mem::uninitialized"
[memsize]: https://doc.rust-lang.org/1.25.0/std/mem/fn.size_of_val.html "std::mem::size_of_val"
[nullptr]: https://doc.rust-lang.org/std/ptr/fn.null.html "std::ptr::null"
[void]: https://doc.rust-lang.org/std/os/raw/enum.c_void.html "std::os::raw::c_void"

[conv-from]: https://doc.rust-lang.org/std/convert/trait.From.html "Trait std::convert::From"

[r4cppp]: https://github.com/nrc/r4cppp/blob/master/control%20flow.md#switchmatch "Rust for C++ programmers"
