use winit::{CreationError, Event, EventsLoop, Window, WindowBuilder, WindowEvent};
use winit::dpi::LogicalSize;

pub const WINDOW_NAME: &str = "Hello Winit";

#[derive(Debug)]
pub struct WinitState {
    pub events_loop: EventsLoop,
    pub window: Window,
}

impl Default for WinitState {
    
    fn default() -> Self {
        Self::new(
            WINDOW_NAME,
            LogicalSize {
                width: 800.0,
                height: 600.0,
            },
        )
        .expect("Could not create a window!")
    }
}

impl WinitState {
    pub fn new<T: Into<String>>(title: T, size: LogicalSize) -> Result<Self, CreationError> {
        let events_loop = EventsLoop::new();
        let output = WindowBuilder::new()
            .with_title(title)
            .with_dimensions(size)
            .build(&events_loop);
        output.map(|window| Self {
            events_loop,
            window,
        })
    }
}

pub fn do_the_render(hal: &mut HalState, locals: &LocalState) -> Result<(), &str> {
    hal.draw_clear_frame(locals.color())
}

fn main() {
    let mut winit_state = WinitState::default();
    let mut hal_state = HalState::new(&winit_state.window);
    let mut local_state = LocalState::default();

    loop {
        let inputs = UserInput::poll_events_loop(&mut winit_state.event_loop);
        if inputs.end_requested { break; }
        local_state.update_from_input(inputs);
        if let Err(e) = do_the_render(&mut hal_state, &local_state) {
            error!("Rendering Error: {:?}", e);
            break;
        }
    }

}
//    let mut running = true;
//
//    while running {
//        winit_state.events_loop.poll_events(|event| match event {
//            Event::WindowEvent {
//                event: WindowEvent::CloseRequested,
//                ..
//            } => running = false,
//            _ => (),
//        });
//    }
