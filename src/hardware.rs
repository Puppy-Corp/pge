use std::ops::Range;
use crate::buffer::Buffer;
use crate::buffer::BufferSlice;
use crate::ArenaId;
use crate::Window;

pub trait Hardware {
    fn create_buffer(&mut self, name: &str) -> BufferHandle;
    fn create_texture(&mut self, name: &str, data: &[u8]) -> TextureHandle;
    fn create_pipeline(&mut self, name: &str, window: WindowHandle) -> PipelineHandle;
    fn render(&mut self, encoder: RenderEncoder, window: WindowHandle);
    fn create_window(&mut self, window: &Window) -> WindowHandle;
    fn destroy_window(&mut self, handle: WindowHandle);
	fn write_buffer(&mut self, buffer: BufferHandle, data: &[u8]);
}

#[derive(Debug, Clone, Copy)]
pub struct WindowHandle {
    pub id: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct PipelineHandle {
    pub id: u32,
}

pub struct Surface {

}

#[derive(Debug, Clone, Copy)]
pub struct BufferHandle {
    pub id: u32,
}

#[derive(Debug)]
pub struct Pipeline {}

pub struct RenderEncoder {
    pub passes: Vec<RenderPass>
}

impl RenderEncoder {
    pub fn new() -> Self {
        Self {
            passes: Vec::new(),
        }
    }

    pub fn begin_render_pass(&mut self) -> &mut RenderPass {
        let render_pass = RenderPass::default();
        self.passes.push(render_pass);
        self.passes.last_mut().unwrap()
    }
}   

#[derive(Default)]
pub struct RenderPass {
    pub subpasses: Vec<Subpass>,
    pub vertex_buffers: Vec<(u32, BufferSlice)>,
    pub index_buffer: Option<BufferSlice>,
    pub pipeline: Option<PipelineHandle>,
    pub buffers: Vec<(u32, BufferHandle)>,
    pub textures: Vec<(u32, TextureHandle)>,
    pub indices: Option<Range<u32>>,
    pub instances: Option<Range<u32>>,
}

impl RenderPass {
    pub fn bind_buffer(&mut self, slot: u32, buffer: Buffer) {
        self.buffers.push((slot, buffer.handle));
    }

    pub fn bind_texture(&mut self, slot: u32, texture: TextureHandle) {
        self.textures.push((slot, texture));
    }

    pub fn set_vertex_buffer(&mut self, slot: u32, buffer: BufferSlice) {
        self.vertex_buffers.push((slot, buffer));
    }

    pub fn set_index_buffer(&mut self, buffer: BufferSlice) {
        self.index_buffer = Some(buffer);
    }

    pub fn draw_indexed(&mut self, indices: Range<u32>, instances: Range<u32>) {
        self.indices = Some(indices);
        self.instances = Some(instances);
        let subpass = Subpass {
            vertex_buffers: self.vertex_buffers.clone(),
            index_buffer: self.index_buffer.clone(),
            pipeline: self.pipeline.clone(),
            buffers: self.buffers.clone(),
            indices: self.indices.clone(),
            instances: self.instances.clone(),
        };
        self.subpasses.push(subpass);
    }

    pub fn set_pipeline(&mut self, pipeline: PipelineHandle) {
        self.pipeline = Some(pipeline);
    }
}

pub struct Subpass {
    pub vertex_buffers: Vec<(u32, BufferSlice)>,
    pub index_buffer: Option<BufferSlice>,
    pub pipeline: Option<PipelineHandle>,
    pub buffers: Vec<(u32, BufferHandle)>,
    pub indices: Option<Range<u32>>,
    pub instances: Option<Range<u32>>,
}

#[derive(Debug, Clone, Copy)]
pub struct TextureHandle {
    pub id: u32,
}