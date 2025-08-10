pub fn alpha_blend(foreground: (u8, u8, u8, u8), background: (u8, u8, u8, u8)) -> (u8, u8, u8, u8) {
    let (r_f, g_f, b_f, a_f) = foreground;
    let (r_b, g_b, b_b, a_b) = background;
    let a_f_f = a_f as f32 / 255.0;
    let a_b_f = a_b as f32 / 255.0;

    // Alpha blending formula for premultiplied colors
    let a_out = a_f_f + a_b_f * (1.0 - a_f_f);
    let r_out = (r_f as f32 + r_b as f32 * (1.0 - a_f_f)) / a_out;
    let g_out = (g_f as f32 + g_b as f32 * (1.0 - a_f_f)) / a_out;
    let b_out = (b_f as f32 + b_b as f32 * (1.0 - a_f_f)) / a_out;

    (
        (r_out * a_out) as u8,
        (g_out * a_out) as u8,
        (b_out * a_out) as u8,
        (a_out * 255.0) as u8,
    )
}

pub fn alpha_multiply(color: (u8, u8, u8, u8)) -> (u8, u8, u8, u8) {
    let (r, g, b, a) = color;
    let a_f = a as f32 / 255.0;
    let r = (r as f32 * a_f) as u8;
    let g = (g as f32 * a_f) as u8;
    let b = (b as f32 * a_f) as u8;
    (r, g, b, a)
}
pub fn draw_border(
    buf: &mut [u8],
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    screen_width: usize,
) {
    for x in x..(x + width) {
        set_pixel(buf, x, y, screen_width, (255, 255, 255, 255));
        set_pixel(buf, x, y + height, screen_width, (255, 255, 255, 255));
    }
    for y in y..(y + height) {
        set_pixel(buf, x, y, screen_width, (0, 0, 0, 255));
        set_pixel(buf, x + width, y, screen_width, (0, 0, 0, 255));
    }
}

pub fn draw_rect(
    buf: &mut [u8],
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    screen_width: usize,
    color: (u8, u8, u8, u8),
) {
    for x in x..(x + width) {
        for y in y..(y + height) {
            set_pixel(buf, x, y, screen_width, color);
        }
    }
}

pub fn set_pixel(buf: &mut [u8], x: usize, y: usize, screen_width: usize, color: (u8, u8, u8, u8)) {
    let background_color = get_pixel(buf, x, y, screen_width);
    let color = alpha_blend(color, background_color);
    let (r, g, b, a) = color;
    let i = (y * screen_width + x) * 4;
    buf[i] = b;
    buf[i + 1] = g;
    buf[i + 2] = r;
    buf[i + 3] = a;
}

pub fn get_pixel(buf: &[u8], x: usize, y: usize, screen_width: usize) -> (u8, u8, u8, u8) {
    let i = (y * screen_width + x) * 4;
    (buf[i], buf[i + 1], buf[i + 2], buf[i + 3])
}
