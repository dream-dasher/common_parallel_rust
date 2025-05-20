//! A simple egui example showing a grid of colored cells

use eframe::egui;
use egui::{Color32, CornerRadius, Rect, Stroke, Vec2};

fn main() {
        let native_options = eframe::NativeOptions {
                viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
                ..Default::default()
        };

        eframe::run_native(
                "Egui Grid Example",
                native_options,
                Box::new(|cc| {
                        egui_extras::install_image_loaders(&cc.egui_ctx);
                        Ok(Box::new(GridApp::new()))
                }),
        )
        .unwrap();
}

struct GridApp {
        grid: Vec<Vec<bool>>,
        grid_size: usize,
        cell_color: Color32,
}

impl GridApp {
        fn new() -> Self {
                let grid_size = 10;
                let mut grid = Vec::with_capacity(grid_size);

                for _ in 0..grid_size {
                        let mut row = Vec::with_capacity(grid_size);
                        for _ in 0..grid_size {
                                // Initialize all cells as false (black)
                                row.push(false);
                        }
                        grid.push(row);
                }

                Self {
                        grid,
                        grid_size,
                        cell_color: Color32::from_rgb(46, 182, 125), // Default color: greenish
                }
        }

        fn toggle_cell(&mut self, row: usize, col: usize) {
                if row < self.grid_size && col < self.grid_size {
                        self.grid[row][col] = !self.grid[row][col];
                }
        }

        fn randomize_grid(&mut self) {
                for row in 0..self.grid_size {
                        for col in 0..self.grid_size {
                                self.grid[row][col] = rand::random::<bool>();
                        }
                }
        }

        fn clear_grid(&mut self) {
                for row in 0..self.grid_size {
                        for col in 0..self.grid_size {
                                self.grid[row][col] = false;
                        }
                }
        }

        fn fill_grid(&mut self) {
                for row in 0..self.grid_size {
                        for col in 0..self.grid_size {
                                self.grid[row][col] = true;
                        }
                }
        }

        fn resize_grid(&mut self, new_size: usize) {
                self.grid_size = new_size;

                // Create a new grid with the updated size
                let mut new_grid = Vec::with_capacity(new_size);
                for i in 0..new_size {
                        let mut row = Vec::with_capacity(new_size);
                        for j in 0..new_size {
                                // Preserve existing cell values where possible
                                if i < self.grid.len() && j < self.grid[i].len() {
                                        row.push(self.grid[i][j]);
                                } else {
                                        row.push(false);
                                }
                        }
                        new_grid.push(row);
                }

                self.grid = new_grid;
        }
}

impl eframe::App for GridApp {
        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
                egui::CentralPanel::default().show(ctx, |ui| {
                        ui.heading("Grid Example");

                        // Control panel
                        ui.horizontal(|ui| {
                                if ui.button("Randomize").clicked() {
                                        self.randomize_grid();
                                }

                                if ui.button("Clear All").clicked() {
                                        self.clear_grid();
                                }

                                if ui.button("Fill All").clicked() {
                                        self.fill_grid();
                                }

                                let mut temp_grid_size = self.grid_size;
                                ui.add(egui::Slider::new(&mut temp_grid_size, 5..=30).text("Grid Size"));
                                if ui.button("Apply Size").clicked() {
                                        self.resize_grid(temp_grid_size);
                                }

                                ui.color_edit_button_srgba(&mut self.cell_color);
                        });

                        ui.separator();

                        // Draw the grid
                        let available_rect = ui.available_rect_before_wrap();
                        let grid_side = (available_rect.width().min(available_rect.height()) - 20.0).max(100.0);
                        let cell_size = grid_side / self.grid_size as f32;

                        // Add some padding around the grid
                        let grid_rect = Rect::from_min_size(
                                available_rect.min + Vec2::new(10.0, 10.0),
                                Vec2::new(grid_side, grid_side),
                        );

                        // Background for the grid
                        ui.painter()
                                .rect_filled(grid_rect, CornerRadius::ZERO, Color32::from_gray(240));

                        // Draw grid lines
                        for i in 0..=self.grid_size {
                                let x = grid_rect.min.x + (i as f32 * cell_size);
                                let y = grid_rect.min.y + (i as f32 * cell_size);

                                // Horizontal line
                                ui.painter().line_segment(
                                        [egui::Pos2::new(grid_rect.min.x, y), egui::Pos2::new(grid_rect.max.x, y)],
                                        Stroke::new(1.0, Color32::GRAY),
                                );

                                // Vertical line
                                ui.painter().line_segment(
                                        [egui::Pos2::new(x, grid_rect.min.y), egui::Pos2::new(x, grid_rect.max.y)],
                                        Stroke::new(1.0, Color32::GRAY),
                                );
                        }

                        // Draw cells
                        for row in 0..self.grid.len() {
                                for col in 0..self.grid[row].len() {
                                        let cell_min_x = grid_rect.min.x + (col as f32 * cell_size);
                                        let cell_min_y = grid_rect.min.y + (row as f32 * cell_size);

                                        let cell_rect = Rect::from_min_size(
                                                egui::Pos2::new(cell_min_x, cell_min_y),
                                                Vec2::new(cell_size, cell_size),
                                        );

                                        // Fill cell with color or black based on its state
                                        let fill_color =
                                                if self.grid[row][col] { self.cell_color } else { Color32::BLACK };

                                        ui.painter().rect_filled(
                                                cell_rect.shrink(1.0), // Shrink slightly to see the grid lines
                                                CornerRadius::ZERO,
                                                fill_color,
                                        );

                                        // Handle cell click
                                        let cell_response = ui.allocate_rect(cell_rect, egui::Sense::click());
                                        if cell_response.clicked() {
                                                self.toggle_cell(row, col);
                                        }
                                }
                        }
                });
        }
}
