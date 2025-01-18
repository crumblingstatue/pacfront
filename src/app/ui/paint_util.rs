use {
    eframe::egui,
    std::f32::consts::{FRAC_PI_4, PI},
};

pub fn draw_logo(painter: &egui::Painter, center: egui::Pos2, radius: f32) {
    // Draw a pacman shape
    let start_angle = PI + FRAC_PI_4;
    let end_angle = 3.0 * PI - FRAC_PI_4;

    let mut points = vec![center];
    let num_points = 16;
    for i in 0..=num_points {
        let t = i as f32 / num_points as f32;
        let angle = start_angle + t * (end_angle - start_angle);
        points.push(egui::pos2(
            center.x + radius * angle.cos(),
            center.y + radius * angle.sin(),
        ));
    }
    points.push(center);

    painter.add(egui::epaint::PathShape::convex_polygon(
        points,
        egui::Color32::YELLOW,
        egui::Stroke::new(1.0, egui::Color32::BLACK),
    ));

    // Draw a pellet
    painter.circle(
        center - egui::vec2(8.0, 0.0),
        3.0,
        egui::Color32::LIGHT_YELLOW,
        egui::Stroke::new(1.0, egui::Color32::BLACK),
    );
}
