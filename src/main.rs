#[repr(C)]
#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone)]
struct Primitive {
    translate: [f32; 2],
    // Uncomment this line, lines 63, 67, and line 11 in shader.vert to make this work
    // filler: [f32; 2],
}

fn main() {
    use wgpu::winit::{
        ElementState, Event, EventsLoop, KeyboardInput, VirtualKeyCode, WindowEvent,
    };

    let mut events_loop = EventsLoop::new();

    use wgpu::winit::Window;

    let instance = wgpu::Instance::new();

    let window = Window::new(&events_loop).unwrap();
    let size = window
        .get_inner_size()
        .unwrap()
        .to_physical(window.get_hidpi_factor());

    let surface = instance.create_surface(&window);

    let adapter = instance.get_adapter(&wgpu::AdapterDescriptor {
        power_preference: wgpu::PowerPreference::LowPower,
    });

    let mut device = adapter.request_device(&wgpu::DeviceDescriptor {
        extensions: wgpu::Extensions {
            anisotropic_filtering: false,
        },
        limits: wgpu::Limits::default(),
    });

    let vbo = device
        .create_buffer_mapped(3, wgpu::BufferUsage::VERTEX)
        .fill_from_slice(&[
            Vertex {
                position: [0.0, -0.5],
            },
            Vertex {
                position: [0.5, 0.5],
            },
            Vertex {
                position: [-0.5, 0.5],
            },
        ]);

    let mut primitives = Vec::new();

    primitives.push(Primitive {
        translate: [0.0, 0.0],
        // filler: [0.0, 0.0],
    });
    primitives.push(Primitive {
        translate: [1.0, 0.0],
        // filler: [0.0, 0.0],
    });

    let primitives_buffer_size = (std::mem::size_of::<Primitive>() * primitives.len()) as u64;

    let primitives_ubo = device
        .create_buffer_mapped(primitives.len(), wgpu::BufferUsage::UNIFORM)
        .fill_from_slice(&primitives);

    let (vs_module, fs_module) = {
        let mut compiler = shaderc::Compiler::new().unwrap();

        let vs_bytes = compiler
            .compile_into_spirv(
                include_str!("../shaders/shader.vert"),
                shaderc::ShaderKind::Vertex,
                "shader.vert",
                "main",
                None,
            )
            .unwrap();

        let fs_bytes = compiler
            .compile_into_spirv(
                include_str!("../shaders/shader.frag"),
                shaderc::ShaderKind::Fragment,
                "shader.frag",
                "main",
                None,
            )
            .unwrap();

        (
            device.create_shader_module(vs_bytes.as_binary_u8()),
            device.create_shader_module(fs_bytes.as_binary_u8()),
        )
    };

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        bindings: &[wgpu::BindGroupLayoutBinding {
            binding: 0,
            visibility: wgpu::ShaderStage::VERTEX,
            ty: wgpu::BindingType::UniformBuffer,
        }],
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        bindings: &[wgpu::Binding {
            binding: 0,
            resource: wgpu::BindingResource::Buffer {
                buffer: &primitives_ubo,
                range: 0..primitives_buffer_size,
            },
        }],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[&bind_group_layout],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        layout: &pipeline_layout,
        vertex_stage: wgpu::PipelineStageDescriptor {
            module: &vs_module,
            entry_point: "main",
        },
        fragment_stage: Some(wgpu::PipelineStageDescriptor {
            module: &fs_module,
            entry_point: "main",
        }),
        rasterization_state: wgpu::RasterizationStateDescriptor {
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: wgpu::CullMode::None,
            depth_bias: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
        },
        primitive_topology: wgpu::PrimitiveTopology::TriangleList,
        color_states: &[wgpu::ColorStateDescriptor {
            format: wgpu::TextureFormat::Bgra8Unorm,
            color_blend: wgpu::BlendDescriptor::REPLACE,
            alpha_blend: wgpu::BlendDescriptor::REPLACE,
            write_mask: wgpu::ColorWrite::ALL,
        }],
        depth_stencil_state: None,
        index_format: wgpu::IndexFormat::Uint16,
        vertex_buffers: &[wgpu::VertexBufferDescriptor {
            stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[wgpu::VertexAttributeDescriptor {
                offset: 0,
                format: wgpu::VertexFormat::Float2,
                shader_location: 0,
            }],
        }],
        sample_count: 1,
    });

    let mut swap_chain = device.create_swap_chain(
        &surface,
        &wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: size.width.round() as u32,
            height: size.height.round() as u32,
            present_mode: wgpu::PresentMode::Vsync,
        },
    );
    let mut running = true;
    while running {
        events_loop.poll_events(|event| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(code),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => match code {
                    VirtualKeyCode::Escape => running = false,
                    _ => {}
                },
                WindowEvent::CloseRequested => running = false,
                _ => {}
            },
            _ => {}
        });

        let frame = swap_chain.get_next_texture();
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color::GREEN,
                }],
                depth_stencil_attachment: None,
            });
            rpass.set_pipeline(&render_pipeline);
            rpass.set_bind_group(0, &bind_group, &[]);
            rpass.set_vertex_buffers(&[(&vbo, 0)]);
            rpass.draw(0..3, 0..(primitives.len() as u32));
        }

        device.get_queue().submit(&[encoder.finish()]);
    }
}
