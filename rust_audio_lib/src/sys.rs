#![allow(non_snake_case, non_upper_case_globals)]

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
#[cfg(test)]
pub const kAudioHardwareBadObjectError: OSStatus = 560947818;

// https://developer.apple.com/documentation/coreaudio/1494461-anonymous/kaudioobjectunknown
#[cfg(test)]
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

pub fn get_property_data<T>(
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
