
mod huffman;
use huffman::Huffman;

use eframe::egui;
use tinyfiledialogs;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([500.0, 300.0]),
        ..Default::default()
    };

    return
        eframe::run_native(
            "Hello egui",
            options,
            Box::new(|_| {
                Box::<Application>::default()
            })
        )
}

struct Application {
    status: String
}

impl Default for Application {
    fn default() -> Self {
        Self {
            status: "Compress or decompress a Huffman encoded file".to_owned()
        }
    }
}

impl eframe::App for Application {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(&mut self.status);

            // ui.toggle_value(&mut self.toggle, "Whatev");

            if ui.button("Open file").clicked() {
                let filepath = tinyfiledialogs::open_file_dialog("File to compress", "", None);
                match filepath {
                    None => (),
                    Some(filepath) => {
                        self.status = format!("Compressing {}", filepath);
                        let huffman = Huffman::from_file(&filepath)
                            .expect("A valid file to create a Huffman-compressed file");
                        let compressed = huffman.compress();
                        let serialised_filepath = compressed.serialise(filepath).unwrap();
                        self.status = format!("Saved compressed file to {}", serialised_filepath);
                    }
                }
            }
        });
        return;
    }
}
