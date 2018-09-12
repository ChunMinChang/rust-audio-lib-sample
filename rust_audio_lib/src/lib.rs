mod sys; // Module contains types and functions of the external libraries

pub mod utils {
    use super::sys; // Bring `sys` module into scope
    use std::mem;   // For mem::uninitialized(), mem::size_of_val()
    use std::os::raw::c_void;
    use std::ptr; // For ptr::null()

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

    pub fn get_default_device_id(scope: &Scope) -> Result<DeviceId, Error> {
        let address: &sys::AudioObjectPropertyAddress = if scope == &Scope::Input {
            &DEFAULT_INPUT_DEVICE_PROPERTY_ADDRESS
        } else {
            &DEFAULT_OUTPUT_DEVICE_PROPERTY_ADDRESS
        };
        let id = get_property_data::<sys::AudioObjectID>(sys::kAudioObjectSystemObject, address)?;
        if id == sys::kAudioObjectUnknown {
            Err(Error::NoDevice)
        } else {
            Ok(to_device_id(id))
        }
    }

    fn to_device_id(id: sys::AudioObjectID) -> DeviceId {
        id as DeviceId
    }

    fn get_property_data<T>(
        id: sys::AudioObjectID,
        address: &sys::AudioObjectPropertyAddress,
    ) -> Result<T, Error> {
        assert!(id != sys::kAudioObjectUnknown, "Bad AudioObjectID!");
        // Use `mem::uninitialized()` to bypasses memory-initialization checks
        let mut data: T = unsafe { mem::uninitialized() };
        let mut size = mem::size_of_val(&data);
        let status = audio_object_get_property_data(id, address, &mut size, &mut data);
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
                size as *mut u32,    // Cast raw usize pointer to raw u32 pointer
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
    mod tests {
        // A private internal submodule in utils
        use super::*; // To use the functions in utils

        #[test] // Built only within `cargo test`.
        #[should_panic(expected = "Bad")]
        fn test_get_property_data_invalid_id() {
            // Invalid AudioObjectID with valid input adrress.
            assert_eq!(
                get_property_data::<AudioObjectID>(
                    kAudioObjectUnknown,
                    &DEFAULT_INPUT_DEVICE_PROPERTY_ADDRESS
                ).unwrap_err(),
                kAudioHardwareBadObjectError
            );
            // Invalid AudioObjectID with valid output adrress.
            assert_eq!(
                get_property_data::<AudioObjectID>(
                    kAudioObjectUnknown,
                    &DEFAULT_OUTPUT_DEVICE_PROPERTY_ADDRESS
                ).unwrap_err(),
                kAudioHardwareBadObjectError
            );
        }

        #[test] // Built only within `cargo test`.
        fn test_get_property_data() {
            // Check the default input device id is valid.
            assert_ne!(
                get_property_data::<AudioObjectID>(
                    kAudioObjectSystemObject,
                    &DEFAULT_INPUT_DEVICE_PROPERTY_ADDRESS
                ).unwrap(),
                kAudioObjectUnknown
            );
            // Check the default output device id is valid.
            assert_ne!(
                get_property_data::<AudioObjectID>(
                    kAudioObjectSystemObject,
                    &DEFAULT_OUTPUT_DEVICE_PROPERTY_ADDRESS
                ).unwrap(),
                kAudioObjectUnknown
            );
        }
    }
}

#[no_mangle] // Tell the Rust compiler not to mangle the name of this function.
pub extern "C" fn get_default_device_id(
    scope: utils::Scope,
    id: *mut utils::DeviceId,
) -> utils::Error {
    if id.is_null() {
        return utils::Error::InvalidParameters;
    }
    match utils::get_default_device_id(&scope) {
        Ok(device_id) => {
            unsafe { *id = device_id };
            utils::Error::Ok
        }
        Err(error) => error,
    }
}
