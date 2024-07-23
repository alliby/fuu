pub mod app;
pub mod state;
pub mod image;
pub mod scenes;
pub mod themes;
pub mod utils;

use anyhow::Result;
use vello::Scene;
use vello::util::RenderContext;
use winit::event_loop::EventLoop;

fn main() -> Result<()> {
    let paths = utils::read_args();

    if paths.is_empty() {
        panic!("No valid image found");
    }
    
    let mut app = app::App {
	context: RenderContext::new(),
	renderers: vec![],
	render_state: app::RenderState::Suspended(None),
	app_state: state::AppState::new(paths),
	scene: Scene::new(),
    };

    // Create and run a winit event loop
    let event_loop = EventLoop::new()?;
    event_loop
        .run_app(&mut app)
        .expect("Couldn't run event loop");

    Ok(())
}
