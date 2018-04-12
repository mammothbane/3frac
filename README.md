## Build instructions

### Windows

- Install MinGW-w64.

- Install MSYS 1.0 (for MSYS Makefile support&mdash;for whatever reason `cmake` needs this).

- Add MinGW and MSYS `bin`s to path.

- Download the headers `<GLES/*.h>`, `<GLES2/*.h>`, `<EGL/*.h>`, `<KHR/*.h>` from the OpenGL ES 
    [repository](https://www.khronos.org/registry/OpenGL/index.php#repository) and put them in
    `<MinGW root>/x86_64-w64-mingw32/include` in their appropriate subdirectories.
