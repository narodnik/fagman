use miniquad::*;

mod shader;

#[repr(C)]
struct Vertex {
    pos: [f32; 2],
    color: [f32; 4],
    uv: [f32; 2],
}

struct Stage {
    ctx: Box<dyn RenderingBackend>,
    pipeline: Pipeline,
    //bindings: Bindings,
    white_texture: TextureId,
    text_bitmap: Vec<u8>,
    king_bitmap: Vec<u8>,
    king_texture: TextureId,
    last_char: char,
    font: fontdue::Font,
    show_king: bool,
    king_dim: (u16, u16),
}

impl Stage {
    pub fn new() -> Stage {
        let mut ctx: Box<dyn RenderingBackend> = window::new_rendering_backend();

        /*
        let vertices: [Vertex; 4] = [
            Vertex {
                pos: [-0.5, 0.5],
                color: [1., 1., 1., 1.],
                uv: [0., 0.],
            },
            Vertex {
                pos: [0.5, 0.5],
                color: [1., 1., 1., 1.],
                uv: [1., 0.],
            },
            Vertex {
                pos: [-0.5, -0.5],
                color: [1., 1., 1., 1.],
                uv: [0., 1.],
            },
            Vertex {
                pos: [0.5, -0.5],
                color: [1., 1., 1., 1.],
                uv: [1., 1.],
            },
        ];
        let vertex_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&vertices),
        );

        let indices: [u16; 6] = [0, 1, 2, 1, 2, 3];
        let index_buffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&indices),
        );
        */

        /*
        let pixels: [u8; 4 * 4 * 4] = [
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00,
            0x00, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        ];
        let texture = ctx.new_texture_from_rgba8(4, 4, &pixels);
        */

        let white_texture = ctx.new_texture_from_rgba8(1, 1, &[255, 255, 255, 255]);

        let font = include_bytes!("../InterVariable.ttf") as &[u8];
        let font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();
        let (metrics, text_bitmap) = font.rasterize('b', 256.0);
        let text_bitmap: Vec<_> = text_bitmap
            .iter()
            .flat_map(|coverage| vec![255, 255, 255, *coverage])
            .collect();
        //let texture =
        //    ctx.new_texture_from_rgba8(metrics.width as u16, metrics.height as u16, &text_bitmap);

        let img = image::load_from_memory(include_bytes!("../king.png"))
            .unwrap()
            .to_rgba8();
        let width = img.width() as u16;
        let height = img.height() as u16;
        let king_bitmap = img.into_raw();
        let king_texture = ctx.new_texture_from_rgba8(width, height, &king_bitmap);

        //let bindings = Bindings {
        //    vertex_buffers: vec![vertex_buffer],
        //    index_buffer: index_buffer,
        //    images: vec![texture],
        //};

        let mut shader_meta: ShaderMeta = shader::meta();
        shader_meta
            .uniforms
            .uniforms
            .push(UniformDesc::new("Projection", UniformType::Mat4));
        shader_meta
            .uniforms
            .uniforms
            .push(UniformDesc::new("Model", UniformType::Mat4));

        let shader = ctx
            .new_shader(
                match ctx.info().backend {
                    Backend::OpenGl => ShaderSource::Glsl {
                        vertex: shader::GL_VERTEX,
                        fragment: shader::GL_FRAGMENT,
                    },
                    Backend::Metal => ShaderSource::Msl {
                        program: shader::METAL,
                    },
                },
                shader_meta,
            )
            .unwrap();

        let params = PipelineParams {
            color_blend: Some(BlendState::new(
                Equation::Add,
                BlendFactor::Value(BlendValue::SourceAlpha),
                BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
            )),
            ..Default::default()
        };

        let pipeline = ctx.new_pipeline(
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("in_pos", VertexFormat::Float2),
                VertexAttribute::new("in_color", VertexFormat::Float4),
                VertexAttribute::new("in_uv", VertexFormat::Float2),
            ],
            shader,
            params,
        );

        Stage {
            pipeline,
            //bindings,
            ctx,
            white_texture,
            text_bitmap,
            king_bitmap,
            king_texture,
            last_char: 'a',
            font,
            show_king: true,
            king_dim: (width, height),
        }
    }
}

impl EventHandler for Stage {
    fn update(&mut self) {}

    // Only do drawing here. Apps might not call this when minimized.
    fn draw(&mut self) {
        let (screen_width, screen_height) = window::screen_size();

        // Polygons must have counter-clockwise orientation

        let (font_metrics, text_bitmap) = self.font.rasterize(self.last_char, 256.0);
        let text_bitmap: Vec<_> = text_bitmap
            .iter()
            .flat_map(|coverage| vec![255, 255, 255, *coverage])
            .collect();

        //    0             1
        // (0, 0) ------ (0, 1)
        //   |          /  |
        //   |        /    |
        //   |      /      |
        //   |    /        |
        // (0, 1) ------ (1, 1)
        //    2             3
        //
        // faces: 021, 123
        let vertices: [Vertex; 4] = if self.last_char == ' ' && !self.show_king {
            [
                // top left
                Vertex {
                    pos: [0.25, 0.25],
                    color: [1., 0., 1., 1.],
                    uv: [0., 0.],
                },
                // top right
                Vertex {
                    pos: [0.75, 0.25],
                    color: [1., 1., 0., 1.],
                    uv: [1., 0.],
                },
                // bottom left
                Vertex {
                    pos: [0.25, 0.75],
                    color: [0., 0., 0.8, 1.],
                    uv: [0., 1.],
                },
                // bottom right
                Vertex {
                    pos: [0.75, 0.75],
                    color: [1., 1., 0., 1.],
                    uv: [1., 1.],
                },
            ]
        } else {
            let (img_width, img_height) = if self.last_char == ' ' && self.show_king {
                (self.king_dim.0 as f32, self.king_dim.1 as f32)
            } else {
                (font_metrics.width as f32, font_metrics.height as f32)
            };
            let scale = 4.0;
            let width = scale * img_width / screen_width;
            let height = scale * img_height / screen_height;

            [
                Vertex {
                    pos: [0.25, 0.25],
                    color: [1., 1., 1., 1.],
                    uv: [0., 0.],
                },
                Vertex {
                    pos: [0.25 + width, 0.25],
                    color: [1., 1., 1., 1.],
                    uv: [1., 0.],
                },
                Vertex {
                    pos: [0.25, 0.25 + height],
                    color: [1., 1., 1., 1.],
                    uv: [0., 1.],
                },
                Vertex {
                    pos: [0.25 + width, 0.25 + height],
                    color: [1., 1., 1., 1.],
                    uv: [1., 1.],
                },
            ]
        };
        //debug!("screen size: {:?}", window::screen_size());
        let vertex_buffer = self.ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&vertices),
        );

        let indices: [u16; 6] = [0, 2, 1, 1, 2, 3];
        let index_buffer = self.ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&indices),
        );

        let texture = if self.last_char == ' ' {
            if self.show_king {
                let (width, height) = self.king_dim;
                self.king_texture
            } else {
                self.white_texture
            }
        } else {
            self.ctx.new_texture_from_rgba8(
                font_metrics.width as u16,
                font_metrics.height as u16,
                &text_bitmap,
            )
        };

        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer: index_buffer,
            images: vec![texture],
        };

        //self.ctx.begin_default_pass(Default::default());
        self.ctx
            .begin_default_pass(PassAction::clear_color(0., 0., 0., 1.));

        self.ctx.apply_pipeline(&self.pipeline);

        // This will make the top left (0, 0) and the bottom right (1, 1)
        // Default is (-1, 1) -> (1, -1)
        let proj = glam::Mat4::from_translation(glam::Vec3::new(-1., 1., 0.))
            * glam::Mat4::from_scale(glam::Vec3::new(2., -2., 1.));

        let mut uniforms_data = [0u8; 128];
        let data: [u8; 64] = unsafe { std::mem::transmute_copy(&proj) };
        uniforms_data[0..64].copy_from_slice(&data);

        let (off_x, off_y) = (0., 0.);
        //let scale_w = 1. / screen_width;
        //let scale_h = 1. / screen_height;
        let (scale_w, scale_h) = (1., 1.);
        let model = glam::Mat4::from_translation(glam::Vec3::new(off_x, off_y, 0.))
            * glam::Mat4::from_scale(glam::Vec3::new(scale_w, scale_h, 1.));

        let data: [u8; 64] = unsafe { std::mem::transmute_copy(&model) };
        uniforms_data[64..].copy_from_slice(&data);
        self.ctx
            .apply_uniforms_from_bytes(uniforms_data.as_ptr(), uniforms_data.len());

        self.ctx.apply_bindings(&bindings);
        self.ctx.draw(0, 6, 1);
        self.ctx.end_render_pass();

        self.ctx.commit_frame();
    }

    fn key_down_event(&mut self, keycode: KeyCode, modifiers: KeyMods, repeat: bool) {
        if repeat {
            return;
        }
        match keycode {
            KeyCode::A => {
                if modifiers.shift {
                    self.last_char = 'A'
                } else {
                    self.last_char = 'a'
                }
            }
            KeyCode::B => {
                if modifiers.shift {
                    self.last_char = 'B'
                } else {
                    self.last_char = 'b'
                }
            }
            KeyCode::C => {
                if modifiers.shift {
                    self.last_char = 'C'
                } else {
                    self.last_char = 'c'
                }
            }
            KeyCode::D => {
                if modifiers.shift {
                    self.last_char = 'D'
                } else {
                    self.last_char = 'd'
                }
            }
            KeyCode::E => {
                if modifiers.shift {
                    self.last_char = 'E'
                } else {
                    self.last_char = 'e'
                }
            }
            KeyCode::F => {
                if modifiers.shift {
                    self.last_char = 'F'
                } else {
                    self.last_char = 'f'
                }
            }
            KeyCode::G => {
                if modifiers.shift {
                    self.last_char = 'G'
                } else {
                    self.last_char = 'g'
                }
            }
            KeyCode::H => {
                if modifiers.shift {
                    self.last_char = 'H'
                } else {
                    self.last_char = 'h'
                }
            }
            KeyCode::I => {
                if modifiers.shift {
                    self.last_char = 'I'
                } else {
                    self.last_char = 'i'
                }
            }
            KeyCode::J => {
                if modifiers.shift {
                    self.last_char = 'J'
                } else {
                    self.last_char = 'j'
                }
            }
            KeyCode::K => {
                if modifiers.shift {
                    self.last_char = 'K'
                } else {
                    self.last_char = 'k'
                }
            }
            KeyCode::L => {
                if modifiers.shift {
                    self.last_char = 'L'
                } else {
                    self.last_char = 'l'
                }
            }
            KeyCode::M => {
                if modifiers.shift {
                    self.last_char = 'M'
                } else {
                    self.last_char = 'm'
                }
            }
            KeyCode::N => {
                if modifiers.shift {
                    self.last_char = 'N'
                } else {
                    self.last_char = 'n'
                }
            }
            KeyCode::O => {
                if modifiers.shift {
                    self.last_char = 'O'
                } else {
                    self.last_char = 'o'
                }
            }
            KeyCode::P => {
                if modifiers.shift {
                    self.last_char = 'P'
                } else {
                    self.last_char = 'p'
                }
            }
            KeyCode::Q => {
                if modifiers.shift {
                    self.last_char = 'Q'
                } else {
                    self.last_char = 'q'
                }
            }
            KeyCode::R => {
                if modifiers.shift {
                    self.last_char = 'R'
                } else {
                    self.last_char = 'r'
                }
            }
            KeyCode::S => {
                if modifiers.shift {
                    self.last_char = 'S'
                } else {
                    self.last_char = 's'
                }
            }
            KeyCode::T => {
                if modifiers.shift {
                    self.last_char = 'T'
                } else {
                    self.last_char = 't'
                }
            }
            KeyCode::U => {
                if modifiers.shift {
                    self.last_char = 'U'
                } else {
                    self.last_char = 'u'
                }
            }
            KeyCode::V => {
                if modifiers.shift {
                    self.last_char = 'V'
                } else {
                    self.last_char = 'v'
                }
            }
            KeyCode::W => {
                if modifiers.shift {
                    self.last_char = 'W'
                } else {
                    self.last_char = 'w'
                }
            }
            KeyCode::X => {
                if modifiers.shift {
                    self.last_char = 'X'
                } else {
                    self.last_char = 'x'
                }
            }
            KeyCode::Y => {
                if modifiers.shift {
                    self.last_char = 'Y'
                } else {
                    self.last_char = 'y'
                }
            }
            KeyCode::Z => {
                if modifiers.shift {
                    self.last_char = 'Z'
                } else {
                    self.last_char = 'z'
                }
            }
            KeyCode::Space => {
                self.last_char = ' ';
                self.show_king = true;
            }
            KeyCode::Enter => {
                self.last_char = ' ';
                self.show_king = false;
            }
            _ => {}
        }
        //debug!("{:?}", keycode);
    }
    //fn mouse_motion_event(&mut self, x: f32, y: f32) {
    //    //println!("{} {}", x, y);
    //}
    //fn mouse_wheel_event(&mut self, x: f32, y: f32) {
    //    println!("{} {}", x, y);
    //}
    fn mouse_button_down_event(&mut self, button: MouseButton, x: f32, y: f32) {
        self.last_char = ' ';
        self.show_king = true;
        window::show_keyboard(true);
        //println!("{:?} {} {}", button, x, y);
    }
    //fn mouse_button_up_event(&mut self, button: MouseButton, x: f32, y: f32) {
    //    //println!("{:?} {} {}", button, x, y);
    //}

    fn resize_event(&mut self, width: f32, height: f32) {
        //debug!("resize! {} {}", width, height);
    }
}

fn main() {
    let mut conf = miniquad::conf::Conf {
        window_title: "DarkFi".to_string(),
        high_dpi: true,
        window_resizable: true,
        platform: miniquad::conf::Platform {
            linux_backend: miniquad::conf::LinuxBackend::WaylandWithX11Fallback,
            android_panic_hook: false,
            ..Default::default()
        },
        ..Default::default()
    };
    let metal = std::env::args().nth(1).as_deref() == Some("metal");
    conf.platform.apple_gfx_api = if metal {
        conf::AppleGfxApi::Metal
    } else {
        conf::AppleGfxApi::OpenGl
    };

    miniquad::start(conf, || {
        window::show_keyboard(true);
        Box::new(Stage::new())
    });
}
