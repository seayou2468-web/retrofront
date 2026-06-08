use super::{DriverFrame, GfxDriver, PresentStatus};
use crate::gfx::context::ContextDriver;
use crate::gfx::frame::VideoFrame;
use crate::gfx::hardware::{GfxBackendKind, HostRenderHandles};

pub const OPENGL_VERTEX_SHADER: &str = r#"#version 300 es
precision mediump float;
layout(location = 0) in vec2 a_position;
layout(location = 1) in vec2 a_texcoord;
out vec2 v_texcoord;
void main() {
    v_texcoord = a_texcoord;
    gl_Position = vec4(a_position, 0.0, 1.0);
}
"#;

pub const OPENGL_FRAGMENT_SHADER: &str = r#"#version 300 es
precision mediump float;
in vec2 v_texcoord;
uniform sampler2D u_frame;
out vec4 color;
void main() {
    color = texture(u_frame, v_texcoord);
}
"#;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlDrawCall {
    pub viewport: [i32; 4],
    pub texture_size: [u32; 2],
    pub framebuffer: usize,
    pub native_view: u64,
    pub gl_context: u64,
    pub vertex_shader: &'static str,
    pub fragment_shader: &'static str,
    pub uses_ios_sdk_context: bool,
    pub bottom_left_origin: bool,
}

#[derive(Debug, Clone, Default)]
pub struct OpenGlDriver {
    handles: HostRenderHandles,
    last_draw_call: Option<GlDrawCall>,
    initialized: bool,
}

impl OpenGlDriver {
    pub fn configure(&mut self, context: &ContextDriver) {
        self.handles = context.handles();
    }
    pub fn last_draw_call(&self) -> Option<&GlDrawCall> {
        self.last_draw_call.as_ref()
    }

    fn build_call(
        &self,
        width: u32,
        height: u32,
        bottom_left_origin: bool,
    ) -> Result<GlDrawCall, String> {
        if !self.handles.has_opengl() {
            return Err(
                "OpenGL backend requires native_view and iOS SDK GL context handles".into(),
            );
        }
        Ok(GlDrawCall {
            viewport: [0, 0, width as i32, height as i32],
            texture_size: [width, height],
            framebuffer: usize::MAX,
            native_view: self.handles.native_view,
            gl_context: self.handles.gl_context,
            vertex_shader: OPENGL_VERTEX_SHADER,
            fragment_shader: OPENGL_FRAGMENT_SHADER,
            uses_ios_sdk_context: cfg!(target_os = "ios") || self.handles.gl_context != 0,
            bottom_left_origin,
        })
    }
}

impl GfxDriver for OpenGlDriver {
    fn name(&self) -> &'static str {
        "opengl-es-ios-sdk"
    }
    fn init(&mut self, context: &ContextDriver, _bootstrap_frame: &VideoFrame) {
        self.configure(context);
        self.initialized = true;
    }
    fn context_reset(&mut self, context: &ContextDriver) {
        self.configure(context);
        self.last_draw_call = None;
    }
    fn present(&mut self, frame: &DriverFrame) -> Result<PresentStatus, String> {
        if !self.initialized {
            self.initialized = true;
        }
        let (width, height, frame_number, bottom_left_origin) = match frame {
            DriverFrame::Software(frame) => (frame.width, frame.height, 0, false),
            DriverFrame::Hardware(frame) => (
                frame.width,
                frame.height,
                frame.frame_number,
                frame.request.bottom_left_origin,
            ),
        };
        let call = self.build_call(width, height, bottom_left_origin)?;
        self.last_draw_call = Some(call);
        Ok(PresentStatus {
            backend: GfxBackendKind::OpenGl,
            frame_number,
            rendered: true,
            details: format!(
                "OpenGL ES draw call prepared for {width}x{height} using iOS SDK context"
            ),
        })
    }
    fn destroy(&mut self) {
        self.initialized = false;
        self.last_draw_call = None;
    }
}
