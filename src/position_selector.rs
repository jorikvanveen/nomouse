use std::{collections::HashMap, hash::Hash};

use cosmic_text::{Attrs, Buffer, Color, FontSystem, Metrics, Shaping, SwashCache};

use crate::render_utils::{alpha_multiply, draw_border, draw_rect};

#[derive(Debug, Default)]
pub struct FinalSelector {
    // 24 boxes
    // 3 rows
    // 8 columns
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
    pub n_rows: usize,
    pub n_cols: usize,
    pub depth: usize,
    pub keycodes: Vec<u32>,
}

impl FinalSelector {
    pub fn new(
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        n_rows: usize,
        n_cols: usize,
        keycodes: Vec<u32>,
    ) -> FinalSelector {
        Self {
            x,
            y,
            width,
            height,
            n_cols,
            n_rows,
            depth: 0,
            keycodes,
        }
    }

    pub fn select(&self, box_x: usize, box_y: usize) -> FinalSelector {
        //dbg!(&self);
        let width = self.width / self.n_cols;
        let height = self.height / self.n_rows;
        let x = self.x + width * box_x;
        let y = self.y + height * box_y;
        FinalSelector {
            x,
            y,
            width,
            height,
            n_rows: self.n_rows,
            n_cols: self.n_cols,
            depth: self.depth + 1,
            keycodes: self.keycodes.clone(),
        }
    }

    pub fn handle_input(&mut self, pressed_key: u32) {
        let idx = match self.keycodes.iter().position(|key| *key == pressed_key) {
            Some(idx) => idx,
            None => return,
        };

        let col = idx % self.n_cols;
        let row = idx / self.n_cols;
        dbg!(col, row);
        *self = self.select(col, row);
    }

    pub fn draw(
        &self,
        buf: &mut [u8],
        screen_width: usize,
        font_system: &mut FontSystem,
        swash_cache: &mut SwashCache,
        keycode_symbols: &HashMap<u32, String>,
    ) {
        draw_border(buf, self.x, self.y, self.width, self.height, screen_width);
        for row in 0..self.n_rows {
            for col in 0..self.n_cols {
                let width = self.width / self.n_cols;
                let height = self.height / self.n_rows;
                let x = self.x + width * col;
                let y = self.y + height * row;
                draw_border(buf, x, y, width, height, screen_width);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Rect {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

// Each box is assigned a sequence of two keycodes.
#[derive(Debug)]
pub struct InitialSelector {
    keycodes: Vec<u32>,
    rects: HashMap<(u32, u32), Rect>,
    last_key: Option<u32>,
}

impl InitialSelector {
    pub fn new(
        keycodes: Vec<u32>,
        n_rows: usize,
        n_cols: usize,
        screen_width: usize,
        screen_height: usize,
    ) -> Self {
        let mut rects = HashMap::new();
        for row in 0..n_rows {
            let row_key = keycodes[row];
            for col in 0..n_cols {
                let col_key = keycodes[col];
                let width = screen_width / n_rows;
                let height = screen_height / n_cols;
                let x = width * row;
                let y = height * col;
                rects.insert(
                    (row_key, col_key),
                    Rect {
                        x,
                        y,
                        width,
                        height,
                    },
                );
            }
        }
        Self {
            keycodes,
            rects,
            last_key: None,
        }
    }

    pub fn handle_input(&mut self, keycode: u32) -> Option<Rect> {
        dbg!(keycode);
        if let Some(last) = self.last_key {
            match self.rects.get(&(last, keycode)) {
                Some(rect) => return Some(rect.clone()),
                None => return None,
            }
        }

        if self.keycodes.contains(&keycode) {
            self.last_key = Some(keycode);
        }
        None
    }

    pub fn draw(
        &self,
        buf: &mut [u8],
        screen_width: usize,
        font_system: &mut FontSystem,
        swash_cache: &mut SwashCache,
        keycode_symbols: &HashMap<u32, String>,
    ) {
        for (keypair, rect) in self.rects.iter() {
            if let Some(key) = self.last_key {
                if key != keypair.0 {
                    continue;
                }
            }
            //dbg!(rect);
            let metrics = Metrics::new(14.0, 10.0);
            let mut buffer = Buffer::new(font_system, metrics);
            let mut buffer = buffer.borrow_with(font_system);
            let attrs = Attrs::new();
            buffer.set_text(
                format!(
                    "{}, {}",
                    keycode_symbols.get(&keypair.0).unwrap(),
                    keycode_symbols.get(&keypair.1).unwrap()
                )
                .as_str(),
                &attrs,
                Shaping::Advanced,
            );
            //buffer.shape_until_scroll(true);
            draw_border(buf, rect.x, rect.y, rect.width, rect.height, screen_width);
            buffer.draw(
                swash_cache,
                Color::rgb(0x0, 0x0, 0x0),
                |x, y, w, h, color| {
                    let color = alpha_multiply(color.as_rgba_tuple());
                    draw_rect(
                        buf,
                        x as usize + rect.x + 5,
                        y as usize + rect.y + 5,
                        w as usize,
                        h as usize,
                        screen_width,
                        color,
                    );
                },
            );
        }
    }
}

#[derive(Debug)]
pub enum SelectorState {
    Initial(InitialSelector),
    Final(FinalSelector),
}

impl SelectorState {
    pub fn draw(
        &self,
        buf: &mut [u8],
        screen_width: usize,
        font_system: &mut FontSystem,
        swash_cache: &mut SwashCache,
        keycode_symbols: &HashMap<u32, String>,
    ) {
        match self {
            SelectorState::Initial(selector) => {
                selector.draw(buf, screen_width, font_system, swash_cache, keycode_symbols)
            }
            SelectorState::Final(selector) => {
                selector.draw(buf, screen_width, font_system, swash_cache, keycode_symbols)
            }
        }
    }

    pub fn handle_key(&mut self, key: u32) {
        match self {
            SelectorState::Initial(initial_selector) => {
                match initial_selector.handle_input(key) {
                    Some(rect) => {
                        *self = SelectorState::Final(FinalSelector::new(
                            rect.x,
                            rect.y,
                            rect.width,
                            rect.height,
                            3,
                            8,
                            initial_selector.keycodes.clone(),
                        ))
                    }
                    None => {}
                };
            }
            SelectorState::Final(final_selector) => final_selector.handle_input(key),
        }
    }
}
//for y in selector.y..(selector.y + selector.height) {
//                for x in selector.x..(selector.x + selector.width) {
//                    let i = (y * screen_width + x) * 4;
//                    let (r, g, b, a) = alpha_multiply((0, 255, 0, 230));
//                    framebuf[i] = b; //blue
//                    framebuf[i + 1] = g; //green
//                    framebuf[i + 2] = r; //red
//                    framebuf[i + 3] = a; //alpha
//                }
//            }
//
//            for row in 0..selector.n_rows {
//                for col in 0..selector.n_cols {
//                    let x = selector.x + col * (selector.width / selector.n_cols);
//                    let y = selector.y + row * (selector.height / selector.n_rows);
//                    draw_border(
//                        framebuf,
//                        x,
//                        y,
//                        selector.width / selector.n_cols,
//                        selector.height / selector.n_rows,
//                        screen_width,
//                    );
//                }
//            }
//
//            surface.wl_surface.damage(0, 0, i32::MAX, i32::MAX);
//            surface.wl_surface.commit();
//            surface.wl_surface.attach(surface.wl_buf.as_ref(), 0, 0);
//

//buffer.draw(&mut swash_cache, text_color, |x, y, w, h, color| {
//    let [r, g, b, a] = color.as_rgba();
//    let (r, g, b, a) = alpha_multiply((r, g, b, a));
//    dbg!((x, y, w, h));
//    dbg!(&color.as_rgba());
//    draw_rect(
//        framebuf,
//        x as usize,
//        y as usize,
//        w as usize,
//        h as usize,
//        screen_width,
//        (r, g, b, a),
//    );
//});
