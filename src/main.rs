use std::{
	cmp::Ordering,
	num::NonZeroU32,
	ops::DerefMut,
	rc::Rc,
	time::{Duration, Instant},
};

use gallop::{Gallop, GallopEvent, GALLOP_TOLERANCE};
use gilrs::{Axis, Button, Gilrs};
use image::Image;
use softbuffer::{Context, Surface};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;
use winit::{
	application::ApplicationHandler,
	dpi::LogicalSize,
	event::{KeyEvent, WindowEvent},
	event_loop::{ControlFlow, EventLoop},
	keyboard::{Key, NamedKey},
	window::Window,
};

mod gallop;
mod image;

fn main() {
	setup_logging();

	let el = EventLoop::new().unwrap();
	// We NEED poll here because of how gilrs does events
	el.set_control_flow(ControlFlow::Poll);

	let mut etch = Etch {
		window: None,
		gilrs: Gilrs::new().unwrap(),
		img: Image::new(WIDTH as u32, HEIGHT as u32, Some(BACKGROUND_COLOUR.into())),
		stylus: Vec2 {
			x: 10.0,
			y: HEIGHT / 2.0,
		},

		dial: DialState::default(),
		left_angle: 0.0,
		right_angle: 0.0,
		next_check: Instant::now(),

		gallop_x: Gallop::default(),
		gallop_y: Gallop::default(),
		next_gallop_check: Instant::now(),
	};

	el.run_app(&mut etch).unwrap();
}

fn setup_logging() {
	let env_filter =
		EnvFilter::builder().with_default_directive(LevelFilter::INFO.into()).from_env().unwrap();

	tracing_subscriber::fmt().with_env_filter(env_filter).init();
}

#[derive(Copy, Clone, Debug, Default)]
struct DialState {
	left: Vec2<f32>,
	right: Vec2<f32>,
}

struct SurfacedWindow {
	window: Rc<Window>,
	surface: Surface<Rc<Window>, Rc<Window>>,
}

struct Etch {
	window: Option<SurfacedWindow>,
	gilrs: Gilrs,
	img: Image,
	stylus: Vec2<f32>,

	dial: DialState,
	left_angle: f32,
	right_angle: f32,
	next_check: Instant,

	gallop_x: Gallop,
	gallop_y: Gallop,
	next_gallop_check: Instant,
}

impl Etch {
	pub fn process_gamepad_events(&mut self) {
		while let Some(gilrs::Event {
			id, event, time, ..
		}) = self.gilrs.next_event()
		{
			match event {
				gilrs::EventType::AxisChanged(axis, value, _code) if !value.is_nan() => {
					match axis {
						Axis::LeftStickX => self.dial.left.x = value * 100.0,
						Axis::LeftStickY => self.dial.left.y = value * 100.0,
						Axis::RightStickX => self.dial.right.x = value * 100.0,
						Axis::RightStickY => self.dial.right.y = value * 100.0,
						_ => (),
					}
				}
				gilrs::EventType::ButtonPressed(btn, code) => match btn {
					Button::South => self.clear_pressed(),
					_ => (),
				},
				_ => (),
			}
		}
	}
}

impl Etch {
	pub fn clear_pressed(&mut self) {
		self.img.fill(BACKGROUND_COLOUR.into());
	}

	pub fn save(&self) {
		let location = rfd::FileDialog::new()
			.add_filter("gif", &["gif"])
			.add_filter("png", &["png"])
			.set_file_name("etch.gif")
			.save_file();

		if let Some(path) = location {
			tracing::info!("saving gif to {}", path.to_string_lossy());
			let gif = self.img.gif();
			gif.save(path).unwrap();
		}
	}
}

// Why are my consts HERE of all places
const DIAL_SENSITIVITY: f32 = 10.0;
const WIDTH: f32 = 640.0;
const HEIGHT: f32 = 480.0;

// a very sublte gentle, dark-and-dull green
const BACKGROUND_COLOUR: u32 = 0x00868886;
const LINE_COLOUR: u32 = 0x00303230;
const STYLUS_COLOUR: u32 = 0x00a0a0a0;

impl ApplicationHandler for Etch {
	fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
		let window = Rc::new(event_loop.create_window(Window::default_attributes()).unwrap());
		window.set_title("that really etches my sketch");
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
			WindowEvent::KeyboardInput {
				device_id,
				event,
				is_synthetic,
			} => {
				let KeyEvent {
					logical_key,
					state,
					repeat,
					..
				} = event;

				// We only track keys down and non-repeat
				if repeat || !state.is_pressed() {
					return;
				}

				match logical_key.as_ref() {
					Key::Character("a") => self.gallop_x.push(0),
					Key::Character("s") => self.gallop_x.push(1),
					Key::Character("d") => self.gallop_x.push(2),
					Key::Character("f") => self.gallop_x.push(3),
					Key::Character("j") => self.gallop_y.push(0),
					Key::Character("k") => self.gallop_y.push(1),
					Key::Character("l") => self.gallop_y.push(2),
					Key::Character(";") => self.gallop_y.push(3),

					Key::Named(NamedKey::Backspace) => self.clear_pressed(),

					Key::Character("p") => self.save(),
					_ => (),
				}
			}
			WindowEvent::RedrawRequested => {
				self.process_gamepad_events();

				let stylus_prev = self.stylus;

				// We check the state of the joystick at 40fps
				if self.next_check.elapsed() > Duration::from_millis(25) {
					let left_angle = xy_to_deg(self.dial.left.x, self.dial.left.y);

					let left_is_large = self.dial.left.mag() > 50.0;
					let left_neither_nan = !left_angle.is_nan() && !self.left_angle.is_nan();

					let left_delta = if left_is_large && left_neither_nan {
						let delta = angle_delta(left_angle, self.left_angle);
						delta
					} else {
						0.0
					};
					self.left_angle = left_angle;

					let right_angle = xy_to_deg(self.dial.right.x, self.dial.right.y);

					let right_is_large = self.dial.right.mag() > 50.0;
					let right_neither_nan = !right_angle.is_nan() && !self.right_angle.is_nan();

					let right_delta = if right_is_large && right_neither_nan {
						let delta = angle_delta(right_angle, self.right_angle);
						delta
					} else {
						0.0
					};
					self.right_angle = right_angle;

					tracing::trace!(
						"ANGLE ({}) {left_angle} // {left_delta}v -=- ({}) {right_angle} // {right_delta}v",
						self.dial.left.mag(), self.dial.right.mag()
					);

					let movement_x = left_delta / DIAL_SENSITIVITY;
					let movement_y = right_delta / DIAL_SENSITIVITY;
					self.stylus.x =
						(self.stylus.x + movement_x).clamp(0.0, self.img.width() as f32);
					self.stylus.y =
						(self.stylus.y - movement_y).clamp(0.0, self.img.height() as f32);

					self.next_check = Instant::now();
				}

				if self.next_gallop_check.elapsed() > Duration::from_millis(25) {
					while let Some(ge) = self.gallop_x.event() {
						match ge {
							GallopEvent::Positive(gdur) => {
								if gdur <= GALLOP_TOLERANCE {
									self.stylus.x += ge.value();
								}
							}
							GallopEvent::Negative(gdur) => {
								if gdur <= GALLOP_TOLERANCE {
									self.stylus.x += ge.value();
								}
							}
						}
					}

					while let Some(ge) = self.gallop_y.event() {
						match ge {
							GallopEvent::Positive(gdur) => {
								if gdur <= GALLOP_TOLERANCE {
									self.stylus.y += ge.value();
								}
							}
							GallopEvent::Negative(gdur) => {
								if gdur <= GALLOP_TOLERANCE {
									self.stylus.y += ge.value();
								}
							}
						}
					}
				}

				// If the stylus moved, we should draw
				if stylus_prev != self.stylus {
					self.img.line(
						self.stylus.as_u32(),
						stylus_prev.as_u32(),
						2,
						LINE_COLOUR.into(),
					);

					// Letting the line-draw take care of overdrawing the stylus
					// worked well except going up for some reason, so we
					// explicitly overdraw here.
					self.img.rect(stylus_prev.as_u32(), Vec2::new(2, 2), LINE_COLOUR.into());

					// And then draw the stylus
					self.img.rect(self.stylus.as_u32(), Vec2::new(2, 2), STYLUS_COLOUR.into());

					tracing::trace!("STYLUS: ({},{})", self.stylus.x, self.stylus.y);
				}

				let Some(surfaced) = self.window.as_mut() else {
					tracing::warn!("self.window is None in Redraw!");
					return;
				};

				let (window_width, window_height) = {
					let phys = surfaced.window.inner_size();
					(phys.width as usize, phys.height as usize)
				};

				let mut buffer = surfaced.surface.buffer_mut().unwrap();
				neam::nearest_buffer(
					self.img.data(),
					1,
					self.img.width(),
					self.img.height(),
					buffer.deref_mut(),
					window_width as u32,
					window_height as u32,
				);

				surfaced.window.pre_present_notify();
				buffer.present().unwrap();
				surfaced.window.request_redraw();
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
	// If we move too fast, or flick the stick just right, these values can be
	// bypassed. Perhaps we can check on either side of 180, but this is working
	// for now. It used to be 270 and 90 for high/low, respectively, but it was
	// getting a touch jumpy.
	const HIGH: f32 = 225.0;
	const LOW: f32 = 135.0;

	if rhs >= HIGH && lhs < LOW {
		// It is likely we crossed zero in the clockwise direction
		(lhs + 360.0) - rhs
	} else if rhs < LOW && lhs > HIGH {
		// It is likely we crossed zero in the anti-clockwise direction
		lhs - (rhs + 360.0)
	} else {
		lhs - rhs
	}
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Vec2<T> {
	pub x: T,
	pub y: T,
}

impl<T> Vec2<T> {
	pub fn new(x: T, y: T) -> Self {
		Self { x, y }
	}
}

impl Vec2<f32> {
	pub fn as_u32(&self) -> Vec2<u32> {
		Vec2 {
			x: self.x as u32,
			y: self.y as u32,
		}
	}

	pub fn mag(&self) -> f32 {
		(self.x * self.x + self.y * self.y).sqrt()
	}
}
