# 0.7 (Feral Cat)

### Added

- Implement `std::error::Error` for `grr::Error`
- Support for Mesh/Task shaders (NV)
- Unbinding for indirect buffers

### Changed
- Resources are now `Copy`-able, removes references in all functions
- Split `GraphicsPipelineDesc` into `VertexPipelineDesc` and `MeshPipelineDesc`

# 0.6 (Puma)

### Fixed
- Indexed draw: index pointer was independent of the index ty

### Changed
- Add some missing `pub`s for structs

# 0.5 (Glyph Cat)

### Changed
- Adjust Host->Image copy API to specify passed host data

# 0.4 (Oncilla)

### Added
- Add more delete functions
- Framebuffer invalidation
- Improve error handling
- Debug utilities (messages, markers and labels)
- Buffer copies
- Indirect execution commands
- Multisampling support
- Device limits

# 0.3 (Bay Cat)

### Added
- Graphics Pipeline States:
    - Input Assembly
    - Color Blend
    - Depth Stencil
    - rasterization
- Seamless Cubemap Filtering
- Automatic Mipmap generation
- Uniform Constants
- Uniform Buffers
- 2D Array/Cube/3D image creation
- Bind framebuffers
- Set framebuffer attachments
- Added a few more image formats

### Fixed
- Clearing of depth/stencil attachments

# 0.2 (Caracal)

### Added
- Add `set_viewport` and `set_scissor` commands.
- Add support for samplers (`create_sampler`, `bind_sampler` and `delete_sampler`).

### Changed
- Enforce zero-to-one depth range for clip volume.
- Moved vertex divisors to vertex buffer binding.

### Fixed
- Fix vertex attribute locations.


# 0.1 (Initial release)
