use qrcode::QrCode;

/// Minecraft map color index
/// Use basic color palette: white background + black foreground
const MAP_COLOR_WHITE: u8 = 34;  // White (quartz block series, base color 8)
const MAP_COLOR_BLACK: u8 = 119; // Black (base color 29)

/// Render QR code to 128x128 Minecraft map pixel data
pub fn render_qr_to_map(data: &str) -> Option<[u8; 128 * 128]> {
    let code = QrCode::new(data.as_bytes()).ok()?;
    let modules = code.to_colors();
    let qr_size = code.width();

    // Calculate scaling: make QR code fit 128x128, keep margin
    let margin = 4;
    let available = 128 - margin * 2;
    let scale = available / qr_size;
    let scale = if scale < 1 { 1 } else { scale };

    let total_qr_pixels = qr_size * scale;
    let offset_x = (128 - total_qr_pixels) / 2;
    let offset_y = (128 - total_qr_pixels) / 2;

    let mut pixels = [MAP_COLOR_WHITE; 128 * 128];

    for qr_y in 0..qr_size {
        for qr_x in 0..qr_size {
            let is_dark = modules[qr_y * qr_size + qr_x] == qrcode::Color::Dark;
            if is_dark {
                for dy in 0..scale {
                    for dx in 0..scale {
                        let px = offset_x + qr_x * scale + dx;
                        let py = offset_y + qr_y * scale + dy;
                        if px < 128 && py < 128 {
                            pixels[py * 128 + px] = MAP_COLOR_BLACK;
                        }
                    }
                }
            }
        }
    }

    Some(pixels)
}
