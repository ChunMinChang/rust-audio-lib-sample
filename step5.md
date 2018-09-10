# Library Based on Platform C API - Part 1: Basic Outline

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

The ```get_property_data``` is a mock API that should be implemented to call the native [```AudioObjectGetPropertyData```][gpd] API. [```AudioObjectGetPropertyData```][gpd] takes one [```AudioObjectID```][aoid] variable as its parameter and returns an [OSStatus][oss]. The returned [OSStatus][oss] value is used to check if the query is successful. If the query works, we can get a [```AudioObjectID```][aoid] value by a passed-in pointer parameter. Since the [```AudioObjectID```][aoid] type is ```u32``` and [OSStatus][oss] type is ```i32```, ```get_property_data``` takes a ```u32``` parameter and return a [```Result```][result] whose **ok** type is ```u32``` and **error** type is ```i32```. Returning a [```Result```][result] is a common error handling pattern in *Rust*, so we adopt this idiom.

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

It verifies the API works for both default input and output devices.

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



## References
- [AudioBehaviorCheck][abc]
- OSX/CoreAudio
  - [OSStatus][oss]
  - [AudioObjectID][aoid]
  - [AudioObjectGetPropertyData][gpd]
- [Result][result]

[abc]: https://github.com/ChunMinChang/AudioBehaviorCheck/blob/3da870e4dfa5bd9b3c47ca6e658552dabd8217e4/AudioObjectUtils.cpp#L65-L75 "ABC: AudioObjectUtils::GetDefaultDeviceId"

[aoid]: https://developer.apple.com/documentation/coreaudio/audioobjectid?language=objc "AudioObjectID"
[oss]: https://developer.apple.com/documentation/kernel/osstatus?language=objc "OSStatus"
[gpd]: https://developer.apple.com/documentation/coreaudio/1422524-audioobjectgetpropertydata?language=objc "AudioObjectGetPropertyData"

[result]: https://doc.rust-lang.org/book/second-edition/ch09-02-recoverable-errors-with-result.html "The Rust Programming Language: Recoverable Errors with Result"