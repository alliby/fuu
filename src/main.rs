pub mod app;
pub mod image;
pub mod scenes;
pub mod themes;
pub mod utils;

use app::{handle_key, AppState};
use scenes::Scenes;
use themes::Theme;

use anyhow::Result;
use std::sync::Arc;
use vello::util::{RenderContext, RenderSurface};
use vello::{AaConfig, Renderer, RendererOptions};
use winit::event::*;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

// Simple struct to hold the state of the renderer
pub struct RenderState<'s> {
    // The fields MUST be in this order, so that the surface is dropped before the window
    surface: RenderSurface<'s>,
    window: Arc<Window>,
}

fn main() -> Result<()> {
    let paths = utils::read_args();

    if paths.is_empty() {
        panic!("No valid image found");
    }

    // The vello RenderContext which is a global context that lasts for the lifetime of the application
    let mut render_cx = RenderContext::new().unwrap();

    // An array of renderers, one per wgpu device
    let mut renderers: Vec<Option<Renderer>> = vec![];

    // Create and run a winit event loop
    let event_loop = EventLoop::new()?;

    let window = create_winit_window(&event_loop);

    // Create a vello Surface
    let size = window.inner_size();
    let surface_future = render_cx.create_surface(
        window.clone(),
        size.width,
        size.height,
        wgpu::PresentMode::AutoVsync,
    );

    let surface = pollster::block_on(surface_future).expect("Error creating surface");

    // Create a vello Renderer for the surface (using its device id)
    renderers.resize_with(render_cx.devices.len(), || None);
    renderers[surface.dev_id].get_or_insert_with(|| create_vello_renderer(&render_cx, &surface));

    let mut app_state = AppState::new(paths);
    let mut scenes = Scenes::default();
    let mut render_state = RenderState {
        surface,
        window: window.clone(),
    };

    event_loop
        .run(move |event, event_loop| match event {
            Event::WindowEvent { ref event, .. } => match event {
                // Exit the event loop when a close is requested (e.g. window's close button is pressed)
                WindowEvent::CloseRequested => event_loop.exit(),

                // Resize the surface when the window is resized
                WindowEvent::Resized(size) => {
                    app_state.window_size = (size.width, size.height);
                    render_cx.resize_surface(&mut render_state.surface, size.width, size.height);
                    render_state.window.request_redraw();
                }

                // This is where all the rendering happens
                WindowEvent::RedrawRequested => {
                    // Get the window size
                    let width = render_state.surface.config.width;
                    let height = render_state.surface.config.height;
                    // Get a handle to the device
                    let device_handle = &render_cx.devices[render_state.surface.dev_id];

                    let request_redraw = scenes::draw(&mut scenes, &mut app_state);
                    if request_redraw {
                        render_state.window.request_redraw();
                    }

                    let surface = &render_state.surface;

                    // Get the surface's texture
                    let surface_texture = surface
                        .surface
                        .get_current_texture()
                        .expect("failed to get surface texture");

                    // Render to the surface's texture
                    renderers[surface.dev_id]
                        .as_mut()
                        .unwrap()
                        .render_to_surface(
                            &device_handle.device,
                            &device_handle.queue,
                            &scenes.main,
                            &surface_texture,
                            &vello::RenderParams {
                                base_color: Theme::ALL[app_state.active_theme].background, // Background color
                                width,
                                height,
                                antialiasing_method: AaConfig::Msaa8,
                            },
                        )
                        .expect("failed to render to surface");

                    // Queue the texture to be presented on the surface
                    surface_texture.present();

                    device_handle.device.poll(wgpu::Maintain::Poll);
                }
                WindowEvent::KeyboardInput { event, .. } => {
                    if handle_key(&mut app_state, event) {
                        render_state.window.request_redraw();
                    }
                }
                _ => {}
            },
            _ => {}
        })
        .expect("Couldn't run event loop");
    Ok(())
}

/// Helper function that creates a Winit window and returns it (wrapped in an Arc for sharing between threads)
fn create_winit_window(event_loop: &winit::event_loop::EventLoopWindowTarget<()>) -> Arc<Window> {
    Arc::new(
        WindowBuilder::new()
            .with_resizable(true)
            .with_title("Fuu - Image Viewer")
            .build(event_loop)
            .unwrap(),
    )
}

/// Helper function that creates a vello `Renderer` for a given `RenderContext` and `RenderSurface`
fn create_vello_renderer(render_cx: &RenderContext, surface: &RenderSurface) -> Renderer {
    Renderer::new(
        &render_cx.devices[surface.dev_id].device,
        RendererOptions {
            surface_format: Some(surface.format),
            use_cpu: false,
            antialiasing_support: vello::AaSupport::all(),
            num_init_threads: None,
        },
    )
    .expect("Could create renderer")
}
