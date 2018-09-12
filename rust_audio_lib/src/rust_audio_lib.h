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