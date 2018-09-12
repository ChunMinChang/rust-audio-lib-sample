#!/bin/sh

LIBRARY_PATH="target/debug/"
LIBRARY_NAME="rust_audio_lib"

C_SOURCE="src/get_default_device_id.c"
CPP_SOURCE="src/get_default_device_id.cpp"

C_EXE="get_default_device_id-c"
CPP_EXE="get_default_device_id-cpp"

run_and_clean()
{
  echo "Run c executable file:"
  ./$C_EXE

  echo "Run c++ executable file:"
  ./$CPP_EXE

  echo "Clean executable files."
  rm $C_EXE $CPP_EXE
}

compile_to_exe()
{
  compiler=$1
  source=$2
  exe=$3
  append="$4"
  $compiler $source -L $LIBRARY_PATH -l$LIBRARY_NAME $append -o $exe
}

echo "Build Rust library."
cargo build --lib

echo "Build executable files."
FRAMEWORK_DEPENDENCY="-framework CoreAudio"
compile_to_exe gcc $C_SOURCE $C_EXE "$FRAMEWORK_DEPENDENCY"
compile_to_exe g++ $CPP_SOURCE $CPP_EXE "$FRAMEWORK_DEPENDENCY"

echo "Run executable files."
run_and_clean

echo "Clean Rust library."
cargo clean