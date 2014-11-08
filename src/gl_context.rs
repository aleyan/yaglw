use gl;
use gl::types::*;
use shader::Shader;
use std::raw;
use std::mem;
use std::ptr;
use std::str;

unsafe fn from_c_str<'a>(s: *const u8) -> &'a str {
  let mut len = 0;
  {
    let mut s_shift = s;
    while *s_shift as char != '\0' {
      s_shift = s_shift.offset(1);
      len += 1;
    }
  }

  let as_slice: raw::Slice<u8> =
    raw::Slice {
      data: s,
      len: len,
    };

  str::raw::from_utf8(mem::transmute(as_slice))
}

/// A handle to an OpenGL context. Only create one of these per thread.
#[deriving(Send)]
pub struct GLContext;

impl GLContext {
  /// Create a new OpenGL context.
  pub fn new() -> GLContext {
    // TODO(cgaebel): Have a thread-local variable checking whether or not
    // there is only one GLContext, and fail if there's more than one.
    GLContext
  }

  /// Stops the processing of any triangles hidden from view when rendering.
  pub fn enable_culling(&self) {
    unsafe {
      gl::FrontFace(gl::CCW);
      gl::CullFace(gl::BACK);
      gl::Enable(gl::CULL_FACE);
    }
  }

  #[allow(missing_docs)]
  pub fn enable_alpha_blending(&self) {
    unsafe {
      gl::Enable(gl::BLEND);
      gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }
  }

  #[allow(missing_docs)]
  pub fn enable_smooth_lines(&self) {
    unsafe {
      gl::Enable(gl::LINE_SMOOTH);
      gl::LineWidth(2.5);
    }
  }

  /// Allows us to use the OpenGL depth buffer, which makes OpenGL do logical
  /// things when two things are rendered at the same x and y coordinates, but
  /// different z coordinates.
  pub fn enable_depth_buffer(&self, depth: GLclampd) {
    unsafe {
      gl::Enable(gl::DEPTH_TEST);
      gl::DepthFunc(gl::LESS);
      gl::ClearDepth(depth);
    }
  }

  /// At the beginning of each frame, OpenGL clears the buffer. This sets the
  /// color the buffer is cleared to.
  pub fn set_background_color(&self, r: GLfloat, g: GLfloat, b: GLfloat, a: GLfloat) {
    unsafe {
      gl::ClearColor(r, g, b, a);
    }
  }

  /// Replace the current OpenGL buffer with all pixels of the
  /// "background color", as set with `set_background_color`.
  pub fn clear_buffer(&self) {
    unsafe {
      gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
  }

  /// Compiles a shader for the current graphics card.
  pub fn compile_shader(&self, src: String, ty: GLenum) -> GLuint {
    unsafe {
      let shader = gl::CreateShader(ty);
      // Attempt to compile the shader
      src.with_c_str(|ptr| gl::ShaderSource(shader, 1, &ptr, ptr::null()));
      gl::CompileShader(shader);

      // Get the compile status
      let mut status = gl::FALSE as GLint;
      gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

      // Fail on error
      if status != (gl::TRUE as GLint) {
        let mut len = 0;
        gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
        let mut buf = Vec::from_elem(len as uint - 1, 0u8); // subtract 1 to skip the trailing null character
        gl::GetShaderInfoLog(shader, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
        panic!("error compiling 0x{:x} shader: {}", ty, str::from_utf8(buf.as_slice()).expect("ShaderInfoLog not valid utf8"));
      }
      shader
    }
  }

  fn get_current_shader(&self) -> GLuint {
    unsafe {
      let mut ret: GLint = -1;
      gl::GetIntegerv(gl::CURRENT_PROGRAM, &mut ret);
      assert!(ret >= 0, "Need positive shader. Got {}.", ret);
      ret as GLuint
    }
  }

  /// Apply a given shader while rendering the body of the closure.
  pub fn use_shader<T>(&self, shader: &Shader, f: |&GLContext| -> T) -> T {
    // TODO(cgaebel): I heard that OpenGL MIGHT be synchronized on any of the
    // `Get` functions, which means this will be unnecessarily slow. One day
    // we should think about maintaining the shader stack ourselves.
    let old_shader = self.get_current_shader();
    unsafe {
      gl::UseProgram(shader.id);
    }
    let r = f(self);
    if old_shader != 0 {
      unsafe {
        gl::UseProgram(old_shader);
      }
    }
    r
  }

  /// Prints opengl version information.
  pub fn print_stats(&self) {
    unsafe {
      let opengl_version = gl::GetString(gl::VERSION);
      let glsl_version = gl::GetString(gl::SHADING_LANGUAGE_VERSION);
      info!(
        "OpenGL version: {}", 
        from_c_str(opengl_version),
      );
      info!(
        "GLSL version: {}",
        from_c_str(glsl_version),
      );
    }
  }
}
