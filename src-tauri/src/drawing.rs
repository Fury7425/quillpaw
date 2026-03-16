use serde::{Deserialize, Serialize};
use tiny_skia::{Color, LineCap, Paint, PathBuilder, Pixmap, Stroke, Transform};
use tokio::fs;

#[derive(Deserialize, Serialize)]
struct DrawingPayload {
    canvas: CanvasSpec,
    strokes: Vec<StrokeSpec>,
}

#[derive(Deserialize, Serialize)]
struct CanvasSpec {
    width: u32,
    height: u32,
    background: String,
}

#[derive(Deserialize, Serialize)]
struct StrokeSpec {
    tool: String,
    color: String,
    width: f32,
    opacity: f32,
    points: Vec<[f32; 3]>,
}

/// Render a drawing JSON payload into a PNG buffer.
pub async fn render_png(drawing_json: &str) -> Result<Vec<u8>, String> {
    let payload: DrawingPayload = serde_json::from_str(drawing_json).map_err(|e| e.to_string())?;
    let mut pixmap = Pixmap::new(payload.canvas.width, payload.canvas.height)
        .ok_or("Invalid canvas dimensions")?;

    let bg = parse_hex_color(&payload.canvas.background)?;
    pixmap.fill(bg);

    for stroke_spec in payload.strokes {
        if stroke_spec.points.len() < 2 {
            continue;
        }

        let mut pb = PathBuilder::new();
        let p0 = stroke_spec.points[0];
        pb.move_to(p0[0], p0[1]);

        for p in stroke_spec.points.iter().skip(1) {
            pb.line_to(p[0], p[1]);
        }

        let path = pb.finish().ok_or("Invalid stroke path")?;
        let mut paint = Paint::default();
        let mut color = if stroke_spec.tool == "eraser" {
            bg
        } else {
            parse_hex_color(&stroke_spec.color)?
        };
        color.set_alpha(stroke_spec.opacity);
        paint.set_color(color);
        paint.anti_alias = true;

        let mut stroke = Stroke::default();
        stroke.width = stroke_spec.width;
        stroke.line_cap = LineCap::Round;

        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    }

    pixmap.encode_png().map_err(|e| e.to_string())
}

fn parse_hex_color(hex: &str) -> Result<Color, String> {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Invalid R")?;
        let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "Invalid G")?;
        let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Invalid B")?;
        Ok(Color::from_rgba8(r, g, b, 255))
    } else if hex.len() == 8 {
        let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Invalid R")?;
        let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "Invalid G")?;
        let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Invalid B")?;
        let a = u8::from_str_radix(&hex[6..8], 16).map_err(|_| "Invalid A")?;
        Ok(Color::from_rgba8(r, g, b, a))
    } else {
        // Fallback or error
        Ok(Color::from_rgba8(30, 30, 30, 255))
    }
}

/// Save a drawing JSON file to the vault drawings folder.
pub async fn save(vault_path: &str, filename: &str, drawing_json: &str) -> Result<String, String> {
    let p = format!("{vault_path}/.assets/drawings/{filename}");
    fs::create_dir_all(format!("{vault_path}/.assets/drawings"))
        .await
        .map_err(|e| e.to_string())?;
    fs::write(&p, drawing_json)
        .await
        .map_err(|e| e.to_string())?;
    Ok(p)
}

/// Load a drawing JSON file from the vault drawings folder.
pub async fn load(vault_path: &str, filename: &str) -> Result<String, String> {
    let p = format!("{vault_path}/.assets/drawings/{filename}");
    fs::read_to_string(p).await.map_err(|e| e.to_string())
}
