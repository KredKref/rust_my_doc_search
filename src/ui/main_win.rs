//! @Author: DengLibin
//! @Date: Create in 2024-11-21 10:24:02
//! @Description:

use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

use eframe::egui;
use egui::{
    pos2, text::LayoutJob, vec2, Align, Button, Color32, CursorIcon, FontId, Frame, IconData,
    Image, ImageButton, Label, Response, RichText, ScrollArea, Stroke, TextFormat, TextWrapMode,
    Ui, ViewportBuilder, WidgetText,
};

use indexmap::IndexMap;
use log::{error, info};
use rfd::FileDialog;
use rust_common::file_util;
use sqlx::SqlitePool;
use tantivy::Index;
use tokio::{sync::mpsc, time::sleep};

use crate::{
    app::{self, get_data_dir, init_log},
    component::cmd::open_folder_and_select_file,
    dao::{
        files_dao::{self, IndexFile},
        index_dir_dao, sqlite_dao,
    },
    file_scanner::{
        file_extractor::{self, extract_file},
        file_text_extractor::{self, FileText},
    },
    sys::global::{GlobalError, GlobalResult},
    tantivy::{tantivy_jieba, tantivy_search},
    ui::ui_global::load_global_font,
    util::text_utils,
};
//窗口宽高
const WIN_WIDTH: f32 = 1050.0;
const WIN_HEIGHT: f32 = 720.0;

const LETF_WIDTH: f32 = 280.0;

/// @Author: DengLibin
/// @Date: Create in 2024-12-16 09:48:39
/// @Description:
pub fn get_icon() -> IconData {
    // 加载图标图像
    let bytes = include_bytes!("../../imgs/ico.ico");
    let img: image::DynamicImage = image::load_from_memory(bytes).expect("Failed to load icon.png");

    // 将图像转换为RGBA格式
    let rgba_image = img.to_rgba8();

    // 获取图像的宽度和高度
    let (width, height) = rgba_image.dimensions();

    // 将RGBA像素数据转换为Vec<u8>
    let rgba: Vec<u8> = rgba_image.into_raw();

    let icon: IconData = IconData {
        rgba: rgba,
        width: width,
        height: height,
    };
    icon
}

/// @Author: DengLibin
/// @Date: Create in 2024-11-21 10:30:04
/// @Description: 展示窗口
pub fn show_win(runtime: tokio::runtime::Runtime) -> Result<(), eframe::Error> {
    let viewport = ViewportBuilder::default()
        .with_resizable(true)
        .with_active(true)
        .with_app_id("MySearch")
        .with_icon(get_icon())
        .with_inner_size((WIN_WIDTH, WIN_HEIGHT));

    let options = eframe::NativeOptions {
        viewport,

        ..Default::default()
    };
    eframe::run_native(
        "MySearch",
        options,
        Box::new(|cc| {
            // This gives us image support: 支持图片
            egui_extras::install_image_loaders(&cc.egui_ctx);
            //加载全局字体
            load_global_font(&cc.egui_ctx);

            let app = Box::<MyApp>::new(MyApp::new(runtime));

            Ok(app)
        }),
    )
}

struct MyFile {
    path: String,    //路径
    name: String,    //文件名
    is_file: bool,   //是否文件
    content: String, //文件内容
}

struct MyApp {
    name: String,
    age: u32,
    search_text: String,                                  //搜索文本
    tokenize: Vec<String>,                                //搜索文本分词
    files: Vec<MyFile>,                                   // 文件列表
    forder_img: Image<'static>,                           //文件夹图标
    file_img: Image<'static>,                             //文件图标
    close_img: Image<'static>,                            //关闭图标
    index_dirs: Vec<index_dir_dao::IndexDir>,             //索引的文件夹
    current_index: i32,                                   //当前选中索引
    current_del_index: i32,                               //当前删除的
    sqlite_pool: Arc<SqlitePool>,                         //sqlite连接池
    runtime: tokio::runtime::Runtime,                     // 异步运行时
    tip_content: Vec<String>,                             //弹窗提示信息
    show_tip: bool,                                       //是否显示弹窗
    msg: String,                                          //消息
    msg_sender: Arc<std::sync::mpsc::Sender<String>>,     //消息发送者
    msg_receiver: Arc<std::sync::mpsc::Receiver<String>>, //消息接收者
    tantivy_index: Arc<RwLock<Index>>,                    //索引
    scaning_count: i32,                                   //扫描中文件夹数量
    file_count: i64,                                      //文件总数
}

impl eframe::App for MyApp {
    /// @Author: DengLibin
    /// @Date: Create in 2023-11-03 08:47:00
    /// @Description: update 函数 会循环调用
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.my_ui(ctx, frame);
    }
}

impl MyApp {
    fn new(runtime: tokio::runtime::Runtime) -> Self {
        let data_dir: String = app::get_data_dir();
        let db_path = format!("sqlite:{}/mysearch.db", data_dir);
        let sqlite_pool =
            runtime.block_on(async { sqlite_dao::get_conn_pool(&db_path).await.unwrap() });

        let (tx, rx) = std::sync::mpsc::channel::<String>();
        let arc_tx: Arc<std::sync::mpsc::Sender<String>> = Arc::new(tx);
        let arc_rx: Arc<std::sync::mpsc::Receiver<String>> = Arc::new(rx);

        //索引
        let index =
            tantivy_search::open_index(format!("{}/indices", get_data_dir()).as_str()).unwrap();
        let arc_index = Arc::new(RwLock::new(index));

        let mut my_app: MyApp = Self {
            name: "Arthur".to_owned(),
            age: 42,
            search_text: "".into(),
            tokenize: vec![],
            files: vec![],
            forder_img: Image::new(egui::include_image!("../../imgs/forder.png"))
                .fit_to_original_size(0.8),

            file_img: Image::new(egui::include_image!("../../imgs/file.png"))
                .fit_to_original_size(0.8),

            close_img: Image::new(egui::include_image!("../../imgs/close.png"))
                .fit_to_original_size(0.8),

            index_dirs: vec![],
            current_index: -1,
            current_del_index: -1,
            sqlite_pool: Arc::new(sqlite_pool),
            runtime: runtime,
            tip_content: vec![],
            show_tip: false,
            msg: "".into(),
            msg_sender: arc_tx,
            msg_receiver: arc_rx,
            tantivy_index: arc_index,
            scaning_count: 0,
            file_count: 0_i64,
        };
        my_app.init().unwrap();
        my_app
    }

    /**
     *界面
     */
    fn my_ui(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.tip_ui(ctx);
        //菜单
        self.menu_ui(ctx);
        //界面
        self.body_ui(ctx);
    }

    /// @Author: DengLibin
    /// @Date: Create in 2024-12-16 14:00:56
    /// @Description: 菜单
    fn menu_ui(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("帮助", |ui| {
                    if ui.button("关于").clicked() {
                        self.show_tips(&mut vec![
                            "使用: 添加文件夹，自动创建索引，完成后即可搜索".to_string(),
                            "版本: v1.0.0".to_string(),
                            "作者: Mr.Deng".to_string(),
                            "邮箱: 910807415@qq.com".to_string(),
                        ]);
                    }
                });
            });
        });
    }
    /// @Author: DengLibin
    /// @Date: Create in 2024-12-16 14:04:42
    /// @Description: 界面
    fn body_ui(&mut self, ctx: &egui::Context) {
        // 获取宽度
        let window_width = ctx.screen_rect().width();
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                //分配高度
                ui.allocate_space(vec2(0.0, ui.ctx().screen_rect().height() - 50.0));
                //第一列
                ScrollArea::vertical().animated(true).show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.set_min_width(LETF_WIDTH);
                        ui.set_max_width(LETF_WIDTH);
                        self.index_dir_ui(ui);
                    });
                });

                // 第二列内容
                ui.vertical(|ui| {
                    ui.set_min_width(window_width - LETF_WIDTH - 30.0);
                    ui.set_max_width(window_width - LETF_WIDTH - 30.0);
                    self.serarch_ui(ui);
                });
                //接收消息
                self.reaceive_msg();
            });

            //底部显示消息
            self.wrap_label_text(ui, self.msg.as_str());
        });
    }
    /// @Author: DengLibin
    /// @Date: Create in 2024-12-06 16:08:13
    /// @Description: 接收消息
    fn reaceive_msg(&mut self) {
        let mut process_msg_count = 0;
        while process_msg_count < 100 {
            let m = self.msg_receiver.try_recv();
            if let Ok(msg) = m {
                if msg.contains("创建索引完成") {
                    self.scaning_count -= 1;
                    self.count_files();
                }
                self.msg = msg;
                
            } else {
                break;
            }
            process_msg_count += 1;
        }
    }

    /// @Author: DengLibin
    /// @Date: Create in 2024-12-06 11:54:19
    /// @Description: 提示信息
    fn tip_ui(&mut self, ctx: &egui::Context) {
        if !self.show_tip {
            return;
        }
        let width: f32 = 300.0;
        let height = 300.0;
        let tip_w: egui::Window<'_> = egui::Window::new("提示")
            .default_size((width, height))
            .default_pos((
                WIN_WIDTH / 2.0 - width / 2.0,
                WIN_HEIGHT / 2.0 - height / 2.0,
            ))
            .collapsible(false)
            .open(&mut self.show_tip)
            .resizable(false);
        tip_w.show(ctx, |ui| {
            for tip_msg in self.tip_content.iter() {
                ui.label(tip_msg);
                ui.add_space(5.0);
            }
        });
    }

    /// @Author: DengLibin
    /// @Date: Create in 2024-11-29 18:28:36
    /// @Description: 文件夹界面
    fn index_dir_ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            self.wrap_label_text(ui, format!("文件总数:{}", self.file_count).as_str());
            ui.add_space(20.0);

            let btn_res = ui.add_enabled(self.scaning_count == 0, Button::new("重新扫描"));

            if btn_res.clicked() {
                let dirs: Vec<(String, i64)> = self
                    .index_dirs
                    .iter()
                    .map(|dir| (dir.path.clone(), dir.id))
                    .collect();
                for ele in dirs {
                    self.scan_files(ele.0, ele.1);
                }
            }
            ui.add_space(20.0);
            //按钮
            if ui.button("添加文件夹").clicked() {
                // 弹出文件夹选择对话框
                if let Some(folder) = FileDialog::new().pick_folder() {
                    let selected_folder = folder.display().to_string();
                    //self.index_dirs.push(selected_folder.clone());
                    let r = self.add_index_dir(selected_folder.clone());
                    if let Ok(index_idr_id) = r {
                        self.list_index_dirs().unwrap();
                        //扫描该文件夹下文件
                        self.scan_files(selected_folder, index_idr_id);
                    } else if let Err(e) = r {
                        self.show_tip(&e.msg);
                    }
                }
            }
        });

        if self.current_del_index >= 0 {
            self.index_dirs.remove(self.current_del_index as usize);
            self.current_del_index = -1; //重置
        }

        for i in 0..self.index_dirs.len() as i32 {
            ui.add_space(5.0);
            let text = format!("{}", self.index_dirs[i as usize].path);

            let mut rich_text: RichText = text.as_str().into();
            rich_text = rich_text.size(15.0).color(Color32::BLACK);

            //水平布局
            ui.horizontal(|ui| {
                ui.set_max_width(LETF_WIDTH - 30.0);
                let mut r: Option<Response> = Option::None;
                if i == self.current_index {
                    Frame::none()
                        .fill(Color32::from_rgb(220, 220, 220)) // 设置背景颜色
                        .rounding(5.0) // 圆角
                        .show(ui, |ui| {
                            // let res = ui.label(rich_text);
                            let res = self.wrap_label_text(ui, rich_text);
                            r = Some(res);
                        });
                } else {
                    // let res = ui.label(rich_text);
                    let res = self.wrap_label_text(ui, rich_text);
                    r = Some(res);
                }

                if let Some(_res) = r {
                    /*
                                        //光标样式
                                        if res.hovered() {
                                            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                                        }
                                        //点击效果
                                        if res.clicked() {
                                            self.current_index = i;
                                            self.list_files();
                                        }
                    */
                }

                //居右
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    if self.scaning_count == 0 {
                        //关闭图标
                        let close_btn = ImageButton::new(self.close_img.clone())
                            // .rounding(45.0)
                            .frame(false); //去掉默认边框和背景
                        let close_res = ui.add(close_btn);
                        //光标样式
                        if close_res.hovered() {
                            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                        }
                        //点击效果
                        if close_res.clicked() {
                            self.rm_index_dir(i, self.index_dirs[i as usize].id);
                        }
                    } else {
                        let mut rich_text: RichText = "...".into();
                        rich_text = rich_text.size(15.0).color(Color32::BLACK);
                        ui.label(rich_text);
                    }
                });
            });
            /*
                       let y = ui.cursor().min.y;

                       ui.painter().line_segment(
                           [pos2(10.0, y), pos2(LETF_WIDTH - 10.0, y)], // 起点和终点
                           Stroke::new(1.0, Color32::GREEN),            // 线条宽度和颜色
                       );
            */
        }
    }

    /// @Author: DengLibin
    /// @Date: Create in 2024-11-29 18:28:12
    /// @Description: 搜索界面
    fn serarch_ui(&mut self, ui: &mut Ui) {
        //搜索框
        self.input_box_ui(ui);
        ui.add_space(25.0);

        //结果列表
        ScrollArea::vertical().animated(true).show(ui, |ui| {
            self.files_ui(ui).unwrap();
        });
    }

    /// @Author: DengLibin
    /// @Date: Create in 2024-11-29 11:53:37
    /// @Description: 搜索
    fn input_box_ui(&mut self, ui: &mut Ui) {
        //元素大小
        let search_size: f32 = 25.0;
        let win_width = ui.ctx().screen_rect().width();
        //水平布局
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.search_text)
                    .font(FontId::proportional(search_size))
                    .min_size((win_width - LETF_WIDTH - 120.0, search_size).into())
                    .desired_rows(1)
                    .hint_text("请输入"),
            );

            // 使用 RichText 设置按钮字体大小
            let button_text = RichText::new("搜索").size(search_size);
            let search_btn = egui::Button::new(button_text)
                .min_size((80.0, search_size).into())
                .stroke(Stroke {
                    width: 2.0,           // 边框宽度
                    color: Color32::GRAY, // 边框颜色
                });
            let btn_res = ui.add(search_btn);
            //搜索按钮点击
            if btn_res.clicked() {
                self.search_res();
            }
            //键盘事件
            if ui.ctx().input(|i| i.key_pressed(egui::Key::Enter)) {
                self.search_res();
            }
        });
    }

    /// @Author: DengLibin
    /// @Date: Create in 2024-11-29 16:03:40
    /// @Description: 文件列表
    fn files_ui(&mut self, ui: &mut Ui) -> Result<(), Box<dyn std::error::Error>> {
        //点击的文件
        let mut clicked_file: Option<&MyFile> = None;
        for my_file in self.files.iter() {
            //文件名称

            // let mut rich_text_file_name: RichText = my_file.name.as_str().into();
            // rich_text_file_name = rich_text_file_name.size(20.0).color(Color32::BLACK);

            //复制图标
            let forder_ico = self.forder_img.clone();
            let file_ico = self.file_img.clone();
            ui.add_space(15.0);
            ui.horizontal(|ui| {
                //图标
                ui.vertical(|ui| {
                    if my_file.is_file {
                        ui.add_space(3.0);
                        ui.add(file_ico);
                    } else {
                        ui.add_space(6.0);
                        ui.add(forder_ico);
                    }
                });
                if !my_file.is_file {
                    ui.add_space(5.0);
                }
                //文件名
                // let res = self.wrap_label_text(ui, rich_text_file_name);
                let res = self.highlight_text_fragment(
                    ui,
                    my_file.name.as_str(),
                    &self.tokenize,
                    16.0,
                    Color32::BLACK,
                );

                //光标样式
                if res.hovered() {
                    ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                    ui.painter()
                        .rect_filled(res.rect, 0.0, Color32::from_black_alpha(50));
                }
                //点击效果
                if res.clicked() {
                    clicked_file = Some(&my_file);
                }
            });

            //文件内容
            self.highlight_text_fragment(
                ui,
                my_file.content.as_str(),
                &self.tokenize,
                12.0,
                Color32::GRAY,
            );
            //文件路径
            ui.add_space(10.0);
            ui.label(my_file.path.replace(".out/", "/"));

            let y = ui.cursor().min.y;

            ui.painter().line_segment(
                [
                    pos2(LETF_WIDTH + 30.0, y),
                    pos2(ui.ctx().screen_rect().width() - 10.0, y),
                ], // 起点和终点
                Stroke::new(0.5, Color32::GRAY), // 线条宽度和颜色
            );
        }
        if let Some(my_file) = clicked_file {
            if file_util::exist(&my_file.path) {
                //定位文件
                open_folder_and_select_file(&my_file.path);
            }
        }
        Ok(())
    }

    /// @Author: DengLibin
    /// @Date: Create in 2024-12-03 11:19:57
    /// @Description: 包裹label
    fn wrap_label_text(&self, ui: &mut Ui, text: impl Into<WidgetText>) -> Response {
        ui.add(
            Label::new(text)
                .wrap()
                .wrap_mode(TextWrapMode::Wrap)
                .halign(Align::Min),
        )
    }

    /// @Author: DengLibin
    /// @Date: Create in 2024-12-19 15:55:42
    /// @Description:
    fn highlight_text_fragment(
        &self,
        ui: &mut Ui,
        text: &str,
        keywords: &Vec<String>,
        font_size: f32,
        normarl_color: Color32,
    ) -> Response {
        //关键词位置
        let poses = text_utils::keywords_pos(text, keywords);

        let mut job = LayoutJob::default();
        let mut start = 0;
        for pos in poses {
            //关键词前面的文字
            if start < pos.0 {
                let pre_text = &text[start..pos.0];
                job.append(
                    pre_text,
                    0.0,
                    TextFormat {
                        font_id: FontId::proportional(font_size),
                        color: normarl_color,
                        ..Default::default()
                    },
                );
            }
            //关键词
            let key_text = &text[pos.0..pos.1 + 1];
            job.append(
                key_text,
                0.0,
                TextFormat {
                    font_id: FontId::proportional(font_size),
                    color: Color32::RED,
                    ..Default::default()
                },
            );
            start = pos.1 + 1;
        }

        //剩余的文字
        let left_text = &text[start..];
        job.append(
            left_text,
            0.0,
            TextFormat {
                font_id: FontId::proportional(font_size),
                color: normarl_color,
                ..Default::default()
            },
        );

        return ui.label(job);
    }

    /**
     * 示例
     */
    #[allow(dead_code)]
    fn update_demo(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            //添加布局
            let mut rich_text: RichText = "第一个示例".into();
            rich_text = rich_text.size(78_f32).color(Color32::RED);
            ui.heading(rich_text); //头部
                                   //水平布局
            ui.horizontal(|ui| {
                let name_label = ui.label("您的名字: "); //label
                                                         //单行文本框
                ui.text_edit_singleline(&mut self.name)
                    .labelled_by(name_label.id);
            });

            //添加滑块
            ui.add(egui::Slider::new(&mut self.age, 0..=120).text("年龄"));
            if ui.button("增加一年").clicked() {
                self.age += 1; //年龄增加
            }
            //添加lable
            ui.label(format!("Hello '{}', 年龄 {}", self.name, self.age));
        });
    }

    /// @Author: DengLibin
    /// @Date: Create in 2024-11-29 16:12:18
    /// @Description: 读取文件夹下文件

    #[allow(dead_code)]
    fn list_files(&mut self) {
        self.files.clear();
        let index_dir_id = self.index_dirs[self.current_index as usize].id;
        let r = self.runtime.block_on(async {
            let files = files_dao::select_by_index_dir_id(&self.sqlite_pool, index_dir_id).await;
            files
        });
        match r {
            Ok(files) => {
                for index_file in files {
                    let my_file = MyFile {
                        path: index_file.path,
                        name: index_file.name,
                        is_file: true,
                        content: "".into(),
                    };

                    self.files.push(my_file);
                }
            }
            Err(e) => {
                error!("查询文件异常:{}", e);
            }
        }
    }

    /// @Author: DengLibin
    /// @Date: Create in 2024-12-03 17:11:29
    /// @Description: 初始化相关
    fn init(&mut self) -> GlobalResult<()> {
        self.runtime.block_on(async {
            init_log().await?;
            index_dir_dao::create_index_dir_table(&self.sqlite_pool).await?;
            files_dao::create_index_file_table(&self.sqlite_pool).await
        })?;
        self.count_files();
        return self.list_index_dirs();
    }

    /// @Author: DengLibin
    /// @Date: Create in 2024-12-03 17:08:26
    /// @Description: 获取索引的文件夹列表
    fn list_index_dirs(&mut self) -> GlobalResult<()> {
        let r = self
            .runtime
            .block_on(async { index_dir_dao::select_all(&self.sqlite_pool).await });

        let index_dirs: Vec<index_dir_dao::IndexDir> = r?;
        self.index_dirs = index_dirs;
        Ok(())
    }

    /// @Author: DengLibin
    /// @Date: Create in 2024-12-03 17:13:52
    /// @Description: 添加索引文件夹
    fn add_index_dir(&mut self, dir: String) -> GlobalResult<i64> {
        for index_dir in self.index_dirs.iter() {
            if dir.contains(index_dir.path.as_str()) {
                return Err(GlobalError {
                    msg: "所在文件夹已索引".into(),
                });
            }
        }

        self.runtime
            .block_on(async { index_dir_dao::add_index_dir(&self.sqlite_pool, dir).await })
    }
    /// @Author: DengLibin
    /// @Date: Create in 2024-12-06 10:00:18
    /// @Description: 删除索引文件夹
    fn rm_index_dir(&mut self, index: i32, id: i64) {
        self.current_del_index = index as i32;
        let arc_tantivy_index = self.tantivy_index.clone();
        let _ = self.runtime.block_on(async {
            let r = index_dir_dao::delete(&self.sqlite_pool, id).await;
            if let Ok(()) = r {
                {
                    let mut index = arc_tantivy_index.write().unwrap();
                    let r = tantivy_search::delete_by_index_dir_id(&mut index, id);
                    if let Ok(()) = r {
                        info!("删除索引成功");
                    } else {
                        error!("删除索引失败:{}", r.unwrap_err());
                    }
                }
                files_dao::delete_by_index_dir(&self.sqlite_pool, id).await
            } else {
                r
            }
        });
        self.count_files();
        self.files.clear();
    }
    /// @Author: DengLibin
    /// @Date: Create in 2024-12-06 11:55:48
    /// @Description: 显示提示窗口
    fn show_tip(&mut self, content: &str) {
        self.tip_content.clear();
        self.tip_content.push(content.into());
        self.show_tip = true;
    }
    fn show_tips(&mut self, contents: &mut Vec<String>) {
        self.tip_content.clear();
        self.tip_content.append(contents);
        self.show_tip = true;
    }

    /// @Author: DengLibin
    /// @Date: Create in 2024-12-19 09:45:23
    /// @Description: 搜索
    fn search_res(&mut self) {
        let search_res = {
            let index = self.tantivy_index.read().unwrap();

            //搜索
            //分词
            let mut keywords = tantivy_jieba::tokenize(&self.search_text);

            self.tokenize.clear();
            self.tokenize.append(&mut keywords);

            let query_str = self.tokenize.join(" AND ");
            // println!("搜索:{}", query_str);
            //从长到短排序
            self.tokenize
                .sort_by(|item1, item2| item2.len().cmp(&item1.len()));

            let search_res = tantivy_search::search_doc(&index, query_str.as_str(), 1, 500);

            search_res
        };

        self.files.clear();
        if let Ok(docs) = search_res {
            // println!("搜索结果数量:{}", docs.len());
            let mut files = docs
                .into_iter()
                .map(|doc| MyFile {
                    is_file: true,
                    name: doc.file_name,
                    path: doc.file_path,
                    content: doc.file_content,
                })
                .collect::<Vec<MyFile>>();
            //分组
            files = Self::group_by_file_path(files);
            self.files.append(&mut files);
        } else {
            error!("搜索异常:{}", search_res.unwrap_err());
        }
        self.get_text_snippets();
    }

    /// @Author: DengLibin
    /// @Date: Create in 2024-12-06 14:36:55
    /// @Description: 扫描文件
    fn scan_files(&mut self, dir: String, index_dir_id: i64) {
        let dir_c = dir.clone();
        let dir_c2 = dir.clone();
        let dir_c3 = dir.clone();
        let msg_sender1: Arc<std::sync::mpsc::Sender<String>> = self.msg_sender.clone();
        let msg_sender2: Arc<std::sync::mpsc::Sender<String>> = self.msg_sender.clone();
        let arc_tantivy_index = self.tantivy_index.clone();
        let arc_sqlite_pool = self.sqlite_pool.clone();
        self.scaning_count += 1;
        self.runtime.spawn(async move {
            let (tx, mut rx) = mpsc::channel::<String>(1000); // 创建通道，设置缓冲区大小
            let (text_sender, mut text_receiver) = mpsc::channel::<FileText>(1); // 创建通道，设置缓冲区大小
            let text_sender_arc = Arc::new(text_sender);

            //接收提取的文件文本内容
            //接收文件内容
            tokio::spawn(async move {
                let mut count = 0;
                let mut all_docs: Vec<tantivy_search::IndexDocument> = vec![];
                while let Some(file_text) = text_receiver.recv().await {
                    let _r = msg_sender2.send(format!(
                        "创建索引:{},已完成数量:{}",
                        file_text.file_path, count
                    ));
                    if !file_text.success {
                        error!("提取内容错误:{}:{}", file_text.file_path, file_text.err);
                        continue;
                    }
                    //添加到索引
                    let file_path = file_text.file_path;
                    let content = file_text.text;

                    let mut docs: Vec<tantivy_search::IndexDocument> =
                        tantivy_search::IndexDocument::split_to_list(
                            file_path,
                            content,
                            index_dir_id,
                        );
                    all_docs.append(&mut docs);
                    count += 1;

                    if all_docs.len() > 1000 {
                        let mut index = arc_tantivy_index.write().unwrap();
                        let r = tantivy_search::insert_doc_list(&mut index, &all_docs);
                        if let Err(e) = r {
                            error!("添加索引文档异常:{}", e);
                        } else {
                            info!("添加索引文档成功,数量：{}", all_docs.len());
                        }
                        all_docs.clear();
                    }
                }
                if !all_docs.is_empty() {
                    let mut index = arc_tantivy_index.write().unwrap();
                    let r = tantivy_search::insert_doc_list(&mut index, &all_docs);
                    if let Err(e) = r {
                        error!("添加索引文档异常:{}", e);
                    } else {
                        info!("添加索引文档成功,数量：{}", all_docs.len());
                    }
                    all_docs.clear();
                }

                let _r = msg_sender2.send(format!("{}:创建索引完成,文件数量:{}", dir_c3, count));
                info!("提取文件内容完成:{}", count);

                sleep(Duration::from_secs(3)).await;
                //删除解压目录
                tokio::task::spawn_blocking(|| {
                    info!("删除解压目录:{}", dir_c2);
                    let r = file_extractor::remove_out_dir_all(dir_c2);
                    if let Err(e) = r {
                        error!("删除解压目录异常:{}", e);
                    }
                });
            });

            //接收文件
            tokio::spawn(async move {
                //接收提取的文件
                let mut count = 0;
                //已存在的
                let exits_files = files_dao::file_paths(arc_sqlite_pool.as_ref(), index_dir_id)
                    .await
                    .unwrap();

                //扫描到的文件
                let mut index_files: Vec<IndexFile> = vec![];
                while let Some(file_path) = rx.recv().await {
                    let file_path = file_path.replace("\\", "/");
                    let msg =
                        format!("扫描文件:{}(已扫描数量:{})", file_path, count).replace("\\", "/");
                    let r = msg_sender1.send(msg);
                    count += 1;
                    if let Err(e) = r {
                        error!("发送消息失败{}:", e);
                    }
                    //判断是否已存在
                    if exits_files.contains(&file_path) {
                        let _ = msg_sender1.send(format!("跳过:{}", file_path));
                        continue;
                    }
                    let index_file = IndexFile::new(file_path.clone(), index_dir_id);
                    index_files.push(index_file);

                    //提取文件内容
                    file_text_extractor::spawn_extract_text(file_path, text_sender_arc.clone())
                        .await;
                }
                if !index_files.is_empty() {
                    let r = files_dao::insert_batch(arc_sqlite_pool.as_ref(), index_files).await;
                    if let Ok(()) = r {
                        info!("文件存储完成")
                    }
                }
                info!("收到文件完成:{}", count);
                let _r = msg_sender1.send(format!("文件扫描完成:{},数量:{}", dir_c, count));
            });

            //扫描文件夹
            tokio::spawn(async move {
                info!("扫描文件夹:{}", dir);
                let r = extract_file(&dir, tx).await;
                if let Err(e) = r {
                    error!("提取文件异常:{}", e)
                }
            });
        });
    }

    /// @Author: DengLibin
    /// @Date: Create in 2024-12-11 18:13:51
    /// @Description: 取关键词片段
    fn get_text_snippets(&mut self) {
        for my_file in self.files.iter_mut() {
            let snippets = text_utils::find_text_snippets(&my_file.content, &self.tokenize, 20, 3);

            let mut fragments = String::new();
            for snippet in snippets.into_iter() {
                fragments.push_str(&snippet);
            }
            my_file.content.clear();
            my_file.content.push_str(&fragments);
        }
    }

    /// @Author: DengLibin
    /// @Date: Create in 2024-12-11 15:49:03
    /// @Description: 按文件分组，合并内容
    fn group_by_file_path(files: Vec<MyFile>) -> Vec<MyFile> {
        let mut map: IndexMap<String, MyFile> = IndexMap::new();
        for my_file in files.into_iter() {
            let path = my_file.path.clone();
            let o = map.get_mut(path.as_str());
            if let Some(file) = o {
                file.content.push_str(&my_file.content); //追加
            } else {
                map.insert(path, my_file);
            }
        }

        map.into_iter().map(|(_key, value)| value).collect()
    }

    /// @Author: DengLibin
    /// @Date: Create in 2024-12-16 12:28:17
    /// @Description: 统计文件数量
    fn count_files(&mut self) {
        let file_count = self
            .runtime
            .block_on(async { files_dao::count(&self.sqlite_pool).await });
        self.file_count = file_count;
    }
}
