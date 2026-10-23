//! GPU compositor: shows one frame or blends two with a Happy-Kitten
//! transition. Everything renders into an `Rgba8Unorm` texture that egui can
//! display directly and that export can read back.

use bytemuck::{Pod, Zeroable};
use eframe::wgpu;
use hk_core::TransitionKind;

pub use eframe::wgpu as wgpu_crate;

pub const OUTPUT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct TextUniforms {
    res: [f32; 2],
    progress: f32,
    time: f32,
    kind: u32,
    _pad: u32,
    anchor: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Uniforms {
    res: [f32; 2],
    progress: f32,
    time: f32,
    kind: u32,
    fit: u32,
    _pad: [u32; 2],
    a_xform: [f32; 4],
    b_xform: [f32; 4],
    a_cover: [f32; 4],
    b_cover: [f32; 4],
    /// Colour grade gains (rgb, w unused) of the two pictures.
    a_gain: [f32; 4],
    b_gain: [f32; 4],
}

/// A decoded picture living in GPU memory.
pub struct GpuFrame {
    view: wgpu::TextureView,
    size: (u32, u32),
    _texture: wgpu::Texture,
}

impl GpuFrame {
    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    /// Approximate GPU memory in bytes (for cache budgeting).
    pub fn bytes(&self) -> usize {
        self.size.0 as usize * self.size.1 as usize * 4
    }
}

/// A render target the compositor draws into.
pub struct Output {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    /// Copy of the finished frame that text passes read while writing into `texture`.
    scratch: wgpu::Texture,
    scratch_view: wgpu::TextureView,
    size: (u32, u32),
}

impl Output {
    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub fn size(&self) -> (u32, u32) {
        self.size
    }
}

pub struct Compositor {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    sampler: wgpu::Sampler,
    uniforms: wgpu::Buffer,
    text_pipeline: wgpu::RenderPipeline,
    text_uniforms: wgpu::Buffer,
    black: GpuFrame,
    fit: std::cell::Cell<u32>,
    gains: std::cell::Cell<([f32; 4], [f32; 4])>,
}

const NO_GAIN: [f32; 4] = [1.0, 1.0, 1.0, 0.0];

impl Compositor {
    pub fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("hk-transitions"),
            source: wgpu::ShaderSource::Wgsl(include_str!("transitions.wgsl").into()),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("hk-transitions"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: OUTPUT_FORMAT,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("hk-linear"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });
        let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("hk-uniforms"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let text_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("hk-text-overlay"),
            source: wgpu::ShaderSource::Wgsl(include_str!("text_overlay.wgsl").into()),
        });
        let text_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("hk-text-overlay"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &text_shader,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &text_shader,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: OUTPUT_FORMAT,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let text_uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("hk-text-uniforms"),
            size: std::mem::size_of::<TextUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let black = upload_rgba(&device, &queue, &[0, 0, 0, 255], 1, 1);
        Compositor {
            device,
            queue,
            pipeline,
            sampler,
            uniforms,
            text_pipeline,
            text_uniforms,
            black,
            fit: std::cell::Cell::new(0),
            gains: std::cell::Cell::new((NO_GAIN, NO_GAIN)),
        }
    }

    /// Colour grade of the picture that is about to be drawn: `a` for the single or outgoing
    /// picture, `b` for the incoming one. Call before every render; it stays until changed.
    pub fn set_grades(&self, a: hk_core::Grade, b: hk_core::Grade) {
        let g = |x: hk_core::Grade| {
            let [r, gr, bl] = x.gains();
            [r, gr, bl, 0.0]
        };
        self.gains.set((g(a), g(b)));
    }

    /// How pictures of another shape are placed on the canvas (see [`hk_core::FitMode`]).
    pub fn set_fit(&self, fit: hk_core::FitMode) {
        self.fit.set(fit.shader_id());
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn create_output(&self, width: u32, height: u32) -> Output {
        let (width, height) = (width.clamp(1, 8192), height.clamp(1, 8192));
        let make = |label: &'static str, usage: wgpu::TextureUsages| {
            self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: OUTPUT_FORMAT,
                usage,
                view_formats: &[],
            })
        };
        let texture = make(
            "hk-output",
            wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
        );
        let scratch = make(
            "hk-scratch",
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        );
        let view = texture.create_view(&Default::default());
        let scratch_view = scratch.create_view(&Default::default());
        Output {
            texture,
            view,
            scratch,
            scratch_view,
            size: (width, height),
        }
    }

    /// Upload tightly packed RGBA8 pixels.
    pub fn upload(&self, rgba: &[u8], width: u32, height: u32) -> GpuFrame {
        upload_rgba(&self.device, &self.queue, rgba, width, height)
    }

    pub fn render_black(&self, out: &Output) {
        self.draw(out, &self.black, &self.black, 0, 0.0, 0.0);
    }

    pub fn render_single(&self, out: &Output, frame: &GpuFrame) {
        self.draw(out, frame, frame, 0, 0.0, 0.0);
    }

    /// `progress` 0 shows `from`, 1 shows `to`. `time` (seconds) drives the
    /// glitch flicker so scrubbing is deterministic.
    pub fn render_blend(
        &self,
        out: &Output,
        from: &GpuFrame,
        to: &GpuFrame,
        kind: TransitionKind,
        progress: f32,
        time: f32,
    ) {
        self.draw(out, from, to, kind.shader_id(), progress, time);
    }

    fn draw(&self, out: &Output, a: &GpuFrame, b: &GpuFrame, kind: u32, progress: f32, time: f32) {
        let u = Uniforms {
            res: [out.size.0 as f32, out.size.1 as f32],
            progress: if progress.is_finite() { progress } else { 0.0 },
            time: if time.is_finite() { time } else { 0.0 },
            kind,
            fit: self.fit.get(),
            _pad: [0; 2],
            a_xform: contain(a.size, out.size),
            b_xform: contain(b.size, out.size),
            a_cover: cover(a.size, out.size),
            b_cover: cover(b.size, out.size),
            a_gain: self.gains.get().0,
            b_gain: self.gains.get().1,
        };
        self.queue
            .write_buffer(&self.uniforms, 0, bytemuck::bytes_of(&u));
        let bind = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("hk-bind"),
            layout: &self.pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&a.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&b.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.uniforms.as_entire_binding(),
                },
            ],
        });
        let mut enc = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("hk-render"),
            });
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("hk-transition"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &out.view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind, &[]);
            pass.draw(0..3, 0..1);
        }
        self.queue.submit([enc.finish()]);
    }

    /// Draw `text` (frame-sized, premultiplied) over what is already in `out`. `progress` runs from
    /// 0 (hidden) to 1 (shown) and `fx` is a [`hk_core::TextFx`] shader id.
    pub fn overlay_text(
        &self,
        out: &Output,
        text: &GpuFrame,
        fx: u32,
        progress: f32,
        time: f32,
        anchor: [f32; 2],
    ) {
        let u = TextUniforms {
            res: [out.size.0 as f32, out.size.1 as f32],
            progress: if progress.is_finite() { progress } else { 1.0 },
            time: if time.is_finite() { time } else { 0.0 },
            kind: fx,
            _pad: 0,
            anchor,
        };
        self.queue
            .write_buffer(&self.text_uniforms, 0, bytemuck::bytes_of(&u));
        let bind = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("hk-text-bind"),
            layout: &self.text_pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&out.scratch_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&text.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.text_uniforms.as_entire_binding(),
                },
            ],
        });
        let mut enc = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("hk-text"),
            });
        enc.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &out.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: &out.scratch,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: out.size.0,
                height: out.size.1,
                depth_or_array_layers: 1,
            },
        );
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("hk-text-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &out.view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.text_pipeline);
            pass.set_bind_group(0, &bind, &[]);
            pass.draw(0..3, 0..1);
        }
        self.queue.submit([enc.finish()]);
    }

    /// Copy an output back to the CPU as tightly packed RGBA8 (export, tests).
    pub fn read_output(&self, out: &Output) -> Result<Vec<u8>, String> {
        let (w, h) = out.size;
        let row = (w * 4).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
        let buf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("hk-readback"),
            size: row as u64 * h as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut enc = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("hk-readback"),
            });
        enc.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &out.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buf,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(row),
                    rows_per_image: Some(h),
                },
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit([enc.finish()]);
        let slice = buf.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|e| e.to_string())?;
        rx.recv()
            .map_err(|e| e.to_string())?
            .map_err(|e| e.to_string())?;
        let data = slice.get_mapped_range().map_err(|e| e.to_string())?;
        let mut pixels = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h as usize {
            let start = y * row as usize;
            pixels.extend_from_slice(&data[start..start + (w * 4) as usize]);
        }
        drop(data);
        buf.unmap();
        Ok(pixels)
    }
}

/// uv scale/offset that fills `dst` with a centred crop of a `src`-sized picture.
fn cover(src: (u32, u32), dst: (u32, u32)) -> [f32; 4] {
    let ai = src.0.max(1) as f32 / src.1.max(1) as f32;
    let ao = dst.0.max(1) as f32 / dst.1.max(1) as f32;
    if ai > ao {
        let k = ao / ai;
        [k, 1.0, 0.5 - 0.5 * k, 0.0]
    } else {
        let k = ai / ao;
        [1.0, k, 0.0, 0.5 - 0.5 * k]
    }
}

/// uv scale/offset that letterboxes a `src`-sized picture inside `dst`.
fn contain(src: (u32, u32), dst: (u32, u32)) -> [f32; 4] {
    let ai = src.0.max(1) as f32 / src.1.max(1) as f32;
    let ao = dst.0.max(1) as f32 / dst.1.max(1) as f32;
    if ai > ao {
        let k = ai / ao;
        [1.0, k, 0.0, 0.5 - 0.5 * k]
    } else {
        let k = ao / ai;
        [k, 1.0, 0.5 - 0.5 * k, 0.0]
    }
}

fn upload_rgba(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    rgba: &[u8],
    width: u32,
    height: u32,
) -> GpuFrame {
    let (width, height) = (width.max(1), height.max(1));
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("hk-frame"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let expected = width as usize * height as usize * 4;
    // Short buffers (corrupt decode) leave the texture black instead of panicking in wgpu.
    if let Some(bytes) = rgba.get(..expected) {
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
    }
    let view = texture.create_view(&Default::default());
    GpuFrame {
        view,
        size: (width, height),
        _texture: texture,
    }
}

/// Create a device without a window (tests, contact sheets, future export).
pub fn headless() -> Result<(wgpu::Device, wgpu::Queue), String> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::PRIMARY,
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
        ..Default::default()
    }))
    .map_err(|e| e.to_string())?;
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cover_crops_instead_of_bars() {
        // a 16:9 picture on a 9:16 canvas shows only the middle strip
        let c = cover((1920, 1080), (1080, 1920));
        assert!(c[0] < 0.4 && c[1] == 1.0 && (c[2] - (0.5 - 0.5 * c[0])).abs() < 1e-6);
        assert_eq!(cover((1280, 720), (1280, 720)), [1.0, 1.0, 0.0, 0.0]);
    }

    #[test]
    fn contain_letterboxes_wide_and_tall_sources() {
        let wide = contain((2000, 1000), (1000, 1000));
        assert!(wide[1] > 1.0 && wide[0] == 1.0);
        let tall = contain((1000, 2000), (1000, 1000));
        assert!(tall[0] > 1.0 && tall[1] == 1.0);
        assert_eq!(contain((1280, 720), (1280, 720)), [1.0, 1.0, 0.0, 0.0]);
    }

    /// Renders a grey picture through the real shader; skipped on machines without a GPU.
    #[test]
    fn warm_grade_makes_grey_orange_and_neutral_changes_nothing() {
        let Ok((device, queue)) = headless() else {
            return;
        };
        let comp = Compositor::new(device, queue);
        let out = comp.create_output(8, 8);
        let grey = comp.upload(&[128u8, 128, 128, 255].repeat(16), 4, 4);
        let pixel = |comp: &Compositor| {
            let px = comp.read_output(&out).expect("read back");
            [px[0] as i32, px[1] as i32, px[2] as i32]
        };
        comp.render_single(&out, &grey);
        let plain = pixel(&comp);
        assert!((plain[0] - 128).abs() <= 1 && (plain[2] - 128).abs() <= 1);
        comp.set_grades(
            hk_core::Grade {
                temperature: 1.0,
                tint: 0.0,
            },
            hk_core::Grade::default(),
        );
        comp.render_single(&out, &grey);
        let warm = pixel(&comp);
        assert!(
            warm[0] > plain[0] + 10 && warm[2] < plain[2] - 10,
            "{warm:?} vs {plain:?}"
        );
        comp.set_grades(hk_core::Grade::default(), hk_core::Grade::default());
        comp.render_single(&out, &grey);
        assert_eq!(pixel(&comp), plain);
    }
}
