use std::{
	num::NonZeroU32,
	rc::Rc,
	time::{Duration, Instant},
};

use gilrs::{Axis, GamepadId, Gilrs};
use softbuffer::{Buffer, Context, Surface};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;
use winit::{
	application::ApplicationHandler,
	dpi::LogicalSize,
	event::{DeviceEvent, WindowEvent},
	event_loop::{ControlFlow, EventLoop},
	window::Window,
};

fn main() {
	setup_logging();

	let el = EventLoop::new().unwrap();
	// We NEED poll here because of how gilrs does events
	el.set_control_flow(ControlFlow::Poll);

	let mut etch = Etch {
		window: None,
		gilrs: Gilrs::new().unwrap(),

		dial: DialState::default(),
		left_angle: 0.0,
		next_check: Instant::now(),
	};

	el.run_app(&mut etch);
}

fn setup_logging() {
	let env_filter =
		EnvFilter::builder().with_default_directive(LevelFilter::INFO.into()).from_env().unwrap();

	tracing_subscriber::fmt().with_env_filter(env_filter).init();
}

#[derive(Copy, Clone, Debug, Default)]
struct DialState {
	left_x: f32,
	left_y: f32,

	right_x: f32,
	right_y: f32,
}

struct SurfacedWindow {
	window: Rc<Window>,
	surface: Surface<Rc<Window>, Rc<Window>>,
}

struct Etch {
	window: Option<SurfacedWindow>,
	gilrs: Gilrs,

	dial: DialState,
	left_angle: f32,
	next_check: Instant,
}

const DIAL_SENSETIVITY: f32 = 2.0;
const WIDTH: f32 = 640.0;
const HEIGHT: f32 = 480.0;

impl ApplicationHandler for Etch {
	fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
		let window = Rc::new(event_loop.create_window(Window::default_attributes()).unwrap());
		window.set_resizable(false);
		window.request_inner_size(LogicalSize::new(WIDTH, HEIGHT));

		let ctx = Context::new(window.clone()).unwrap();
		let surface = Surface::new(&ctx, window.clone()).unwrap();
		self.window = Some(SurfacedWindow { window, surface });
	}

	fn window_event(
		&mut self,
		event_loop: &winit::event_loop::ActiveEventLoop,
		window_id: winit::window::WindowId,
		event: winit::event::WindowEvent,
	) {
		match event {
			WindowEvent::CloseRequested => {
				tracing::info!("close requested! shutting down.");
				event_loop.exit();
			}
			WindowEvent::RedrawRequested => {
				let previous_dial = self.dial;

				// Process gamepad events
				while let Some(gilrs::Event {
					id, event, time, ..
				}) = self.gilrs.next_event()
				{
					match event {
						gilrs::EventType::AxisChanged(axis, value, _code) => {
							tracing::trace!("{axis:?} value={value}");

							match axis {
								Axis::LeftStickX => self.dial.left_x = value * 100.0,
								Axis::LeftStickY => self.dial.left_y = value * 100.0,
								Axis::RightStickX => self.dial.right_x = value * 100.0,
								Axis::RightStickY => self.dial.right_y = value * 100.0,
								_ => (),
							}
						}
						_ => (),
					}
				}

				// We check the state of the joystick at 20fps
				if self.next_check.elapsed() > Duration::from_millis(50) {
					let left_angle = xy_to_deg(self.dial.left_x, self.dial.left_y);

					let left_delta = angle_delta(left_angle, self.left_angle);
					self.left_angle = left_angle;

					tracing::info!("ANGLE {left_angle} // {left_delta}v");
					self.next_check = Instant::now();
				}

				let Some(surfaced) = self.window.as_mut() else {
					tracing::warn!("self.window is None in Redraw!");
					return;
				};

				let (width, height) = {
					let phys = surfaced.window.inner_size();
					(phys.width as usize, phys.height as usize)
				};

				let mut buffer = surfaced.surface.buffer_mut().unwrap();
				for idx in 0..width {
					buffer[idx + (height / 2) * width] = 0xFF00FF00;
				}

				buffer.present().unwrap();
				//surfaced.window.request_redraw();
			}
			WindowEvent::Resized(phys) => {
				tracing::trace!("resized window: {phys:?}");

				if let Some(surfaced) = self.window.as_mut() {
					surfaced
						.surface
						.resize(
							NonZeroU32::new(phys.width).unwrap(),
							NonZeroU32::new(phys.height).unwrap(),
						)
						.unwrap();
				}
			}
			_ => (),
		}
	}
}

fn xy_to_deg(x: f32, y: f32) -> f32 {
	let neg_x = x < 0.0;
	let neg_y = y < 0.0;
	let raw_angle = (y.abs() / x.abs()).atan().to_degrees();
	let raw_angle2 = (x.abs() / y.abs()).atan().to_degrees();

	match (neg_x, neg_y) {
		(false, false) => raw_angle2 + 270.0,
		(false, true) => raw_angle,
		(true, true) => raw_angle2 + 90.0,
		(true, false) => raw_angle + 180.0,
	}
}

/// Compute the difference in angle between the right-hand side
/// and the left-hand side. Intelligently handles the zero-crossing.
/// lhs should be the newer value, rhs the older.
fn angle_delta(lhs: f32, rhs: f32) -> f32 {
	if rhs >= 270.0 && lhs < 90.0 {
		// It is likely we crossed zero in the clockwise direction
		(lhs + 360.0) - rhs
	} else if rhs < 90.0 && lhs > 270.0 {
		// It is likely we crossed zero in the anti-clockwise direction
		lhs - (rhs + 360.0)
	} else {
		lhs - rhs
	}
}
