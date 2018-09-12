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

    pub fn get_default_device_id(scope: &Scope) -> Result<AudioObjectID, OSStatus> {
        let id: AudioObjectID = kAudioObjectSystemObject;
        let address: &AudioObjectPropertyAddress = if scope == &Scope::Input {
            &DEFAULT_INPUT_DEVICE_PROPERTY_ADDRESS
        } else {
            &DEFAULT_OUTPUT_DEVICE_PROPERTY_ADDRESS
        };
        get_property_data::<AudioObjectID>(id, address)
    }

    #[cfg(test)] // Indicates this is only included when running `cargo test`
    mod tests {
        // A private internal submodule in utils
        use super::*; // To use the functions in utils

        #[test] // Built only within `cargo test`.
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
