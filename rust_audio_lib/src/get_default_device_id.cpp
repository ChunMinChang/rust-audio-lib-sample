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