# Library Based on Platform C API - Part 2: Linking to Platform Library

## Foreign Function Interface: Linking to Platform Library

Recall what we did in [Calling Native C APIs from Rust][callingC]: To call native APIs, *Rust* provides a *FFI* mechanism by keyword ```extern```. We can call the native platform-dependent C APIs in the similar way, with different *link* settings. To make it simple, let's assume we have a ```sys``` module that wraps all the the native types and APIs we need (Using *module* is a common way to organize code in *Rust*. See more details in [modules chapter][mod] of *The Rust Programming Language*.). By the assumption we set, we can rewrite *src/lib.rs* as follows:

```rust
mod sys; // Module contains types and functions of the external libraries.

pub mod utils {
    use super::sys::*; // Bring public types and functions in sys module into scope.

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
        fn test_get_property_data_invalid_id() {
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
        fn test_get_property_data() {
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

Beyond the language level, the process to get the ```AudioObjectID``` is pretty similar to the [C++ example][abc].

All the native types like ```AudioObjectPropertyAddress```, ```kAudioObjectUnknown```,... , etc are all defined in the ```sys``` module and are brought into scope of ```utils``` by ```use super::sys::*;```. The key API: ```get_property_data``` is a wrapper for ```AudioObjectGetPropertyData``` that returns a ```Result<AudioObjectID, OSStatus>```. That is, if the calling is successful, then we will get a ```AudioObjectID``` value. Otherwise, we will get an error with ```OSStatus``` value indicating what the wrong is. It's better to throw an error with clear message instead of a native error code from the underlying framework. However, to make it simple, we just leave it for now.

The mock API is replaced by ```get_property_data<T>```. It's a *generic function* that returns a data with type ```T```. See more details about *generic* in [*Rust by Example*][generics] and [*The Rust Programming Language*][gdt].

There are two *unit tests* here. One is to check if the error throwns as expected. The other is to check if the returned ```AudioObjectID``` is valid when all parameters are valid. Both tests exercise in input and output scopes.

Next, let's implement what we defined above in ```sys``` module. Create a *sys.rs* under *src* and put the following code into *src/sys.rs*:

```rust
use std::mem; // For mem::uninitialized(), mem::size_of_val()
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

In fact, [```OSStatus```][oss] or [```AudioObjectID```][aoid] are also just type aliases defined by ```typedef``` in framework's *C* header files(.h), such as ```typedef UInt32 AudioObjectID;``` or ```typedef SInt32 OSStatus;```. Check them in [MacTypes.h][mt], [AudioHardwareBase.h][ahb], and [AudioHardware.h][ah].

The **enum** variables defined in framework's header files like ```kAudioHardwareNoError``` or ```kAudioObjectUnknown``` are defined as [**constants**][const] here.

For the ```struct``` that is interoperable with *C* API, we need to add ```#[repr(C)]``` *attribute*. It specifies the data layout should be aligned in *C*'s style and prevents its memory layout from being mangled by *Rust* compiler for optimization. See more details in [*The Rust Reference*][layout] and [*The Rustonomicon*][reprc].

The key API ```get_property_data<T>``` is a *generic function* that returns a ```data``` with type ```T```. It's implemented based on the native C API: [```AudioObjectGetPropertyData```][gpd]. Given necessary parameters, we can query a ```data``` in any types with [```AudioObjectGetPropertyData```][gpd] since the ```data``` is reinterpreted as a ```void *``` pointer. That's why we use a *generic function* to wrap it. We should be able to query a ```data``` in any types.

Declaring a variable ```data``` in type ```T``` without *initialized* value is illegal in *Rust*, so we use [```std::mem::uninitialized()```][memuninit] to bypass the check. The ```data``` is used to store the queried data and its byte-size is be calculated by [std::mem::size_of_val][memsize].


The key API for this sample is ```AudioObjectGetPropertyData```. Let's look closer to it:

```AudioObjectGetPropertyData``` in *C*:
```c
OSStatus AudioObjectGetPropertyData(
    AudioObjectID inObjectID,
    const AudioObjectPropertyAddress* inAddress,
    UInt32 inQualifierDataSize,
    const void* inQualifierData,
    UInt32* ioDataSize,
    void* outData
);
```

```AudioObjectGetPropertyData``` in *Rust*:
```rust
fn AudioObjectGetPropertyData(
    inObjectID: AudioObjectID,
    inAddress: *const AudioObjectPropertyAddress,
    inQualifierDataSize: u32,
    inQualifierData: *const c_void,
    ioDataSize: *mut u32,
    outData: *mut c_void,
) -> OSStatus;
```

The ```*const T``` and ```*mut T``` in *Rust* are mapped to *C*'s ```const T*```(a **variable** pointer pointing to a **constant** ```T```) and ```T*```(a **variable** pointer pointing to a **variable** ```T```), respectively. That's how we map this API from Rust to C.

The way we call ```AudioObjectGetPropertyData``` in ```get_property_data<T>``` is:

```rust
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

    ...

}
```

The line ```address as *const AudioObjectPropertyAddress``` is to to cast ```AudioObjectPropertyAddress``` **reference** to
a raw ```AudioObjectPropertyAddress``` **pointer**.
The second parameter of [```AudioObjectGetPropertyData```][gpd] is a pointer pointing to a constant ```AudioObjectPropertyAddress```, so we apply ```as``` to cast ```AudioObjectPropertyAddress``` reference(```&AudioObjectPropertyAddress```) to a raw ```AudioObjectPropertyAddress``` pointer(```*const AudioObjectPropertyAddress```, pointing to a ```const AudioObjectPropertyAddress```).

The fourth parameter of [```AudioObjectGetPropertyData```][gpd] is usually a *NULL* pointer, so we can just pass [```std::ptr::null()```][nullptr] to it and set its size ```0``` to the third parameter.

The line ```&mut size as *mut u32``` is to cast a ```u32``` **reference** to ```u32``` raw **pointer**.
The second last parameter of [```AudioObjectGetPropertyData```][gpd] is a *32-bit unsigned int* pointer, indicating the size of ```data``` we have. The size of ```data``` can be calculated by ```mem::size_of_val(&data)```. The returned type of [std::mem::size_of_val][memsize] is ```usize```, so we cast ```size``` from ```usize``` to ```u32``` in advance and then cast its ```u32``` **reference**(```&mut size```) to a raw ```u32``` **pointer** (```*mut u32```).

The line ```&mut data as *mut T as *mut c_void``` is a 2-step type casting from a ```T``` **reference** to a raw ```c_void``` **pointer**.
The last parameter of [```AudioObjectGetPropertyData```][gpd] is a **void** pointer pointing the the address of ```data```.[```std::os::raw::c_void```][void] is used to construct a *C* *void* pointer and it is only useful as a pointer target. It's illegal to directly cast from a reference to a *void* pointer in *Rust*. To get the *void* pointer from ```T``` reference, we need:
1. Cast from a ```T``` reference(```&mut data```) to a raw ```T``` pointer(by ```&mut data as *mut T```)
2. Cast from a raw ```T``` pointer to a *c_void* pointer(by ```*mut T as *mut c_void```)

Finally, let's look how to link the *Rust* function ```fn AudioObjectGetPropertyData(...) -> OSStatus``` to native *C* API [```OSStatus AudioObjectGetPropertyData```][gpd].

The ```#[cfg(target_os = "macos")]``` is a [configuration option][cfg] that compiles code only when the operating system is *Mac OS*. The ```AudioObjectGetPropertyData``` is a platform-dependent API on *Mac OS*, so we use this attribute to mark it. Another common option is ```#[cfg(test)]```. That's what we used in [previous post for testing][prev]. It indicates that the following code is only compiled for the test harness.

The ```#[link(name = "CoreAudio", kind = "framework")]``` is a [*FFI attribute*][ffiattr]. The ```link``` attribute on ```extern``` blocks define how the functions in block link to the native libraries. The ```CoreAudio```, assigned to ```name```, is the name of the native library that the compiler will link to. The ```framework```, assigned to ```kind```, is the type of native library that the compiler will link to. The different ```kind``` values are meant to differentiate how the native library participates in linkage. Note that ```framework``` are only available on macOS targets. For more details about *linking*, read [here][linking].

The rest of code in the ```extern``` block is similar to what we did in [Calling Native C APIs from Rust][callingC].

We don't change the public interface in *src/lib.rs*, so we can leave *tests/integration.rs* and *src/main.rs* as they are.

*tests/integration.rs*:
```rust
extern crate rust_audio_lib; // Introduce the `rust_audio_lib` library crate.
use rust_audio_lib::utils; // Refer to `utils` module

#[test]
fn test_get_default_device_id() {
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

Everything is all set. It's time to run them!

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

And the ids of default input and output devices are successfully printed out!

However, there are lots of warnings when we run it. Most of them are naming issues, such as considering renaming `mSelector` to `m_selector`, or `kAudioObjectUnknown` to `K_AUDIO_OBJECT_UNKNOWN`. This is a conflict of preferred naming between *C* and *Rust*. I prefer to remain the *C* style naming of the corresponding type alias because I can recognize where they are from. To disable the warnings, we could add ```#![allow(non_snake_case, non_upper_case_globals)]``` at the beginning in *sys.rs*.

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
- [Raw Pointers][rawptr]
  - [The Rust Programming Language(first edition): Raw Pointers][refandptr]
  - [The Rust Programming Language: Unsafe Rust][unsafe]
- Rust APIs
  - [std::mem::uninitialized][memuninit]
  - [std::mem::size_of_val][memsize]
  - [std::ptr::null][nullptr]
  - [std::os::raw::c_void][void]

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

[rawptr]: https://doc.rust-lang.org/book/raw-pointers.html "Raw Pointers"
[refandptr]: https://doc.rust-lang.org/book/first-edition/raw-pointers.html#references-and-raw-pointers "The Rust Programming Language(first edition): Raw Pointers"
[unsafe]: https://doc.rust-lang.org/book/second-edition/ch19-01-unsafe-rust.html "The Rust Programming Language: Unsafe Rust"

[memuninit]: https://doc.rust-lang.org/std/mem/fn.uninitialized.html "std::mem::uninitialized"
[memsize]: https://doc.rust-lang.org/1.25.0/std/mem/fn.size_of_val.html "std::mem::size_of_val"
[nullptr]: https://doc.rust-lang.org/std/ptr/fn.null.html "std::ptr::null"
[void]: https://doc.rust-lang.org/std/os/raw/enum.c_void.html "std::os::raw::c_void"