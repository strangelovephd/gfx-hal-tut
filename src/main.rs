#[cfg(feature = "dx12")]
use gfx_backend_dx12 as back;
#[cfg(feature = "metal")]
use gfx_backend_metal as back;
#[cfg(feature = "vulkan")]
use gfx_backend_vulkan as back;

use arrayvec::ArrayVec;
use core::mem::ManuallyDrop;
use gfx_hal::{
  adapter::{Adapter, PhysicalDevice},
  command::{ClearColor, ClearValue, CommandBuffer, MultiShot, Primary},
  device::Device,
  format::{Aspects, ChannelType, Format, Swizzle},
  image::{Extent, Layout, SubresourceRange, Usage, ViewKind},
  pass::{Attachment, AttachmentLoadOp, AttachmentOps, AttachmentStoreOp, SubpassDesc},
  pool::{CommandPool, CommandPoolCreateFlags},
  pso::{PipelineStage, Rect},
  queue::{family::QueueGroup, Submission},
  window::{Backbuffer, FrameSync, PresentMode, Swapchain, SwapchainConfig},
  Backend, Gpu, Graphics, Instance, QueueFamily, Surface,
};
use winit::{
  dpi::LogicalSize, CreationError, Event, EventsLoop, Window, WindowBuilder, WindowEvent,
};

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

pub struct HalState {
  current_frame: usize,
  frames_in_flight: usize,
  in_flight_fences: Vec<<back::Backend as Backend>::Fence>,
  render_finished_semaphores: Vec<<back::Backend as Backend>::Semaphore>,
  image_available_semaphores: Vec<<back::Backend as Backend>::Semaphore>,
  command_buffers: Vec<CommandBuffer<back::Backend, Graphics, MultiShot, Primary>>,
  command_pool: ManuallyDrop<CommandPool<back::Backend, Graphics>>,
  framebuffers: Vec<<back::Backend as Backend>::Framebuffer>,
  image_views: Vec<(<back::Backend as Backend>::ImageView)>,
  render_pass: ManuallyDrop<<back::Backend as Backend>::RenderPass>,
  render_area: Rect,
  queue_group: QueueGroup<back::Backend, Graphics>,
  swapchain: ManuallyDrop<<back::Backend as Backend>::Swapchain>,
  device: ManuallyDrop<back::Device>,
  _adapter: Adapter<back::Backend>,
  _surface: <back::Backend as Backend>::Surface,
  _instance: ManuallyDrop<back::Instance>,
}

impl HalState {
    pub fn new(window: &Window) -> Result<Self, &'static str> {
        // CREATE INSTANCE
        let instance = back::Instance::create(WINDOW_NAME, 1);

        // CREATE SURFACE
        let mut surface = instance.create_surface(window);

        // CREATE ADAPTER
        let adapter = instance
            .enumerate_adapters()
            .into_iter()
            .find(|a| {
                a.queue_families
                    .iter()
                    .any(|qf| qf.supports_graphics() && surface.supports_queue_family(qf))
            })
            .ok_or("Couldn't find a graphical adapter!")?;

        // DEVICE AND QUEUEGROUP
        let (device, queue_group) = {
            let queue_family = adapter
                .queue_families
                .iter()
                .find(|qf| qf.supports_graphics() && surface.supports_queue_family(qf))
                .ok_or("Couldn't find a QueueFamily with graphics!")?;
            let GPU { device, mut queues } = unsafe {
                adapter
                    .physical_device
                    .open(&[(&queue_family, &[1.0; 1])])
                    .map_err(|_| "Couldn't open the PhysicalDevice!")?
            };
            let queue_group = queues
                .take::<Graphics>(queue_family.id())
                .ok_or("Couldn't take ownership of the QueueGroup!")?;
            let _ = if queue_group.queues.len() > 0 {
                Ok(())
            } else {
                Err("The QueueGroup did not have any CommandQueues available!")
            }?;
            (device, queue_group)
        };

        // TODO: SWAPCHAIN        
    }

    pub fn draw_clear_frame(&mut self, color: [f32; 4]) -> Result<(), &'static str> {
        // SETUP FOR THIS FRAME
        let image_available = &self.image_available_semaphores[self.current_frame];
        let render_finished = &self.render_finished_semaphores[self.current_frame];
        // Advance the frame before we start using the '?' operator
        self.current_frame = (self.current_frame + 1) % self.frames_in_flight;

        let (i_u32, i_usize) = unsafe {
            let image_index = self
                .swapchain
                .acquire_image(core::u64::MAX, FrameSync::Semaphore(image_available))
                .map_err(|_| "Couldn't acquire an image from the swapchain!")?;
            (image_index, image_index as usize);
        };


        // RECORD SOME COMMANDS
        unsafe {
            let buffer = &mut self.command_bufferrs[i_usize];
            let clear_values = [ClearValue::Color(ClearColor::Float(color))];
            buffer.begin(false);
            buffer.begin_render_pass_inline(
                &self.render_pass,
                &self.swapchain_framebuffers[i_usize],
                self.render_area,
                clear_values.iter(),
            );
            buffer.finish();
        }

        // SUBMISSION
        let command_buffers: ArrayVec<[_; 1]> = [the_command_buffer].into();
        let wait_semaphores: ArrayVec<[_; 1]> = [(image_available, PipelineStage::COLOR_ATTACHMENT_OUTPUT)].into();
        let signal_semaphores: ArrayVec<[_; 1]> = [render_finished].into();
        let present_wait_semaphores: ArrayVec<[_; 1]> = [render_finished].into();
        let submission = Submission {
            command_buffers,
            wait_semaphores,
            signal_semaphores,
        };

        unsafe {
            the_command_queue.submit(submission, Some(flight_fence));
            the_swapchain.present(&mut the_command_queue, i_u32, present_wait_semaphores)
                .map_err(|_| "Failed to present into the swapchain!")
        }
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
