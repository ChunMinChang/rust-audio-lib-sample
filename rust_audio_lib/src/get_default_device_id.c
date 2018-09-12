#include <stdbool.h>  // for bool
#include <stdio.h>    // for printf
#include "rust_audio_lib.h"

void show_result(Scope scope) {
  device_id id;
  const char* side = scope == Input ? "input" : "output";
  Error error = get_default_device_id(scope, &id);
  if (valid(error)) {
    printf("default %s device id: %d\n", side, id);
  } else {
    printf("Error on getting %s device: %s\n", side, error_message(error));
  }
}

int main() {
  show_result(Input);
  show_result(Output);
  return 0;
}