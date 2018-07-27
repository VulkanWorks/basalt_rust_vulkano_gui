use std::sync::{Arc,Weak};
use vulkano;
use std::collections::BTreeMap;
use Engine;
use super::bin::Bin;
use parking_lot::{Mutex,RwLock};
use vulkano::sampler::Sampler;
use vulkano::buffer::DeviceLocalBuffer;
use interface::itf_dual_buf::ItfDualBuffer;
use mouse;
use interface::bin::{EventInfo,HookTrigger};
use interface::text::Text;

impl_vertex!(ItfVertInfo, position, coords, color, ty);
#[derive(Clone)]
#[repr(C)]
pub(crate) struct ItfVertInfo {
	pub position: (f32, f32, f32),
	pub coords: (f32, f32),
	pub color: (f32, f32, f32, f32),
	pub ty: i32
}

impl Default for ItfVertInfo {
	fn default() -> Self {
		ItfVertInfo {
			position: (0.0, 0.0, 0.0),
			coords: (0.0, 0.0),
			color: (0.0, 0.0, 0.0, 0.0),
			ty: 0,
		}
	}
}

pub(crate) fn scale_verts(win_size: &[f32; 2], verts: &mut Vec<ItfVertInfo>) {
	for vert in verts {
		vert.position.0 += win_size[0] / -2.0;
		vert.position.0 /= win_size[0] / 2.0;
		vert.position.1 += win_size[1] / -2.0;
		vert.position.1 /= win_size[1] / 2.0;
	}
}

#[allow(dead_code)]
struct BinBufferData {
	atlas_i: usize,
	pos: usize,
	len: usize,
}

pub struct Interface {
	engine: Arc<Engine>,
	text: Arc<Text>,
	bin_i: Mutex<u64>,
	bin_map: Arc<RwLock<BTreeMap<u64, Weak<Bin>>>>,
	dual_buffer: Arc<ItfDualBuffer>,
	events_data: Mutex<EventsData>,
}

#[derive(Default)]
struct EventsData {
	focused: Option<Weak<Bin>>,
	mouse_in: BTreeMap<u64, Weak<Bin>>,
}

impl Interface {
	pub(crate) fn new(engine: Arc<Engine>) -> Arc<Self> {
		let bin_map: Arc<RwLock<BTreeMap<u64, Weak<Bin>>>> = Arc::new(RwLock::new(BTreeMap::new()));
		let bin_map_cp = bin_map.clone();
		
		engine.mouse_ref().on_any_press(Arc::new(move |_, info| {
			let bins: Vec<Arc<Bin>> = bin_map_cp.read().iter().filter_map(|(_, b)| b.upgrade()).collect();
			let mut inside = Vec::new();
			
			for bin in bins {
				if bin.mouse_inside(info.window_x, info.window_y) {
					if !bin.style_copy().pass_events.unwrap_or(false) {
						let z = bin.post_update().z_index;
						inside.push((z, bin));
					}
				}
			}
			
			inside.sort_by_key(|&(z, _)| z);
			
			if let Some((_, bin)) = inside.pop() {
				bin.call_left_mouse_press();
			}
		}));
		
		let text = Text::new(engine.clone());
		text.add_font("/usr/share/fonts/abeezee/ABeeZee-Regular.ttf", "default").unwrap();
		
		let itf = Arc::new(Interface {
			text,
			dual_buffer: ItfDualBuffer::new(engine.clone(), bin_map.clone()),
			engine: engine,
			bin_i: Mutex::new(0),
			bin_map: bin_map,
			events_data: Mutex::new(EventsData::default()),
		});
		
		/*	Hook impl Checklist
			-----------------------
			X	Mouse Press
				Mouse Hold
				Mouse Release
			X	Mouse Scroll
			X	Mouse Move
			X	Mouse Enter
			X	Mouse Leave
				Key Press
				Key Hold
				Key Release
			X	On Focus
			X	Lost Focus
		*/
		
		let itf_cp = itf.clone();
		
		itf.engine.mouse_ref().on_any_press(Arc::new(move |_, mouse::PressInfo {
			button,
			window_x,
			window_y,
			..
		}| {
			let mut events_data = itf_cp.events_data.lock();
			
			match button {
				mouse::Button::Left => if let Some(top_bin) = itf_cp.get_bin_atop(window_x, window_y) {
					if let Some(bin_wk) = events_data.focused.take() {
						if let Some(bin) = bin_wk.upgrade() {
							let hooks = bin.hooks.lock();
							
							for (_, hook) in &*hooks {
								if hook.lost_focus {
									hook.run(EventInfo {
										trigger: HookTrigger::LostFocus,
										.. EventInfo::other()
									});
								}
							}
						}
					}
					
					let hooks = top_bin.hooks.lock();
						
					for (_, hook) in &*hooks {
						if hook.on_focus {
							hook.run(EventInfo {
								trigger: HookTrigger::Focus,
								.. EventInfo::other()
							});
						}
					}
					
					events_data.focused = Some(Arc::downgrade(&top_bin));
				}, _ => ()
			}
			
			if let Some(bin_wk) = &events_data.focused {
				if let Some(bin) = bin_wk.upgrade() {
					let hooks = bin.hooks.lock();
					
					use interface::bin;
					let orig_style = bin.style_copy();
					bin.style_update(bin::BinStyle { back_color: Some(bin::Color::srgb_hex("00ffff")), .. orig_style.clone() });
					let bin_cp = bin.clone();
					::std::thread::spawn(move || { ::std::thread::sleep(::std::time::Duration::new(0, 150_000_000)); bin_cp.style_update(orig_style) });
						
					for (_, hook) in &*hooks {
						for hook_button in &hook.mouse_press {
							if *hook_button == button {
								hook.run(EventInfo {
									trigger: HookTrigger::MousePress,
									mouse_btts: vec![button.clone()],
									.. EventInfo::other()
								});
								break;
							}
						} 
					}
				}
			}
		}));
		
		let itf_cp = itf.clone();
		
		itf.engine.mouse_ref().on_scroll(Arc::new(move |_, x, y, s| {
			if let Some(top_bin) = itf_cp.get_bin_atop(x, y) {
				let mut in_bins = vec![top_bin.clone()];
				in_bins.append(&mut top_bin.ancestors());
				
				'ancestors_loop: for bin in &in_bins {
					let hooks = bin.hooks.lock();
					
					for (_, hook) in &*hooks {
						if hook.mouse_scroll {
							hook.run(EventInfo {
								trigger: HookTrigger::MouseScroll,
								scroll_amt: s,
								.. EventInfo::other()
							});
							break 'ancestors_loop;
						} 
					}
				}
			}
		}));
		
		let itf_cp = itf.clone();
		
		itf.engine.mouse_ref().on_move(Arc::new(move |_, delta_x, delta_y, x, y| {
			let mut events_data = itf_cp.events_data.lock();
			
			if let Some(top_bin) = itf_cp.get_bin_atop(x, y) {
				let mut in_bins = vec![top_bin.clone()];
				in_bins.append(&mut top_bin.ancestors());
				
				for bin in &in_bins {
					let hooks = bin.hooks.lock();
					
					if !events_data.mouse_in.contains_key(&bin.id()) {
						for (_, hook) in &*hooks {
							if hook.mouse_enter {
								hook.run(EventInfo {
									trigger: HookTrigger::MouseEnter,
									.. EventInfo::other()
								});
							}
						}
						
						events_data.mouse_in.insert(bin.id(), Arc::downgrade(bin));
					}
						
					for (_, hook) in &*hooks {
						if hook.mouse_move {
							hook.run(EventInfo {
								trigger: HookTrigger::MouseMove,
								mouse_dx: delta_x,
								mouse_dy: delta_y,
								mouse_x: x,
								mouse_y: y,
								.. EventInfo::other()
							});
						}
					}
				}
				
				let keys: Vec<u64> = events_data.mouse_in.keys().cloned().collect();
				
				for bin_id in keys {
					if !in_bins.iter().find(|b| b.id() == bin_id).is_some() {
						if let Some(bin_wk) = events_data.mouse_in.remove(&bin_id) {
							if let Some(bin) = bin_wk.upgrade() {
								let hooks = bin.hooks.lock();
								
								for (_, hook) in &*hooks {
									if hook.mouse_leave {
										hook.run(EventInfo {
											trigger: HookTrigger::MouseLeave,
											.. EventInfo::other()
										});
									} 
								}
							}
						}
					}
				}
			} else {
				let keys: Vec<u64> = events_data.mouse_in.keys().cloned().collect();
				
				for bin_id in keys {
					if let Some(bin_wk) = events_data.mouse_in.remove(&bin_id) {
						if let Some(bin) = bin_wk.upgrade() {
							let hooks = bin.hooks.lock();
							
							for (_, hook) in &*hooks {
								if hook.mouse_leave {
									hook.run(EventInfo {
										trigger: HookTrigger::MouseLeave,
										.. EventInfo::other()
									});
								} 
							}
						}
					}
				}
			}
		}));
		
		itf
	}
	
	pub(crate) fn text_ref(&self) -> &Arc<Text> {
		&self.text
	}
	
	pub fn get_bin_id_atop(&self, x: f32, y: f32) -> Option<u64> {
		let bins: Vec<Arc<Bin>> = self.bin_map.read().iter().filter_map(|(_, b)| b.upgrade()).collect();
		let mut inside = Vec::new();
		
		for bin in bins {
			if bin.mouse_inside(x, y) {
				if !bin.style_copy().pass_events.unwrap_or(false) {
					let z = bin.post_update().z_index;
					inside.push((z, bin));
				}
			}
		}
		
		inside.sort_by_key(|&(z, _)| z);
		inside.pop().map(|v| v.1.id())
	}
	
	pub fn get_bin_atop(&self, x: f32, y: f32) -> Option<Arc<Bin>> {
		let bins: Vec<Arc<Bin>> = self.bin_map.read().iter().filter_map(|(_, b)| b.upgrade()).collect();
		let mut inside = Vec::new();
		
		for bin in bins {
			if bin.mouse_inside(x, y) {
				if !bin.style_copy().pass_events.unwrap_or(false) {
					let z = bin.post_update().z_index;
					inside.push((z, bin));
				}
			}
		}
		
		inside.sort_by_key(|&(z, _)| z);
		inside.pop().map(|v| v.1)
	}
	
	fn bins(&self) -> Vec<Arc<Bin>> {
		self.bin_map.read().iter().filter_map(|(_, b)| b.upgrade()).collect()
	}
	
	pub fn new_bins(&self, amt: usize) -> Vec<Arc<Bin>> {
		let mut out = Vec::with_capacity(amt);
		let mut bin_i = self.bin_i.lock();
		let mut bin_map = self.bin_map.write();
		
		for _ in 0..amt {
			let id = *bin_i;
			*bin_i += 1;
			let bin = Bin::new(id.clone(), self.engine.clone());
			bin_map.insert(id, Arc::downgrade(&bin));
			out.push(bin);
		}
		
		out
	}
	
	pub fn new_bin(&self) -> Arc<Bin> {
		self.new_bins(1).pop().unwrap()
	}
	
	pub fn get_bin(&self, id: u64) -> Option<Arc<Bin>> {
		match self.bin_map.read().get(&id) {
			Some(some) => some.upgrade(),
			None => None
		}
	}
	
	pub fn mouse_inside(&self, mouse_x: f32, mouse_y: f32) -> bool {
		for bin in self.bins() {
			if bin.mouse_inside(mouse_x, mouse_y) {
				return true;
			}
		} false
	}
	
	pub(crate) fn draw_bufs(&self, win_size: [u32; 2], resized: bool)
		-> Vec<(
			Arc<DeviceLocalBuffer<[ItfVertInfo]>>,
			Arc<vulkano::image::traits::ImageViewAccess + Send + Sync>,
			Arc<Sampler>,
			Option<(usize, usize)>,
		)>
	{
		self.dual_buffer.draw_data(win_size, resized)
	}
}

