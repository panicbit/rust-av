
# Requirements

- **LLVM >=3.9**
<!--   There are constants like `#define AV_NOPTS_VALUE ((int64_t)UINT64_C(0x8000000000000000))`
  which are not included in the generated bindings. As a workaround constants of the form
  `const int64_t RUST_AV_NOPTS_VALUE = AV_NOPTS_VALUE;` are added. These are not interpreted
  correctly using LLVM versions before 3.9 (tested with Ubuntu 16.04, LLVM 3.8.0).
  If [this issue](https://github.com/servo/rust-bindgen/issues/316) gets resolved,
  this requirement might vanish. -->
- **FFmpeg 3.x**
