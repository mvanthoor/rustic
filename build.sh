#!/bin/bash

echo "Building Rustic. Please wait."

platform=$(uname -a | awk '{print tolower($0)}')

case $platform in
  "mingw64"*)
    echo "Windows 64-bit"
    ;;
  "mingw32"*)
    echo "Windows 32-bit"
    ;;
  "darwin"*)
    echo "MacOS"
    ;;
esac