
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

struct SizeComparison {
    original: usize,
    compressed: usize
}

struct Application {
    status: String,
    size_comparison: Option<SizeComparison>
}

impl Default for Application {
    fn default() -> Self {
        Self {
            status: "Compress or decompress a Huffman encoded file".to_owned(),
            size_comparison: None
        }
    }
}

fn compress_with_filepath(app: &mut Application, filepath: String) {
    app.status = format!("Compressing {}", filepath);

    match Huffman::from_file(&filepath) {
        Ok((huffman, text_size)) => {
            let compressed = huffman.compress();
            let (serialised_filepath, compressed_size) = compressed.serialise(filepath).unwrap();
            app.status = format!("Saved compressed file to {}", serialised_filepath);
            app.size_comparison = Some (SizeComparison {
                original: text_size,
                compressed: compressed_size
            });
        },
        Err(err) => {
            app.status = err;
        }
    }
}

impl eframe::App for Application {
    fn update(mut self: &mut Self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(&mut self.status);

            // ui.toggle_value(&mut self.toggle, "Whatev");

            if ui.button("Open file").clicked() {
                let filepath = tinyfiledialogs::open_file_dialog("File to compress", "", None);
                match filepath {
                    None => (),
                    Some(filepath) => {
                        compress_with_filepath(&mut self, filepath)
                    }
                }
            }
        });
        return;
    }

}
