//! Device Handle centered API

use crate::*;
use std::ops::Range;
use std::os::raw::c_void;

pub type DeviceHandle = *mut crate::Device;

unsafe fn device_call<F, R>(device: DeviceHandle, fnc: F) -> R
where
    F: FnOnce(&Device) -> R,
{
    let device = Box::from_raw(device);
    let result = fnc(&device);
    Box::leak(device);
    result
}

pub unsafe fn create_device<F>(loader: F, debug: Debug<DebugCallback>) -> DeviceHandle
where
    F: FnMut(&str) -> *const c_void,
{
    let device = Box::new(crate::Device::new(loader, debug));
    Box::into_raw(device)
}

pub unsafe fn destroy_device(device: DeviceHandle) {
    let _ = Box::from_raw(device);
}

pub unsafe fn create_shader(
    device: DeviceHandle,
    stage: ShaderStage,
    source: &[u8],
) -> Result<Shader> {
    device_call(device, |device| device.create_shader(stage, source))
}

pub unsafe fn create_graphics_pipeline<D>(device: DeviceHandle, desc: D) -> Result<Pipeline>
where
    D: Into<GraphicsPipelineDesc>,
{
    device_call(device, move |device| device.create_graphics_pipeline(desc))
}

pub unsafe fn create_vertex_array(
    device: DeviceHandle,
    attributes: &[VertexAttributeDesc],
) -> Result<VertexArray> {
    device_call(device, |device| device.create_vertex_array(attributes))
}

pub unsafe fn create_buffer_from_host(
    device: DeviceHandle,
    data: &[u8],
    memory: MemoryFlags,
) -> Result<Buffer> {
    device_call(device, |device| {
        device.create_buffer_from_host(data, memory)
    })
}

pub unsafe fn bind_pipeline(device: DeviceHandle, pipeline: Pipeline) {
    device_call(device, |device| device.bind_pipeline(pipeline))
}

pub unsafe fn bind_vertex_array(device: DeviceHandle, vao: VertexArray) {
    device_call(device, |device| device.bind_vertex_array(vao))
}

/// Bind vertex buffers to a vertex array.
pub unsafe fn bind_vertex_buffers(
    device: DeviceHandle,
    vao: VertexArray,
    first: u32,
    views: &[VertexBufferView],
) {
    device_call(device, |device| {
        device.bind_vertex_buffers(vao, first, views)
    })
}

pub unsafe fn set_viewport(device: DeviceHandle, first: u32, viewports: &[Viewport]) {
    device_call(device, |device| device.set_viewport(first, viewports))
}

pub unsafe fn set_scissor(device: DeviceHandle, first: u32, scissors: &[Region]) {
    device_call(device, |device| device.set_scissor(first, scissors))
}

pub unsafe fn clear_attachment(device: DeviceHandle, fb: Framebuffer, cv: ClearAttachment) {
    device_call(device, |device| device.clear_attachment(fb, cv))
}

pub unsafe fn draw(
    device: DeviceHandle,
    primitive: Primitive,
    vertices: Range<u32>,
    instance: Range<u32>,
) {
    device_call(device, |device| device.draw(primitive, vertices, instance))
}
