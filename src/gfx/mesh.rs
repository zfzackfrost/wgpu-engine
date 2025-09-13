use crate::gfx::{IndexBuffer, IndexType, Vertex, VertexBuffer};

use std::ops::Range;

pub struct Mesh<V: Vertex, I: IndexType = u32> {
    vertices: VertexBuffer<V>,
    indices: Option<IndexBuffer<I>>,
}
impl<V: Vertex, I: IndexType> Mesh<V, I> {
    #[inline]
    pub fn new(vertices: VertexBuffer<V>, indices: Option<IndexBuffer<I>>) -> Self {
        Self { vertices, indices }
    }
    #[inline]
    pub fn count(&self) -> u32 {
        if let Some(indices) = self.indices.as_ref() {
            indices.count()
        } else {
            self.vertices.count()
        }
    }
    #[inline]
    pub fn bind(&self, rpass: &mut wgpu::RenderPass<'_>) {
        if let Some(indices) = self.indices.as_ref() {
            rpass.set_index_buffer(indices.slice(..), indices.index_format());
        }
        rpass.set_vertex_buffer(0, self.vertices.slice(..));
    }
    #[inline]
    pub fn draw(&self, instances: Range<u32>, rpass: &mut wgpu::RenderPass<'_>) {
        if self.indices.is_some() {
            rpass.draw_indexed(0..self.count(), 0, instances);
        } else {
            rpass.draw(0..self.count(), instances);
        }
    }
}
