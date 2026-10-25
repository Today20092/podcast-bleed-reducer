//! Test-only CPU rendering of the actual egui meshes; no native window or audio device.
use egui::{ColorImage, TextureId, epaint::Primitive};
use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufWriter, Write},
    path::Path,
};

pub fn write_capture(
    context: &egui::Context,
    output: &mut egui::FullOutput,
    path: &Path,
    width: usize,
    height: usize,
) -> io::Result<()> {
    // Consume all texture updates even when validation fails: egui checks their Drop.
    let updates = std::mem::take(&mut output.textures_delta.set);
    output.textures_delta.clear();
    if width == 0 || height == 0 || width > 4096 || height > 4096 {
        return Err(io::Error::other("Invalid capture dimensions"));
    }
    let mut textures = HashMap::<TextureId, ColorImage>::new();
    for (id, deltas) in updates {
        for update in deltas {
            let egui::ImageData::Color(image) = update.image;
            if let Some([x, y]) = update.pos {
                let texture = textures.get_mut(&id).ok_or_else(|| {
                    io::Error::other("Capture requires an initial full texture upload")
                })?;
                if x + image.size[0] > texture.size[0] || y + image.size[1] > texture.size[1] {
                    return Err(io::Error::other("Texture delta exceeds its allocation"));
                }
                for row in 0..image.size[1] {
                    let destination = (y + row) * texture.size[0] + x;
                    texture.pixels[destination..destination + image.size[0]].copy_from_slice(
                        &image.pixels[row * image.size[0]..(row + 1) * image.size[0]],
                    );
                }
            } else {
                textures.insert(id, (*image).clone());
            }
        }
    }
    let mut pixels = vec![[32.0, 34.0, 38.0]; width * height];
    for clipped in context.tessellate(output.shapes.clone(), 1.0) {
        let Primitive::Mesh(mesh) = clipped.primitive else {
            return Err(io::Error::other(
                "Native paint callbacks cannot be captured by the CPU check",
            ));
        };
        let texture = textures.get(&mesh.texture_id).ok_or_else(|| {
            io::Error::other("Mesh references a texture absent from this capture")
        })?;
        for indices in mesh.indices.as_chunks::<3>().0 {
            let vertices = [
                mesh.vertices[indices[0] as usize],
                mesh.vertices[indices[1] as usize],
                mesh.vertices[indices[2] as usize],
            ];
            let [a, b, c] = vertices;
            let cross = |x: egui::Vec2, y: egui::Vec2| x.x * y.y - x.y * y.x;
            let area = cross(b.pos - a.pos, c.pos - a.pos);
            if area.abs() < 1e-6 {
                continue;
            }
            let min_x = a
                .pos
                .x
                .min(b.pos.x)
                .min(c.pos.x)
                .max(clipped.clip_rect.min.x)
                .max(0.0)
                .floor() as usize;
            let min_y = a
                .pos
                .y
                .min(b.pos.y)
                .min(c.pos.y)
                .max(clipped.clip_rect.min.y)
                .max(0.0)
                .floor() as usize;
            let max_x = a
                .pos
                .x
                .max(b.pos.x)
                .max(c.pos.x)
                .min(clipped.clip_rect.max.x)
                .min(width as f32)
                .ceil() as usize;
            let max_y = a
                .pos
                .y
                .max(b.pos.y)
                .max(c.pos.y)
                .min(clipped.clip_rect.max.y)
                .min(height as f32)
                .ceil() as usize;
            for y in min_y..max_y {
                for x in min_x..max_x {
                    let point = egui::pos2(x as f32 + 0.5, y as f32 + 0.5);
                    if !clipped.clip_rect.contains(point) {
                        continue;
                    }
                    let weights = [
                        cross(b.pos - point, c.pos - point) / area,
                        cross(c.pos - point, a.pos - point) / area,
                        cross(a.pos - point, b.pos - point) / area,
                    ];
                    if weights.iter().any(|&weight| weight < -1e-6) {
                        continue;
                    }
                    let uv = a.uv.to_vec2() * weights[0]
                        + b.uv.to_vec2() * weights[1]
                        + c.uv.to_vec2() * weights[2];
                    let texel = sample(texture, uv);
                    let mut color = [0.0_f32; 4];
                    for (vertex, weight) in vertices.iter().zip(weights) {
                        for (channel, value) in color.iter_mut().zip(vertex.color.to_array()) {
                            *channel += f32::from(value) * weight;
                        }
                    }
                    let alpha = color[3] * texel[3] / (255.0 * 255.0);
                    for (channel, destination) in pixels[y * width + x].iter_mut().enumerate() {
                        *destination =
                            color[channel] * texel[channel] / 255.0 + *destination * (1.0 - alpha);
                    }
                }
            }
        }
    }
    let mut writer = BufWriter::new(File::create(path)?);
    write!(writer, "P6\n{width} {height}\n255\n")?;
    for pixel in pixels {
        writer.write_all(&pixel.map(|value| value.round().clamp(0.0, 255.0) as u8))?;
    }
    writer.flush()
}

fn sample(image: &ColorImage, uv: egui::Vec2) -> [f32; 4] {
    let x = (uv.x * image.size[0] as f32 - 0.5).clamp(0.0, (image.size[0] - 1) as f32);
    let y = (uv.y * image.size[1] as f32 - 0.5).clamp(0.0, (image.size[1] - 1) as f32);
    let (left, top) = (x.floor() as usize, y.floor() as usize);
    let (right, bottom) = (
        (left + 1).min(image.size[0] - 1),
        (top + 1).min(image.size[1] - 1),
    );
    let mut color = [0.0; 4];
    for (column, row, weight) in [
        (left, top, (1.0 - x.fract()) * (1.0 - y.fract())),
        (right, top, x.fract() * (1.0 - y.fract())),
        (left, bottom, (1.0 - x.fract()) * y.fract()),
        (right, bottom, x.fract() * y.fract()),
    ] {
        for (channel, value) in color
            .iter_mut()
            .zip(image.pixels[row * image.size[0] + column].to_array())
        {
            *channel += f32::from(value) * weight;
        }
    }
    color
}

#[test]
fn cpu_capture_renders_color_and_respects_clip() {
    let context = egui::Context::default();
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(64.0, 64.0),
        )),
        ..Default::default()
    };
    let mut output = context.run_ui(input, |ui| {
        ui.label("Fonts");
        let painter = ui.painter().with_clip_rect(egui::Rect::from_min_max(
            egui::pos2(20.0, 20.0),
            egui::pos2(40.0, 40.0),
        ));
        painter.rect_filled(
            egui::Rect::from_min_max(egui::pos2(10.0, 10.0), egui::pos2(50.0, 50.0)),
            0.0,
            egui::Color32::RED,
        );
    });
    let path = std::env::temp_dir().join(format!(
        "cleanslate-egui-capture-{}.ppm",
        std::process::id()
    ));
    write_capture(&context, &mut output, &path, 64, 64).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    std::fs::remove_file(&path).unwrap();
    let header = b"P6\n64 64\n255\n";
    assert!(bytes.starts_with(header));
    let pixel = |x: usize, y: usize| {
        &bytes[header.len() + (y * 64 + x) * 3..header.len() + (y * 64 + x) * 3 + 3]
    };
    assert_eq!(pixel(30, 30), [255, 0, 0]);
    assert_eq!(pixel(48, 48), [32, 34, 38]);
    assert!(output.textures_delta.is_empty());
}
