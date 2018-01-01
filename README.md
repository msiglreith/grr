<img align="left" src="info/grr_logo.png" width="196"/>

# grr!
#### A bare metal library for OpenGL 4.5+
[![crates.io](https://img.shields.io/crates/v/grr.svg)](https://crates.io/crates/grr)
[![docs](https://docs.rs/grr/badge.svg)](https://docs.rs/grr)
[![license](https://img.shields.io/badge/license-MPL%202.0-brightgreen.svg.svg)](LICENSE)
<hr>

`grr` aims at providing a thin layer above OpenGL 4.5+, exposing a modern API orientated on Vulkan.
The sole purpose of the library is to have a cleaned up API for **fast prototyping**. Neither state caching, extensive error checking or further assistance is implemented nor in-scope of the lib!

## Features
- API is built around **direct state access**
- Following Vulkan terminology and function names
- Only provided latest GL functionality
- Enforce sRGB framebuffer handling

## Getting started
Checkout our [Hello Triangle](https://github.com/msiglreith/grr/blob/master/examples/triangle.rs) example to see the library in action!

## Example
```rust
// Bind graphics pipeline (shaders)
grr.bind_pipeline(&pipeline);
// Configure vertex attributes
grr.bind_vertex_array(&vertex_array);
// Bind vertex buffers fer fetching attribute data
grr.bind_vertex_buffers(
    &vertex_array,
    0,
    &[grr::VertexBufferView {
        buffer: &triangle_data,
        offset: 0,
        stride: (std::mem::size_of::<f32>() * 5) as _,
    }]
);

// Clear default framebuffer
grr.clear_attachment(grr::Framebuffer::DEFAULT, grr::ClearAttachment::ColorFloat(0, [0.5, 0.5, 0.5, 1.0]));
// Draw triangles
grr.draw(grr::Primitive::Triangles, 0..3, 0..1);

// Present on screen!
window.swap_buffers().unwrap();
```
