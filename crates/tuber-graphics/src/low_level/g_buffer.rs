pub struct GBuffer {
    pub albedo: wgpu::Texture,
    pub normal: wgpu::Texture,
    pub emission: wgpu::Texture,
    pub position: wgpu::Texture,
}
