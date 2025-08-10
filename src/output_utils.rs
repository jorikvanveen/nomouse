use wayland_client::{
    WEnum,
    protocol::{
        wl_buffer::WlBuffer,
        wl_output::{Mode, Subpixel, Transform, WlOutput},
    },
};

//wayland_client::protocol::wl_output::Event::Geometry {
//    x,
//    y,
//    physical_width,
//    physical_height,
//    subpixel,
//    make,
//    model,
//    transform,
//} => todo!(),
//wayland_client::protocol::wl_output::Event::Mode {
//    flags,
//    width,
//    height,
//    refresh,
//} => todo!(),
//wayland_client::protocol::wl_output::Event::Done => todo!(),
//wayland_client::protocol::wl_output::Event::Scale { factor } => todo!(),
//wayland_client::protocol::wl_output::Event::Name { name } => todo!(),
//wayland_client::protocol::wl_output::Event::Description { description } => todo!(),
#[derive(Debug)]
pub struct Output {
    pub wl_output: WlOutput,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub physical_width: Option<i32>,
    pub physical_height: Option<i32>,
    pub subpixel: Option<WEnum<Subpixel>>,
    pub make: Option<String>,
    pub model: Option<String>,
    pub transform: Option<WEnum<Transform>>,
    pub flags: Option<WEnum<Mode>>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub refresh: Option<i32>,
    pub scale: Option<i32>,
    pub name: Option<String>,
    pub description: Option<String>,
}

impl Output {
    pub fn new(wl_output: WlOutput) -> Self {
        Self {
            wl_output,
            x: None,
            y: None,
            physical_width: None,
            physical_height: None,
            subpixel: None,
            make: None,
            model: None,
            transform: None,
            flags: None,
            width: None,
            height: None,
            refresh: None,
            scale: None,
            name: None,
            description: None,
        }
    }
}
