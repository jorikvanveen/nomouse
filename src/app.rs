use std::{
    collections::{HashMap, HashSet},
    i32,
    os::{fd::AsFd, linux::raw::stat},
};

use smithay_client_toolkit::seat::pointer::BTN_LEFT;
use wayland_client::{
    Dispatch, EventQueue, WEnum,
    protocol::{
        wl_buffer::WlBuffer,
        wl_callback::WlCallback,
        wl_compositor::WlCompositor,
        wl_keyboard::{KeyState, WlKeyboard},
        wl_output::{Mode, WlOutput},
        wl_pointer::ButtonState,
        wl_registry::WlRegistry,
        wl_seat::WlSeat,
        wl_shm::{self, Format, WlShm},
        wl_shm_pool::WlShmPool,
        wl_surface::WlSurface,
    },
};
use wayland_protocols::xdg::shell::client::{
    xdg_surface::XdgSurface,
    xdg_toplevel::{self, XdgToplevel},
    xdg_wm_base::XdgWmBase,
};
use wayland_protocols_wlr::{
    layer_shell::v1::client::{
        zwlr_layer_shell_v1::{Layer, ZwlrLayerShellV1},
        zwlr_layer_surface_v1::{self, Anchor, KeyboardInteractivity, ZwlrLayerSurfaceV1},
    },
    virtual_pointer::v1::client::{
        zwlr_virtual_pointer_manager_v1::ZwlrVirtualPointerManagerV1,
        zwlr_virtual_pointer_v1::ZwlrVirtualPointerV1,
    },
};

use crate::{
    app,
    buf_utils::{Surface, allocate_shm_buffer},
    output_utils::Output,
    position_selector::{FinalSelector, SelectorState},
};

#[derive(Default, Debug)]
pub struct AppData {
    pub compositor: Option<WlCompositor>,
    pub shm: Option<WlShm>,
    pub formats: Vec<WEnum<Format>>,
    pub xdg_wm_base: Option<XdgWmBase>,
    pub layer_shell: Option<ZwlrLayerShellV1>,
    pub seat: Option<WlSeat>,
    pub keyboard: Option<WlKeyboard>,
    pub outputs: HashMap<u32, Output>,
    pub surface: Option<Surface>,
    pub layer_surface: Option<ZwlrLayerSurfaceV1>,
    pub virtual_pointer_manager: Option<ZwlrVirtualPointerManagerV1>,
    pub pointer: Option<ZwlrVirtualPointerV1>,
    pub adding_output: Option<u32>,
    pub selected_output: Option<u32>,
    pub procesed_keypress_serials: HashSet<u32>,
    pub selector: Option<SelectorState>,
    pub do_click: bool,
}

impl AppData {
    pub fn init_that_shit(&mut self, event_queue: &mut EventQueue<Self>) {
        let qh = event_queue.handle();
        loop {
            println!("roundtripping");
            event_queue.roundtrip(self).unwrap();
            match (&self.compositor, &self.layer_shell, &self.surface) {
                (Some(compositor), Some(layer_shell), None) => {
                    let wl_surface = compositor.create_surface(&qh, ());
                    let layer_surface = layer_shell.get_layer_surface(
                        &wl_surface,
                        None,
                        Layer::Overlay,
                        "gtk-layer-shell".into(),
                        &qh,
                        (),
                    );
                    layer_surface.set_anchor(Anchor::all());
                    layer_surface.set_exclusive_zone(-1);
                    layer_surface.set_keyboard_interactivity(KeyboardInteractivity::Exclusive);
                    wl_surface.commit();
                    self.surface = Some(Surface {
                        width: 1,
                        height: 1,
                        wl_surface: wl_surface,
                        buf: allocate_shm_buffer(4),
                        wl_buf: None,
                    });
                    self.layer_surface = Some(layer_surface);
                    println!("Initialized surface");
                }
                _ => {}
            }
            if self
                .surface
                .as_ref()
                .and_then(|s| s.wl_buf.as_ref())
                .is_some()
            {
                self.surface.as_ref().unwrap().wl_surface.attach(
                    Some(&self.surface.as_ref().unwrap().wl_buf.as_ref().unwrap()),
                    0,
                    0,
                );
                self.surface.as_ref().unwrap().wl_surface.commit();

                let (selected_output_name, selected_output) = self
                    .outputs
                    .iter()
                    .find(|(_, output)| match output.flags {
                        Some(WEnum::Value(mode)) => mode.contains(Mode::Current),
                        _ => false,
                    })
                    .unwrap();

                self.selected_output = Some(selected_output_name.clone());

                break;
            };
        }
    }
}

impl Dispatch<WlRegistry, ()> for AppData {
    fn event(
        state: &mut Self,
        registry: &WlRegistry,
        event: <WlRegistry as wayland_client::Proxy>::Event,
        data: &(),
        conn: &wayland_client::Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        dbg!(&event);
        match event {
            wayland_client::protocol::wl_registry::Event::Global {
                name,
                interface,
                version,
            } => match interface.as_str() {
                "wl_compositor" => {
                    let compositor = registry.bind::<WlCompositor, _, _>(name, 2, qhandle, ());
                    state.compositor = Some(compositor);
                }
                "wl_shm" => {
                    state.shm = Some(registry.bind::<WlShm, _, _>(name, version, qhandle, ()));
                }
                "xdg_wm_base" => {
                    state.xdg_wm_base =
                        Some(registry.bind::<XdgWmBase, _, _>(name, version, qhandle, ()));
                }
                "wl_output" => {
                    state.outputs.insert(
                        name,
                        Output::new(registry.bind::<WlOutput, _, _>(name, version, qhandle, ())),
                    );
                    state.adding_output = Some(name);
                }
                "zwlr_layer_shell_v1" => {
                    state.layer_shell =
                        Some(registry.bind::<ZwlrLayerShellV1, _, _>(name, version, qhandle, ()));
                }
                "wl_seat" => {
                    state.seat = Some(registry.bind::<WlSeat, _, _>(name, version, qhandle, ()));
                }
                "zwlr_virtual_pointer_manager_v1" => {
                    let manager = registry.bind::<ZwlrVirtualPointerManagerV1, _, _>(
                        name,
                        version,
                        qhandle,
                        (),
                    );
                    let pointer = manager.create_virtual_pointer(None, qhandle, ());
                    state.pointer = Some(pointer);
                    state.virtual_pointer_manager = Some(manager);
                }
                _ => {}
            },
            _ => {}
        }
    }
}

impl Dispatch<ZwlrVirtualPointerV1, ()> for AppData {
    fn event(
        state: &mut Self,
        proxy: &ZwlrVirtualPointerV1,
        event: <ZwlrVirtualPointerV1 as wayland_client::Proxy>::Event,
        data: &(),
        conn: &wayland_client::Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        dbg!(&event);
    }
}
impl Dispatch<ZwlrVirtualPointerManagerV1, ()> for AppData {
    fn event(
        state: &mut Self,
        proxy: &ZwlrVirtualPointerManagerV1,
        event: <ZwlrVirtualPointerManagerV1 as wayland_client::Proxy>::Event,
        data: &(),
        conn: &wayland_client::Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        dbg!(&event);
    }
}

impl Dispatch<WlSeat, ()> for AppData {
    fn event(
        state: &mut Self,
        seat: &WlSeat,
        event: <WlSeat as wayland_client::Proxy>::Event,
        data: &(),
        conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
        let keyboard = seat.get_keyboard(&qh, ());
        state.keyboard = Some(keyboard);
        dbg!(&event);
    }
}

impl Dispatch<WlKeyboard, ()> for AppData {
    fn event(
        app_state: &mut Self,
        proxy: &WlKeyboard,
        event: <WlKeyboard as wayland_client::Proxy>::Event,
        data: &(),
        conn: &wayland_client::Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        match event {
            wayland_client::protocol::wl_keyboard::Event::Key {
                serial,
                time,
                key,
                state,
            } => {
                if app_state.procesed_keypress_serials.contains(&serial) {
                    return;
                }
                app_state.procesed_keypress_serials.insert(serial);
                //if let WEnum::Value(KeyState::Pressed) = state {}
                //let selector = &app_state.selector;
                //if selector.depth == 3 {
                //    app_state.do_click = true;
                //}
                if state == WEnum::Value(KeyState::Pressed) {
                    app_state.selector.as_mut().unwrap().handle_key(key);
                }
                if key == 1 {
                    panic!("escape pressed");
                }
            }
            _ => {}
        }
    }
}

impl Dispatch<WlCallback, ()> for AppData {
    fn event(
        state: &mut Self,
        proxy: &WlCallback,
        event: <WlCallback as wayland_client::Proxy>::Event,
        data: &(),
        conn: &wayland_client::Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        dbg!(event);
    }
}
impl Dispatch<XdgToplevel, ()> for AppData {
    fn event(
        state: &mut Self,
        proxy: &XdgToplevel,
        event: <XdgToplevel as wayland_client::Proxy>::Event,
        data: &(),
        conn: &wayland_client::Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        dbg!(&event);
        match event {
            xdg_toplevel::Event::Configure {
                width,
                height,
                states,
            } => {
                let surface = state.surface.as_mut().unwrap();
                let width = width.max(1);
                let height = height.max(1);
                surface.init_buf(width as usize, height as usize);
                surface.width = width as usize;
                surface.height = height as usize;
                let buf = &surface.buf;
                let shm = state.shm.as_ref().unwrap();
                let pool = shm.create_pool(buf.fd.as_fd(), buf.len as i32, qhandle, ());
                let wl_buf =
                    pool.create_buffer(0, width, height, width * 4, Format::Argb8888, qhandle, ());
                surface.wl_surface.attach(Some(&wl_buf), 0, 0);
                surface.wl_surface.damage(0, 0, i32::MAX, i32::MAX);
                surface.wl_buf = Some(wl_buf);
                surface.wl_surface.commit();
            }
            xdg_toplevel::Event::Close => {}
            xdg_toplevel::Event::ConfigureBounds { width, height } => {}
            xdg_toplevel::Event::WmCapabilities { capabilities } => {}
            _ => {}
        }
    }
}
impl Dispatch<XdgSurface, ()> for AppData {
    fn event(
        state: &mut Self,
        proxy: &XdgSurface,
        event: <XdgSurface as wayland_client::Proxy>::Event,
        data: &(),
        conn: &wayland_client::Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        dbg!(&event);
        match event {
            wayland_protocols::xdg::shell::client::xdg_surface::Event::Configure { serial } => {
                proxy.ack_configure(serial);
            }
            _ => todo!(),
        }
    }
}
impl Dispatch<XdgWmBase, ()> for AppData {
    fn event(
        state: &mut Self,
        proxy: &XdgWmBase,
        event: <XdgWmBase as wayland_client::Proxy>::Event,
        data: &(),
        conn: &wayland_client::Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        dbg!(&event);
        match event {
            wayland_protocols::xdg::shell::client::xdg_wm_base::Event::Ping { serial } => {
                proxy.pong(serial)
            }
            _ => todo!(),
        }
    }
}
impl Dispatch<WlBuffer, ()> for AppData {
    fn event(
        state: &mut Self,
        proxy: &WlBuffer,
        event: <WlBuffer as wayland_client::Proxy>::Event,
        data: &(),
        conn: &wayland_client::Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        dbg!(event);
    }
}

impl Dispatch<WlShm, ()> for AppData {
    fn event(
        state: &mut Self,
        proxy: &WlShm,
        event: <WlShm as wayland_client::Proxy>::Event,
        data: &(),
        conn: &wayland_client::Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        match event {
            wl_shm::Event::Format { format } => state.formats.push(format),
            _ => todo!(),
        };
    }
}

impl Dispatch<WlCompositor, ()> for AppData {
    fn event(
        state: &mut Self,
        compositor: &WlCompositor,
        event: <WlCompositor as wayland_client::Proxy>::Event,
        data: &(),
        conn: &wayland_client::Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        dbg!(event);
    }
}

impl Dispatch<WlSurface, ()> for AppData {
    fn event(
        state: &mut Self,
        proxy: &WlSurface,
        event: <WlSurface as wayland_client::Proxy>::Event,
        data: &(),
        conn: &wayland_client::Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        dbg!(event);
    }
}

impl Dispatch<WlShmPool, ()> for AppData {
    fn event(
        state: &mut Self,
        proxy: &WlShmPool,
        event: <WlShmPool as wayland_client::Proxy>::Event,
        data: &(),
        conn: &wayland_client::Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        dbg!(event);
    }
}

impl Dispatch<WlOutput, ()> for AppData {
    fn event(
        state: &mut Self,
        proxy: &WlOutput,
        event: <WlOutput as wayland_client::Proxy>::Event,
        data: &(),
        conn: &wayland_client::Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        dbg!(&event);
        let output = state
            .outputs
            .get_mut(&state.adding_output.unwrap())
            .unwrap();
        match event {
            wayland_client::protocol::wl_output::Event::Geometry {
                x,
                y,
                physical_width,
                physical_height,
                subpixel,
                make,
                model,
                transform,
            } => {
                output.x = Some(x);
                output.y = Some(y);
                output.physical_width = Some(physical_width);
                output.physical_height = Some(physical_height);
                output.subpixel = Some(subpixel);
                output.make = Some(make);
                output.model = Some(model);
                output.transform = Some(transform);
            }
            wayland_client::protocol::wl_output::Event::Mode {
                flags,
                width,
                height,
                refresh,
            } => {
                output.flags = Some(flags);
                output.width = Some(width);
                output.height = Some(height);
                output.refresh = Some(refresh);
            }
            wayland_client::protocol::wl_output::Event::Scale { factor } => {
                output.scale = Some(factor);
            }
            wayland_client::protocol::wl_output::Event::Name { name } => {
                output.name = Some(name);
            }
            wayland_client::protocol::wl_output::Event::Description { description } => {
                output.description = Some(description);
            }
            wayland_client::protocol::wl_output::Event::Done => {
                state.adding_output = None;
            }
            _ => todo!(),
        }
    }
}

impl Dispatch<ZwlrLayerShellV1, ()> for AppData {
    fn event(
        state: &mut Self,
        proxy: &ZwlrLayerShellV1,
        event: <ZwlrLayerShellV1 as wayland_client::Proxy>::Event,
        data: &(),
        conn: &wayland_client::Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        dbg!(&event);
    }
}

impl Dispatch<ZwlrLayerSurfaceV1, ()> for AppData {
    fn event(
        state: &mut Self,
        proxy: &ZwlrLayerSurfaceV1,
        event: <ZwlrLayerSurfaceV1 as wayland_client::Proxy>::Event,
        data: &(),
        conn: &wayland_client::Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        dbg!(&event);
        match event {
            zwlr_layer_surface_v1::Event::Configure {
                serial,
                width,
                height,
            } => {
                let surface = state.surface.as_mut().unwrap();
                let width = width.max(1);
                let height = height.max(1);
                surface.init_buf(width as usize, height as usize);
                //surface.width = width as usize;
                surface.width = 1920;
                surface.height = height as usize;
                let buf = &surface.buf;
                let shm = state.shm.as_ref().unwrap();
                let pool = shm.create_pool(buf.fd.as_fd(), buf.len as i32, qhandle, ());
                let wl_buf = pool.create_buffer(
                    0,
                    width as i32,
                    height as i32,
                    (width * 4) as i32,
                    Format::Argb8888,
                    qhandle,
                    (),
                );
                //surface.wl_surface.attach(Some(&wl_buf), 0, 0);
                //surface.wl_surface.damage(0, 0, i32::MAX, i32::MAX);
                surface.wl_buf = Some(wl_buf);
                state.layer_surface.as_ref().unwrap().ack_configure(serial);
            }
            zwlr_layer_surface_v1::Event::Closed => todo!(),
            _ => todo!(),
        }
        //let surface = state.surface.as_mut().unwrap();
        //let width = width.max(1);
        //let height = height.max(1);
        //surface.init_buf(width as usize, height as usize);
        //surface.width = width as usize;
        //surface.height = height as usize;
        //let buf = &surface.buf;
        //let shm = state.shm.as_ref().unwrap();
        //let pool = shm.create_pool(buf.fd.as_fd(), buf.len as i32, qhandle, ());
        //let wl_buf = pool.create_buffer(0, width, height, width * 4, Format::Argb8888, qhandle, ());
        //surface.wl_surface.attach(Some(&wl_buf), 0, 0);
        //surface.wl_surface.damage(0, 0, i32::MAX, i32::MAX);
        //surface.wl_buf = Some(wl_buf);
        //surface.wl_surface.commit();
    }
}
