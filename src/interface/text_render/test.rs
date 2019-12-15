#[test]
fn render_text() {
	use super::{create_basalt_text,BstTextScript,BstTextLang};
	use crate::interface::bin::{BinStyle,PositionTy};
	use crate::Basalt;
	
	let basalt = Basalt::new(
		crate::Options::default()
			.ignore_dpi(true)
			.window_size(1000, 100)
			.title("Basalt")
	).unwrap();
	
	basalt.spawn_app_loop();
	let background = basalt.interface_ref().new_bin();
	
	background.style_update(BinStyle {
		position_t: Some(PositionTy::FromWindow),
		pos_from_t: Some(26.0),
		pos_from_b: Some(10.0),
		pos_from_l: Some(50.0),
		pos_from_r: Some(10.0),
		text: String::from("."),
		overflow_y: Some(true),
		.. background.style_copy()
	});
	
	let text = create_basalt_text(
		&basalt,
		"The quick brown fox jumps over a lazy dog.",
		BstTextScript::Default,
		BstTextLang::Default
	).unwrap();
	
	background.add_child(text.container.clone());
	background.update_children();
	
	basalt.wait_for_exit().unwrap();
}
