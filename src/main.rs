#![allow(unused)]

use eframe::NativeOptions;
use egui::{panel::Side, CentralPanel, Grid, Layout, SidePanel, Slider, Vec2};
use egui_plotter::{Chart, EguiBackend, MouseConfig};
use plotters::prelude::*;

fn main() {
    eframe::run_native(
        "Riemann Visualizer",
        NativeOptions {
            initial_window_size: Some(Vec2::new(1000.0, 700.0)),
            ..Default::default()
        },
        Box::new(|cc| {
            Box::new(App {
                range: (0.0, 10.0),
                domain: (0.0, 10.0),
                rect_h: 0.5,
                num_rectangles: 5,
                eq: "x".into(),
                show_sums: true,
                ..Default::default()
            })
        }),
    )
    .unwrap();
}

#[derive(Debug, Default)]
struct App {
    /// The equation text buffer which will be parsed into a function to plot
    pub eq: String,

    /// The range of y values to graph
    pub range: (f32, f32),

    /// The domain of x values to graph
    pub domain: (f32, f32),

    /// Whether to show the riemann sums triangles or not
    pub show_sums: bool,

    /// Number of rectangles to use
    pub num_rectangles: i32,

    /// Coefficient to use when deciding what part of the rectangle should touch the line of the equation
    ///
    /// - 0 = Left corner
    /// - 1 = Right corner
    pub rect_h: f32,

    /// Area of the rectangles
    pub rect_area: f32,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        SidePanel::new(Side::Left, "options_panel")
            .min_width(115.0)
            .show(ctx, |ui| {
                Grid::new("the_grid").num_columns(2).show(ui, |ui| {
                    ui.label("Equation: ");
                    ui.text_edit_singleline(&mut self.eq);
                    ui.end_row();

                    ui.label("Show Rectangles: ");
                    ui.checkbox(&mut self.show_sums, "");
                    ui.end_row();

                    ui.label("Rectangles: ");
                    ui.add(Slider::new(&mut self.num_rectangles, 0..=30));
                    ui.end_row();

                    ui.label("Coefficient: ");
                    ui.add(Slider::new(&mut self.rect_h, 0.0..=1.0));
                    ui.end_row();

                    ui.end_row();

                    ui.label("Rectangle Area: ");
                    ui.label(format!("{}", &self.rect_area));
                })
            });
        CentralPanel::default().show(ctx, |ui| {
            let root = EguiBackend::new(ui).into_drawing_area();

            let mut chart = ChartBuilder::on(&root)
                .margin(5)
                .x_label_area_size(30)
                .y_label_area_size(30)
                .build_cartesian_2d(self.domain.0..self.domain.1, self.range.0..self.range.1)
                .unwrap();

            chart
                .configure_mesh()
                .axis_style(&WHITE)
                .label_style(&WHITE)
                .bold_line_style(&RGBAColor(255, 255, 255, 0.025))
                .draw()
                .unwrap();

            let expr: meval::Expr = match self.eq.parse() {
                Ok(x) => x,
                Err(_) => return,
            };

            if let Ok(f) = expr.bind("x") {
                // Plot the equation
                let values = ((self.domain.0 * 1000.0) as i32..(self.domain.1 * 1000.0) as i32)
                    .map(|x| x as f32 / 1000.0)
                    .map(|x| (x, f(x as f64) as f32))
                    .filter(|p| p.1 > self.range.0 && p.1 < self.range.1);
                chart.draw_series(LineSeries::new(values, &RED)).unwrap();

                // Plot the rectangles
                if self.show_sums {
                    let width = (self.domain.1 - self.domain.0) / self.num_rectangles as f32;
                    let rect_coords = (0..self.num_rectangles)
                        .map(|x| x as f32)
                        .map(|x| self.domain.0 + width * x) // this x value is the left most beginning of the rectangle
                        .map(|x| (x, f((x + self.rect_h * width) as f64) as f32));

                    // Get area
                    self.rect_area = rect_coords
                        .clone()
                        .map(|x| width * x.1)
                        .fold(0f32, |acc, x| acc + x);

                    // Create rectangles and plot them
                    rect_coords
                        .map(|x| ((x.0, x.1), (x.0 + width, 0.0f32))) // Get the upper left and bottom right corner
                        .map(|c| Rectangle::new([c.0, c.1], BLUE))
                        .for_each(|rec| chart.plotting_area().draw(&rec).unwrap());
                }
            }

            root.present().unwrap();
        });
    }
}
