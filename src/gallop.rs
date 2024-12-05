use std::{
	cmp::Ordering,
	time::{Duration, Instant},
};

pub const GALLOP_SENSETIVITY: Duration = Duration::from_millis(25);
pub const GALLOP_TOLERANCE: Duration = Duration::from_millis(250);

#[derive(Clone, Debug, Default)]
pub struct Gallop {
	keys: [Option<Instant>; 4],
}

impl Gallop {
	pub fn push(&mut self, idx: usize) {
		if idx >= self.keys.len() {
			panic!("gallop index out of bound");
		}

		self.keys[idx] = Some(Instant::now());
	}

	pub fn event(&mut self) -> Option<GallopEvent> {
		let mut sorted: Vec<(usize, Option<Instant>)> = self.keys.into_iter().enumerate().collect();
		sorted.sort_by(|(_, a_inst), (_, b_inst)| {
			if a_inst.is_none() {
				Ordering::Greater
			} else if b_inst.is_none() {
				Ordering::Less
			} else {
				a_inst.cmp(&b_inst)
			}
		});

		let (early2_idx, Some(early2)) = sorted[0] else {
			return None;
		};

		let (early_idx, Some(early)) = sorted[1] else {
			return None;
		};

		tracing::info!(
			"early = [{}] {}us // early2 = [{}] {}us",
			early_idx,
			early.elapsed().as_micros(),
			early2_idx,
			early2.elapsed().as_micros()
		);

		let high = self.keys.len() - 1;
		if early_idx == 0 && early2_idx == high {
			tracing::trace!("gallop (+) wrap special case");
			self.keys[early2_idx] = None;
			return Some(GallopEvent::Positive(early.duration_since(early2)));
		} else if early_idx == high && early2_idx == 0 {
			tracing::trace!("gallop (-) wrap special case");
		}

		if early_idx > early2_idx {
			let delta = early_idx - early2_idx;
			self.keys[early2_idx] = None;

			if delta > 1 {
				tracing::info!("gallop (+) delta>1");
				None
			} else {
				Some(GallopEvent::Positive(early.duration_since(early2)))
			}
		} else if early_idx < early2_idx {
			let delta = early2_idx - early_idx;
			self.keys[early2_idx] = None;

			if delta > 1 {
				tracing::info!("gallop (-) delta>1");
				None
			} else {
				Some(GallopEvent::Negative(early.duration_since(early2)))
			}
		} else {
			None
		}
	}
}

#[derive(Clone, Debug)]
pub enum GallopEvent {
	Positive(Duration),
	Negative(Duration),
}

impl GallopEvent {
	pub fn value(&self) -> f32 {
		match self {
			Self::Positive(gdur) => {
				let delta = GALLOP_TOLERANCE - *gdur;
				let units = delta.as_millis() as f32 / GALLOP_SENSETIVITY.as_millis() as f32;
				units
			}
			Self::Negative(gdur) => {
				let delta = GALLOP_TOLERANCE - *gdur;
				let units = delta.as_millis() as f32 / GALLOP_SENSETIVITY.as_millis() as f32;
				-units
			}
		}
	}
}
