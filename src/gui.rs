//! GUI モード（egui / eframe）。

use std::path::PathBuf;

use eframe::egui;

use crate::filter;

/// GUI を起動する。
pub fn run() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([520.0, 280.0])
            .with_title("dicfilter"),
        ..Default::default()
    };
    eframe::run_native(
        "dicfilter",
        options,
        Box::new(|cc| {
            install_japanese_font(&cc.egui_ctx);
            Box::new(DicApp::default())
        }),
    )
}

struct DicApp {
    input: Option<PathBuf>,
    output: String,
    filter: String,
    status: String,
}

impl Default for DicApp {
    fn default() -> Self {
        Self {
            input: None,
            output: "output.txt".to_string(),
            filter: "filter.txt".to_string(),
            status: String::new(),
        }
    }
}

impl eframe::App for DicApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("dicfilter");
            ui.add_space(8.0);

            // 入力ファイル（ダイアログで選択）
            ui.horizontal(|ui| {
                ui.label("入力ファイル:");
                if ui.button("選択...").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("テキストファイル", &["txt"])
                        .add_filter("すべてのファイル", &["*"])
                        .pick_file()
                    {
                        self.input = Some(path);
                    }
                }
            });
            let input_label = self
                .input
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "（未選択）".to_string());
            ui.label(format!("  {}", input_label));
            ui.add_space(6.0);

            // 出力ファイル（パス指定）
            ui.horizontal(|ui| {
                ui.label("出力ファイル:");
                ui.text_edit_singleline(&mut self.output);
            });

            // フィルタファイル（パス指定）
            ui.horizontal(|ui| {
                ui.label("フィルタファイル:");
                ui.text_edit_singleline(&mut self.filter);
            });

            ui.add_space(12.0);

            // 実行ボタン
            if ui.button("実行").clicked() {
                self.execute();
            }

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(6.0);
            ui.label(&self.status);
        });
    }
}

impl DicApp {
    fn execute(&mut self) {
        let Some(input) = self.input.as_ref() else {
            self.status = "入力ファイルを選択してください。".to_string();
            return;
        };
        // 既存ファイルを上書きしないよう、衝突しない出力パスを決定する。
        let output = filter::resolve_output_path(&PathBuf::from(self.output.trim()));
        let filter_path = PathBuf::from(self.filter.trim());

        match filter::run(input, &output, &filter_path) {
            Ok(_count) => {
                self.status = format!("{}に出力しました", output.display());
            }
            Err(err) => {
                // エラーログを標準エラー出力へ
                eprintln!("[エラー] {}", err);
                // 出力ファイルへ日本語のエラー詳細を記載（ベストエフォート）
                filter::write_error_report(&output, &err);
                self.status =
                    format!("エラーが発生しました。{}を確認してください", output.display());
            }
        }
    }
}

/// システムにインストールされている CJK フォントを探して egui に登録する。
///
/// 見つからない場合はデフォルトフォントのままとなる（日本語は表示されない）。
fn install_japanese_font(ctx: &egui::Context) {
    const CANDIDATES: &[&str] = &[
        // Windows
        "C:/Windows/Fonts/YuGothM.ttc",
        "C:/Windows/Fonts/meiryo.ttc",
        "C:/Windows/Fonts/msgothic.ttc",
        // macOS
        "/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc",
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
        "/Library/Fonts/Arial Unicode.ttf",
        // Linux
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJKjp-Regular.otf",
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/google-noto-cjk/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/fonts-japanese-gothic.ttf",
    ];

    for path in CANDIDATES {
        if let Ok(bytes) = std::fs::read(path) {
            let mut fonts = egui::FontDefinitions::default();
            fonts
                .font_data
                .insert("cjk".to_owned(), egui::FontData::from_owned(bytes));
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "cjk".to_owned());
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .push("cjk".to_owned());
            ctx.set_fonts(fonts);
            return;
        }
    }
}
