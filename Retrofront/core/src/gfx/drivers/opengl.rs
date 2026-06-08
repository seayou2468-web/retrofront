use super::{DriverFrame, GfxDriver, PresentStatus};
use crate::gfx::config::GfxVideoConfig;
use crate::gfx::context::ContextDriver;
use crate::gfx::frame::VideoFrame;
use crate::gfx::hardware::{GfxBackendKind, HostRenderHandles, OpenGlRenderCommand};
use crate::gfx::CLEAR_COLOR_RGBA;
use std::ffi::CStr;
use std::ptr;

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

static OPENGL_VERTEX_SHADER_CSTR: &CStr = c"#version 300 es\nprecision mediump float;\nlayout(location = 0) in vec2 a_position;\nlayout(location = 1) in vec2 a_texcoord;\nout vec2 v_texcoord;\nvoid main() {\n    v_texcoord = a_texcoord;\n    gl_Position = vec4(a_position, 0.0, 1.0);\n}\n";
static OPENGL_FRAGMENT_SHADER_CSTR: &CStr = c"#version 300 es\nprecision mediump float;\nin vec2 v_texcoord;\nuniform sampler2D u_frame;\nout vec4 color;\nvoid main() {\n    color = texture(u_frame, v_texcoord);\n}\n";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlDrawCall {
    pub viewport: [i32; 4],
    pub output_size: [u32; 2],
    pub texture_size: [u32; 2],
    pub framebuffer: usize,
    pub native_view: u64,
    pub gl_context: u64,
    pub vertex_shader: &'static str,
    pub fragment_shader: &'static str,
    pub uses_ios_sdk_context: bool,
    pub bottom_left_origin: bool,
    pub source_is_hardware: bool,
    pub rotation_quarters: u32,
}

#[derive(Debug, Clone, Default)]
pub struct OpenGlDriver {
    handles: HostRenderHandles,
    last_draw_call: Option<GlDrawCall>,
    video_config: GfxVideoConfig,
    initialized: bool,
}

impl OpenGlDriver {
    pub fn configure(&mut self, context: &ContextDriver) {
        self.handles = context.handles();
    }
    pub fn last_draw_call(&self) -> Option<&GlDrawCall> {
        self.last_draw_call.as_ref()
    }

    pub fn set_video_config(&mut self, config: GfxVideoConfig) {
        self.video_config = config;
    }

    fn build_call(
        &self,
        width: u32,
        height: u32,
        bottom_left_origin: bool,
        source_is_hardware: bool,
    ) -> Result<GlDrawCall, String> {
        if !self.handles.has_opengl() {
            return Err(
                "OpenGL backend requires native_view, GL context, and render callback handles"
                    .into(),
            );
        }
        Ok(GlDrawCall {
            viewport: self.video_config.viewport(width, height),
            output_size: self.video_config.output_size(width, height),
            texture_size: [width, height],
            framebuffer: self.handles.gl_framebuffer,
            native_view: self.handles.native_view,
            gl_context: self.handles.gl_context,
            vertex_shader: OPENGL_VERTEX_SHADER,
            fragment_shader: OPENGL_FRAGMENT_SHADER,
            uses_ios_sdk_context: cfg!(target_os = "ios") || self.handles.gl_context != 0,
            bottom_left_origin,
            source_is_hardware,
            rotation_quarters: self.video_config.rotation_quarters % 4,
        })
    }

    fn execute(&self, call: &GlDrawCall, rgba: Option<&[u8]>) -> Result<(), String> {
        let callback = self
            .handles
            .opengl_render
            .ok_or_else(|| "OpenGL render callback was not configured".to_string())?;
        let command = OpenGlRenderCommand {
            native_view: call.native_view,
            gl_context: call.gl_context,
            framebuffer: call.framebuffer,
            viewport: call.viewport,
            output_size: call.output_size,
            texture_size: call.texture_size,
            source_is_hardware: call.source_is_hardware,
            bottom_left_origin: call.bottom_left_origin,
            rotation_quarters: call.rotation_quarters,
            scale_mode: self.video_config.scale_mode as u32,
            filter_mode: self.video_config.filter_mode as u32,
            vsync: self.video_config.vsync,
            clear_color: CLEAR_COLOR_RGBA,
            vertex_shader: OPENGL_VERTEX_SHADER_CSTR.as_ptr(),
            fragment_shader: OPENGL_FRAGMENT_SHADER_CSTR.as_ptr(),
        };
        let (ptr, len) = rgba.map_or((ptr::null(), 0), |bytes| (bytes.as_ptr(), bytes.len()));
        let rendered = unsafe { callback(&command, ptr, len, self.handles.user_data) };
        if rendered {
            Ok(())
        } else {
            Err("OpenGL render callback reported failure".into())
        }
    }
}

impl GfxDriver for OpenGlDriver {
    fn name(&self) -> &'static str {
        "opengl-es-host"
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
        let (width, height, frame_number, bottom_left_origin, source_is_hardware, rgba) =
            match frame {
                DriverFrame::Software(frame) => (
                    frame.width,
                    frame.height,
                    0,
                    false,
                    false,
                    Some(frame.rgba.as_slice()),
                ),
                DriverFrame::Hardware(frame) => (
                    frame.width,
                    frame.height,
                    frame.frame_number,
                    frame.request.bottom_left_origin,
                    true,
                    None,
                ),
            };
        let call = self.build_call(width, height, bottom_left_origin, source_is_hardware)?;
        self.execute(&call, rgba)?;
        self.last_draw_call = Some(call);
        Ok(PresentStatus {
            backend: GfxBackendKind::OpenGl,
            frame_number,
            rendered: true,
            details: format!("OpenGL ES rendered {width}x{height} through host callback"),
        })
    }
    fn destroy(&mut self) {
        self.initialized = false;
        self.last_draw_call = None;
    }
}
