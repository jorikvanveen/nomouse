use cosmic_text::{Attrs, Buffer, Color, Family, FontSystem, Metrics, Shaping, SwashCache};
use nanoid::nanoid;
use smithay_client_toolkit::seat::pointer::BTN_LEFT;
use std::{
    collections::HashMap,
    i32,
    os::{fd::AsFd, raw::c_void, unix::prelude::OwnedFd},
    ptr::NonNull,
};

use nix::{
    fcntl::OFlag,
    sys::{
        mman::{ProtFlags, shm_open, shm_unlink},
        stat::Mode,
    },
    unistd::ftruncate,
};
use wayland_client::{
    EventQueue,
    protocol::{wl_pointer::ButtonState, wl_shm::Format},
};
mod app;
mod buf_utils;
mod output_utils;
mod position_selector;
mod render_utils;
use app::AppData;

use crate::position_selector::{InitialSelector, SelectorState};

fn main() {
    // qwer uiop
    // asdf jkl;
    // zxcv m,./
    let keycodes: Vec<u32> = vec![
        16, 17, 18, 19, 22, 23, 24, 25, 30, 31, 32, 33, 36, 37, 38, 39, 44, 45, 46, 47, 50, 51, 52,
        53,
    ];
    let keycode_symbols: HashMap<u32, String> = HashMap::from([
        (16, "q".into()),
        (17, "w".into()),
        (18, "e".into()),
        (19, "r".into()),
        (22, "u".into()),
        (23, "i".into()),
        (24, "o".into()),
        (25, "p".into()),
        (30, "a".into()),
        (31, "s".into()),
        (32, "d".into()),
        (33, "f".into()),
        (36, "j".into()),
        (37, "k".into()),
        (38, "l".into()),
        (39, ";".into()),
        (44, "z".into()),
        (45, "x".into()),
        (46, "c".into()),
        (47, "v".into()),
        (50, "m".into()),
        (51, ",".into()),
        (52, ".".into()),
        (53, "/".into()),
    ]);
    let conn = wayland_client::Connection::connect_to_env().unwrap();
    let display = conn.display();
    let mut event_queue: EventQueue<AppData> = conn.new_event_queue();
    let qh = event_queue.handle();
    let _registry = display.get_registry(&qh, ());
    let mut app = AppData {
        ..Default::default()
    };
    app.init_that_shit(&mut event_queue);
    app.selector = Some(SelectorState::Initial(InitialSelector::new(
        keycodes, 12, 16, 1920, 1080,
    )));
    let mut font_system = FontSystem::new();
    let mut swash_cache = SwashCache::new();
    let metrics = Metrics::new(28.0, 30.0);
    let mut buffer = Buffer::new(&mut font_system, metrics);
    let mut buffer = buffer.borrow_with(&mut font_system);
    buffer.set_size(Some(80.0), Some(25.0));
    let attrs = Attrs::new().family(Family::Serif);
    buffer.set_text("hi", &attrs, Shaping::Advanced);
    buffer.shape_until_scroll(true);
    let text_color = Color::rgb(0x0, 0x0, 0x0);

    let mut offset: u8 = 0;
    loop {
        println!("rendering");
        offset = offset.wrapping_add(1);
        let surface = app.surface.as_mut().unwrap();
        let framebuf = surface.buf.as_mut_slice();
        for i in 0..framebuf.len() / 4 {
            // grey background
            framebuf[i * 4] = 128_u8.wrapping_add(1);
            framebuf[i * 4 + 1] = 128;
            framebuf[i * 4 + 2] = 128;
            framebuf[i * 4 + 3] = 128;
        }

        let monitor = app
            .outputs
            .get(app.selected_output.as_ref().unwrap())
            .unwrap();
        let screen_width = monitor.width.unwrap() as usize;
        app.selector.as_ref().unwrap().draw(
            framebuf,
            screen_width,
            &mut font_system,
            &mut swash_cache,
            &keycode_symbols,
        );

        if let SelectorState::Final(selector) = app.selector.as_ref().unwrap() {
            if selector.depth == 1 {
                app.layer_surface.as_ref().unwrap().destroy();
                app.layer_shell.as_ref().unwrap().destroy();
                surface.wl_surface.destroy();
                let monitor_x = monitor.x.unwrap();
                let monitor_y = monitor.y.unwrap();
                let width = monitor.width.unwrap();
                let height = monitor.height.unwrap();

                app.pointer.as_ref().unwrap().motion_absolute(
                    0,
                    monitor_x as u32 + selector.x as u32 + (selector.width / 2) as u32,
                    monitor_y as u32 + selector.y as u32 + (selector.height / 2) as u32,
                    width as u32,
                    height as u32,
                );
                app.pointer
                    .as_ref()
                    .unwrap()
                    .button(1, BTN_LEFT, ButtonState::Pressed);
                app.pointer
                    .as_ref()
                    .unwrap()
                    .button(2, BTN_LEFT, ButtonState::Released);
                event_queue.blocking_dispatch(&mut app).unwrap();
                break;
            }
        }
        surface.wl_surface.attach(surface.wl_buf.as_ref(), 0, 0);
        surface.wl_surface.damage(0, 0, i32::MAX, i32::MAX);
        surface.wl_surface.commit();
        event_queue.blocking_dispatch(&mut app).unwrap();
    }
}
