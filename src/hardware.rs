use std::ops::Range;
use crate::ArenaId;
use crate::Texture;
use crate::Window;

pub trait Hardware {
    fn create_buffer(&mut self, name: &str) -> Buffer;
    fn create_texture(&mut self, name: &str, data: &[u8]) -> TextureHandle;
    fn create_pipeline(&mut self, name: &str, window_id: ArenaId<Window>) -> PipelineHandle;
    fn render(&mut self, encoder: RenderEncoder, surface: ArenaId<Window>);
    fn create_window(&mut self, window_id: ArenaId<Window>, window: &Window) -> ArenaId<Window>;
    fn destroy_window(&mut self, window_id: ArenaId<Window>);
}

pub struct PipelineHandle {
    pub id: u32,
}

pub struct Surface {

}

#[derive(Debug, Clone)]
pub struct Buffer {
    id: u32,
    data: Vec<u8>,
    offset: u64,
}

impl Buffer {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            data: Vec::new(),
            offset: 0,
        }
    }

    pub fn slice(&self, range: Range<u64>) -> BufferSlice {
        BufferSlice {
            buffer_id: self.id,
            range,
        }
    }

    pub fn full(&self) -> BufferSlice {
        BufferSlice {
            buffer_id: self.id,
            range: 0..self.data.len() as u64,
        }
    }
    
    pub fn begin(&mut self) {
        self.offset = 0;
    }

    pub fn write(&mut self, data: &[u8]) {
        let end = self.offset + data.len() as u64;
        if end > self.data.len() as u64 {
            self.data.resize(end as usize, 0);
        }
        self.data[self.offset as usize..end as usize].copy_from_slice(data);
        self.offset = end;
    }

    pub fn len(&self) -> u64 {
        self.data.len() as u64
    }

    pub fn capacity(&self) -> u64 {
        self.data.capacity() as u64
    }
}

#[derive(Debug, Clone)]
pub struct BufferSlice {
    buffer_id: u32,
    range: Range<u64>,
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
    pub pipeline: Option<ArenaId<Pipeline>>,
    pub buffers: Vec<(u32, Buffer)>,
    pub textures: Vec<(u32, ArenaId<Texture>)>,
    pub indices: Option<Range<u32>>,
    pub instances: Option<Range<u32>>,
}

impl RenderPass {
    pub fn bind_buffer(&mut self, slot: u32, buffer: Buffer) {
        self.buffers.push((slot, buffer));
    }

    pub fn bind_texture(&mut self, slot: u32, texture: ArenaId<Texture>) {
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

    pub fn set_pipeline(&mut self, pipeline: ArenaId<Pipeline>) {
        self.pipeline = Some(pipeline);
    }
}

pub struct Subpass {
    pub vertex_buffers: Vec<(u32, BufferSlice)>,
    pub index_buffer: Option<BufferSlice>,
    pub pipeline: Option<ArenaId<Pipeline>>,
    pub buffers: Vec<(u32, Buffer)>,
    pub indices: Option<Range<u32>>,
    pub instances: Option<Range<u32>>,
}

pub struct TextureHandle {
    pub id: u32,
}