# Library Based on Platform C API - Part 3: Refactoring

There are few points I'd like to improve:
1. The type alias from native library in ```sys``` module may be reused by other modules, but ```get_property_data``` may be used only in ```utils``` module. It makes sense to me to move ```get_property_data``` from ```sys``` module to ```utils``` module.
2. We should use a custom error type instead of using ```OSStatus```. The ```OSStatus``` is from native system and we should not expect library user understand what it means.
3. One error is **hiding** behind success state. The returned ```AudioObjectID``` in success state may be a ```kAudioObjectUnknown```. That's why we check ```assert_ne!(get_property_data::<AudioObjectID>(...).unwrap(), kAudioObjectUnknown)``` in ```test_get_property_data()```. This is an **error**! We should throw an error instead of returning a value.
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
        fn test_get_property_data_invalid_id() {
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
        fn test_get_property_data() {
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

We use [```?``` operator to propagate errors][qop] here. If the ```Result``` value returned from the function before ```?``` operator is an ```Err```, then the ```Err``` will be thrown out immediately as what early return works. If the ```Result``` is an ```Ok```, then the value inside the ```Ok``` will be returned from the function before ```?``` operator, and the program will keep going.

To convert the ```OSStatus``` to our custom ```Error```, we implement [```std::convert::From```][conv-from] trait so we can get an ```Error``` by calling ```e.into()``` where ```e```'s type is ```OSStatus```.

On the other hand, we use automatic casting from reference to raw pointer (see [here][refandptr] for more details) to shorten our code:
- ```address``` in ```audio_object_get_property_data<T>```: ```&sys::AudioObjectPropertyAddress``` to ```*const sys::AudioObjectPropertyAddress```
- ```&mut size``` in ```get_property_data<T>```: ```&mut usize``` to ``` *mut usize```
- ```&mut data``` in ```get_property_data<T>```: ```&mut T``` to ```*mut T```

To know more about raw pointer, please read the related chapters in [first edition][refandptr] and [second edition][unsafe] of *The Rust Programming Language* listed on [Raw Pointers][rawptr]. The details about casting between types can be found [here][cast].

After refactoring, we need to use ```{:?}``` to show ```utils::Error``` value in *src/main.rs*:

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

```{:?}``` is used to show the debug message of the variables that implement ```std::fmt::Debug``` trait. Since we apply ```#[derive(Debug)]``` for the ```enum Error```, we can use that directly. See [Debug][debug] in *Rust By Example* for more detail.
If we want to use the normal display ```{}``` for ```utils::Error```, then we need to implement ```std::fmt::Display``` trait by our own.

Finally, we can run ```cargo test``` and ```cargo run``` to check if it works. The result will be same as what it shows above.

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
fn test_get_property_data_invalid_id() {
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
- [```?``` operator][qop]
- [Debug][debug]
- [Raw Pointers][rawptr]
  - [The Rust Programming Language(first edition): Raw Pointers][refandptr]
  - [The Rust Programming Language: Unsafe Rust][unsafe]
- [Casting between types][cast]
- [Checking for Panics with ```should_panic```][testpanic]
- [```std::convert::From```][conv-from] trait

[qop]: https://doc.rust-lang.org/book/second-edition/ch09-02-recoverable-errors-with-result.html#a-shortcut-for-propagating-errors-the--operator "? operator"

[debug]: https://doc.rust-lang.org/rust-by-example/hello/print/print_debug.html "Debug"

[rawptr]: https://doc.rust-lang.org/book/raw-pointers.html "Raw Pointers"
[refandptr]: https://doc.rust-lang.org/book/first-edition/raw-pointers.html#references-and-raw-pointers "The Rust Programming Language(first edition): Raw Pointers"
[unsafe]: https://doc.rust-lang.org/book/second-edition/ch19-01-unsafe-rust.html "The Rust Programming Language: Unsafe Rust"

[cast]: https://doc.rust-lang.org/book/casting-between-types.html "Casting between types"

[testpanic]: https://doc.rust-lang.org/book/second-edition/ch11-01-writing-tests.html#checking-for-panics-with-should_panic "Checking for Panics with should_panic"

[conv-from]: https://doc.rust-lang.org/std/convert/trait.From.html "Trait std::convert::From"

[r4cppp]: https://github.com/nrc/r4cppp/blob/master/control%20flow.md#switchmatch "Rust for C++ programmers"