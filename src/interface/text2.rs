use bindings::harfbuzz::*;
use freetype::freetype::*;
use interface::interface::ItfVertInfo;
use std::ptr;
use std::ffi::CString;
use std::sync::Arc;
use Engine;
use atlas;
use interface::text::WrapTy;
use std::collections::BTreeMap;
use interface::TextAlign;

pub(crate) fn render_text<T: AsRef<str>, F: AsRef<str>>(engine: &Arc<Engine>, text: T, _family: F, size: f32, color: (f32, f32, f32, f32), wrap_ty: WrapTy, align: TextAlign) -> Result<BTreeMap<usize, Vec<ItfVertInfo>>, String> {
	unsafe {
		let size = (size).ceil() as u32;
		let mut ft_library = ptr::null_mut();

		match FT_Init_FreeType(&mut ft_library) {
			0 => (),
			e => return Err(format!("FT_Init_FreeType: error {}", e))
		}
		
		let mut ft_face = ptr::null_mut();
		let bytes = include_bytes!("ABeeZee-Regular.ttf");
		//let bytes = include_bytes!("/usr/share/fonts/TTF/Amiri-Regular.ttf");
		//let bytes = include_bytes!("/usr/share/fonts/TTF/SourceSansPro-Regular.ttf");
		
		match FT_New_Memory_Face(ft_library, bytes.as_ptr(), (bytes.len() as i32).into(), 0, &mut ft_face) {
			0 => (),
			e => return Err(format!("FT_New_Memory_Face: error {}", e))
		}
		
		match FT_Set_Pixel_Sizes(ft_face, 0, size.into()) {
			0 => (),
			e => return Err(format!("FT_Set_Pixel_Sizes: error {}", e))
		}
		
		let (start_y, line_height) = {
			let hb_font = hb_ft_font_create_referenced(ft_face);
			let hb_buffer = hb_buffer_create();
			let ctext = CString::new("Tg").unwrap();
			hb_buffer_add_utf8(hb_buffer, ctext.as_ptr(), -1, 0, -1);
			hb_buffer_guess_segment_properties (hb_buffer);
			hb_shape(hb_font, hb_buffer, ptr::null_mut(), 0);
			let len = hb_buffer_get_length(hb_buffer) as usize;
			let info = Vec::from_raw_parts(hb_buffer_get_glyph_infos(hb_buffer, ptr::null_mut()), len, len);
			let pos = Vec::from_raw_parts(hb_buffer_get_glyph_positions(hb_buffer, ptr::null_mut()), len, len);
			assert!(len == 2);
			
			let t_metrics = match FT_Load_Glyph(ft_face, info[0].codepoint.into(), FT_LOAD_DEFAULT as i32) {
				0 => (*(*ft_face).glyph).metrics.clone(),
				e => return Err(format!("FT_Load_Glyph: error {}", e))
			};
			
			let g_metrics = match FT_Load_Glyph(ft_face, info[1].codepoint.into(), FT_LOAD_DEFAULT as i32) {
				0 => (*(*ft_face).glyph).metrics.clone(),
				e => return Err(format!("FT_Load_Glyph: error {}", e))
			};
			
			let top = (pos[0].y_offset as f32 / 64.0) - (t_metrics.horiBearingY as f32 / 64.0);
			let baseline = top + (t_metrics.height as f32 / 64.0);
			let bottom = (pos[1].y_offset as f32 - g_metrics.horiBearingY as f32 + g_metrics.height as f32) / 64.0;
			(-top, (bottom-top) + (size as f32 / 6.0).ceil())
		};
		
		let hb_font = hb_ft_font_create_referenced(ft_face);
		let hb_buffer = hb_buffer_create();
		let clines: Vec<CString> = text.as_ref().lines().map(|v| CString::new(v).unwrap()).collect();
		
		let mut current_x = 0.0;
		let mut current_y = start_y;
		let mut vert_map = BTreeMap::new();
		let mut lines = Vec::new();
		lines.push(Vec::new());
		lines.last_mut().unwrap().push(Vec::new());
		
		for ctext in clines {
			//hb_buffer_set_flags(hb_buffer, HB_BUFFER_FLAG_PRESERVE_DEFAULT_IGNORABLES);
			hb_buffer_add_utf8(hb_buffer, ctext.as_ptr(), -1, 0, -1);
			hb_buffer_guess_segment_properties(hb_buffer);
			hb_shape(hb_font, hb_buffer, ptr::null_mut(), 0);
			
			let len = hb_buffer_get_length(hb_buffer) as usize;
			let info = ::std::slice::from_raw_parts(hb_buffer_get_glyph_infos(hb_buffer, ptr::null_mut()), len);
			let pos = ::std::slice::from_raw_parts(hb_buffer_get_glyph_positions(hb_buffer, ptr::null_mut()), len);
			
			for i in 0..len {
				match FT_Load_Glyph(ft_face, info[i].codepoint.into(), FT_LOAD_DEFAULT as i32) {
					0 => (),
					e => return Err(format!("FT_Load_Glyph: error {}", e))
				}
				
				let mut glyph = *(*ft_face).glyph;
				
				match FT_Render_Glyph(&mut glyph, FT_Render_Mode::FT_RENDER_MODE_NORMAL) {
					0 => (),
					e => return Err(format!("FT_Render_Glyph: error {}", e))
				}
			
				let bitmap = glyph.bitmap;
				let w = bitmap.width as usize;
				let h = bitmap.rows as usize;
				
				if w == 0 || h == 0 {
					lines.last_mut().unwrap().push(Vec::new());
				} else {
					let mut image_data = Vec::with_capacity(w * h * 4);
				
					for i in 0..((w*h) as isize) {
						image_data.push(0);
						image_data.push(0);
						image_data.push(0);
						image_data.push(*bitmap.buffer.offset(i));
					}
					
					let coords = match engine.atlas_ref().load_raw_with_key(
						&atlas::ImageKey::Glyph(size, info[i].codepoint as u64),
						image_data, w as u32, h as u32
					) {
						Ok(ok) => ok,
						Err(e) => return Err(format!("Atlas::load_raw_with_key: Er(pos[i].y_offset as f32 / 64.0) - (glyph.metrics.horiBearingY as f32 / 64.0)ror {}", e))
					};
				
					let tl = (
						current_x + (pos[i].x_offset as f32 / 64.0) + (glyph.metrics.horiBearingX as f32 / 64.0),
						current_y + (pos[i].y_offset as f32 / 64.0) - (glyph.metrics.horiBearingY as f32 / 64.0),
						0.0
					);
					
					let tr = (tl.0 + (glyph.metrics.width as f32 / 64.0), tl.1, 0.0);
					let bl = (tl.0, tl.1 + (glyph.metrics.height as f32 / 64.0), 0.0);
					let br = (tr.0, bl.1, 0.0);
					
					let ctl = coords.f32_top_left();
					let ctr = coords.f32_top_right();
					let cbl = coords.f32_bottom_left();
					let cbr = coords.f32_bottom_right();
					
					let mut verts = Vec::with_capacity(6);
					verts.push(ItfVertInfo { position: tr, coords: ctr, color: color, ty: 1 });
					verts.push(ItfVertInfo { position: tl, coords: ctl, color: color, ty: 1 });
					verts.push(ItfVertInfo { position: bl, coords: cbl, color: color, ty: 1 });
					verts.push(ItfVertInfo { position: tr, coords: ctr, color: color, ty: 1 });
					verts.push(ItfVertInfo { position: bl, coords: cbl, color: color, ty: 1 });
					verts.push(ItfVertInfo { position: br, coords: cbr, color: color, ty: 1 });
					lines.last_mut().unwrap().last_mut().unwrap().push((coords.atlas_i, verts));
				}
				
				current_x += pos[i].x_advance as f32 / 64.0;
				current_y += pos[i].y_advance as f32 / 64.0;
			}
			
			lines.push(Vec::new());
			lines.last_mut().unwrap().push(Vec::new());
			hb_buffer_clear_contents(hb_buffer);
		}
		
		match wrap_ty {
			WrapTy::ShiftX(w) => {
			
			},
			WrapTy::ShiftY(_) => unimplemented!(),
			WrapTy::Normal(w, h) => {
				let mut cur_line = Vec::new();
				cur_line.push(Vec::new());
				let mut wrapped_lines = Vec::new();
				
				for line in lines {
					let mut start = 0.0;
					let mut end = 0.0;
					let mut last_max_x = 0.0;
					let mut w_len = line.len();
				
					for (w_i, word) in line.into_iter().enumerate() {
						if word.is_empty() {
							continue;
						}
						
						let mut min_x = None;
						let mut max_x = None;
						
						for (_, verts) in &word {
							for vert in verts {
								if max_x.is_none() || vert.position.0 > *max_x.as_ref().unwrap() {
									max_x = Some(vert.position.0);
								}
								
								if min_x.is_none() || vert.position.0 < *min_x.as_ref().unwrap() {
									min_x = Some(vert.position.0);
								}	
							}
						}
						
						let min_x = min_x.unwrap();
						let max_x = max_x.unwrap();
						
						if cur_line.is_empty() {
							start = min_x;
						}
						
						if max_x - start > w && w_i != 0 {
							wrapped_lines.push((cur_line, start, last_max_x));
							cur_line = Vec::new();
							start = min_x;
						} else {
							last_max_x = max_x;
						}
						
						cur_line.push(word);
						
						if w_i == w_len-1 {
							end = max_x;
						}
					}
					
					wrapped_lines.push((cur_line, start, end));
					cur_line = Vec::new();
					start = 0.0;
					end = 0.0;
					last_max_x = 0.0;
				}
				
				for (line_i, (words, start, end)) in wrapped_lines.into_iter().enumerate() {
					for word in words {
						let lwidth = end - start;
						let xoffset = match align {
							TextAlign::Left => -start,
							TextAlign::Center => ((w - lwidth) / 2.0) - start,
							TextAlign::Right => (w - lwidth) - start,
						};
						let yoffset = line_i as f32 * line_height;
					
						for (atlas_i, mut verts) in word {
							for vert in &mut verts {
								vert.position.0 += xoffset;
								vert.position.1 += yoffset;
							}
						
							vert_map.entry(atlas_i).or_insert(Vec::new()).append(&mut verts);
						}
					}
				}
			},
			WrapTy::None => {
				for words in lines {
					for word in words {
						for (atlas_i, mut verts) in word {
							vert_map.entry(atlas_i).or_insert(Vec::new()).append(&mut verts);
						}
					}
				}
			}
		}
		
		Ok(vert_map)
	}
}
