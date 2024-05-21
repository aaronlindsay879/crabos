| start address        | offset   | end address          | size    | description             |
| -------------------- | -------- | -------------------- | ------- | ----------------------- |
| `0x0000000000000000` | 0        | `0x00007fffffffffff` | 128 TiB | user-space memory       |
| `0xffff800000000000` | -128 TiB | `0xffffbfffffffffff` | 64 TiB  | physical memory mapping |
| `0xffffffff00000000` | -4 GiB   | `0xffffffff1fffffff` | 512 MiB | bitmap frame allocator  |
| `0xffffffff20000000` | -3.5 GiB | `0xffffffff3fffffff` | 512 MiB | kernel heap             |
| `0xffffffff40000000` | -3 GiB   | `0xffffffff7fffffff` | 1 GiB   | framebuffer             |
| `0xffffffff80000000` | -2 GiB   | `0xffffffffffffffff` | 2 GiB   | kernel mappings         |