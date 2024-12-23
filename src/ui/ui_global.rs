//! @Author: DengLibin
//! @Date: Create in 2023-11-02 17:05:33
//! @Description: 

///全局加载支持中文的字体
pub fn load_global_font(ctx: &egui::Context){
    let mut fonts = eframe::egui::FontDefinitions::default();

    // Install my own font (maybe supporting non-latin characters):
    fonts.font_data.insert("msyh".to_owned(),
                           eframe::egui::FontData::from_static(include_bytes!(r"../../fonts/SourceHanSansCN-Normal.otf"))); // .ttf and .otf supported

    // Put my font first (highest priority):
    fonts.families.get_mut(&eframe::egui::FontFamily::Proportional).unwrap()
        .insert(0, "msyh".to_owned());

    // Put my font as last fallback for monospace:
    fonts.families.get_mut(&eframe::egui::FontFamily::Monospace).unwrap()
        .push("msyh".to_owned());

    // let mut ctx = egui::CtxRef::default();
    ctx.set_fonts(fonts);
}
