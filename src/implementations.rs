use glutin::{config::GlConfig, prelude::PossiblyCurrentGlContext};
use skia_safe::{ColorType, gpu::{backend_render_targets::make_gl, gl::{Format, FramebufferInfo}, surfaces::wrap_backend_render_target}};
use winit::dpi::PhysicalSize;

use crate::ecs::GpuState;

impl GpuState {
    pub fn create_skia_surface(&mut self, size: PhysicalSize<u32>) {
        let _ = self.gl_context.make_current(&self.gl_surface).unwrap();
        let fb_info = FramebufferInfo {
            fboid: 0,
            format: Format::RGBA8.into(),
            protected: skia_safe::gpu::Protected::No,
        };

        let _sample_count = self.gl_config.num_samples() as usize;
        let stencil_bits = self.gl_config.stencil_size() as usize;

        let backend_render_target = make_gl(
            (size.width as i32, size.height as i32),
            Some(0),
            stencil_bits,
            fb_info);

        unsafe {
            gl::Viewport(0, 0, size.width as i32, size.height as i32);
        };

        self.skia_surface = Some(wrap_backend_render_target(
                &mut self.gr_context,
                &backend_render_target,
                skia_safe::gpu::SurfaceOrigin::BottomLeft,
                ColorType::N32,
                None,
                None
        ).expect("failed to create skia surface"));
    }
}
