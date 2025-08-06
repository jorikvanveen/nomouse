use std::thread;
use std::time::Duration;

// Note: This example assumes you have the following dependencies in Cargo.toml:
// wayland-client = "0.31"
// wayland-protocols-wlr = "0.2"
// wayland-protocols = "0.31"

use wayland_client::{
    Connection, Dispatch, QueueHandle,
    protocol::{
        wl_pointer::{Axis, ButtonState},
        wl_registry, wl_seat,
    },
};
use wayland_protocols_wlr::virtual_pointer::v1::client::{
    zwlr_virtual_pointer_manager_v1::{self, ZwlrVirtualPointerManagerV1},
    zwlr_virtual_pointer_v1::{self, ZwlrVirtualPointerV1},
};

struct AppData {
    virtual_pointer_manager: Option<ZwlrVirtualPointerManagerV1>,
    virtual_pointer: Option<ZwlrVirtualPointerV1>,
    seat: Option<wl_seat::WlSeat>,
}

impl AppData {
    fn new() -> Self {
        Self {
            virtual_pointer_manager: None,
            virtual_pointer: None,
            seat: None,
        }
    }
}

// Implement Dispatch for registry events
impl Dispatch<wl_registry::WlRegistry, ()> for AppData {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            match interface.as_str() {
                "zwlr_virtual_pointer_manager_v1" => {
                    println!("Found virtual pointer manager");
                    let manager = registry.bind::<ZwlrVirtualPointerManagerV1, _, _>(
                        name,
                        version.min(2),
                        qh,
                        (),
                    );
                    state.virtual_pointer_manager = Some(manager);
                }
                "wl_seat" => {
                    println!("Found seat");
                    let seat = registry.bind::<wl_seat::WlSeat, _, _>(name, version.min(7), qh, ());
                    state.seat = Some(seat);
                }
                _ => {}
            }
        }
    }
}

// Implement Dispatch for virtual pointer manager
impl Dispatch<ZwlrVirtualPointerManagerV1, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &ZwlrVirtualPointerManagerV1,
        _: zwlr_virtual_pointer_manager_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        // No events for the manager
    }
}

// Implement Dispatch for virtual pointer
impl Dispatch<ZwlrVirtualPointerV1, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &ZwlrVirtualPointerV1,
        event: zwlr_virtual_pointer_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            _ => {}
        }
    }
}

// Implement Dispatch for seat
impl Dispatch<wl_seat::WlSeat, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &wl_seat::WlSeat,
        _: wl_seat::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        // Handle seat events if needed
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting wlr-virtual-pointer example");

    // Connect to the Wayland compositor
    let conn = Connection::connect_to_env()?;
    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();

    // Get the display and registry
    let display = conn.display();
    let registry = display.get_registry(&qh, ());

    let mut app_data = AppData::new();

    // Initial roundtrip to get globals
    event_queue.roundtrip(&mut app_data)?;

    // Check if we have the required interfaces
    let manager = app_data
        .virtual_pointer_manager
        .as_ref()
        .ok_or("Virtual pointer manager not available")?;

    let seat = app_data.seat.as_ref().ok_or("No seat available")?;

    // Create a virtual pointer
    println!("Creating virtual pointer...");
    let virtual_pointer = manager.create_virtual_pointer(Some(seat), &qh, ());
    app_data.virtual_pointer = Some(virtual_pointer.clone());

    // Perform another roundtrip to ensure the virtual pointer is set up
    event_queue.roundtrip(&mut app_data)?;

    println!("Virtual pointer created successfully!");

    // Example 1: Move the pointer in a square pattern
    println!("Moving pointer in a square pattern...");
    let movements = [
        (100.0, 0.0),  // Right
        (0.0, 100.0),  // Down
        (-100.0, 0.0), // Left
        (0.0, -100.0), // Up
    ];

    for (dx, dy) in movements.iter() {
        virtual_pointer.motion(
            0,   // time (0 means current time)
            *dx, // dx as f64
            *dy, // dy as f64
        );
        virtual_pointer.frame();
        event_queue.roundtrip(&mut app_data)?;
        thread::sleep(Duration::from_millis(500));
    }

    // Example 2: Simulate mouse clicks
    println!("Simulating left mouse button click...");

    // Press left button (BTN_LEFT = 0x110)
    virtual_pointer.button(0, 0x110, ButtonState::Pressed);
    virtual_pointer.frame();
    event_queue.roundtrip(&mut app_data)?;
    thread::sleep(Duration::from_millis(100));

    // Release left button
    virtual_pointer.button(0, 0x110, ButtonState::Released);
    virtual_pointer.frame();
    event_queue.roundtrip(&mut app_data)?;
    thread::sleep(Duration::from_millis(100));

    // Example 3: Simulate scroll wheel
    println!("Simulating scroll wheel...");

    // Scroll up (positive value)
    virtual_pointer.axis(0, Axis::VerticalScroll, 10.0);
    virtual_pointer.frame();
    event_queue.roundtrip(&mut app_data)?;
    thread::sleep(Duration::from_millis(200));

    // Scroll down (negative value)
    virtual_pointer.axis(0, Axis::VerticalScroll, -10.0);
    virtual_pointer.frame();
    event_queue.roundtrip(&mut app_data)?;
    thread::sleep(Duration::from_millis(200));

    // Example 4: Absolute positioning (if supported)
    println!("Moving to absolute position...");
    virtual_pointer.motion_absolute(
        0, 500,  // x coordinate
        300,  // y coordinate
        1920, // output width
        1080, // output height
    );
    virtual_pointer.frame();
    event_queue.roundtrip(&mut app_data)?;

    // Example 5: Right click at current position
    println!("Performing right click...");

    // Press right button (BTN_RIGHT = 0x111)
    virtual_pointer.button(0, 0x111, ButtonState::Pressed);
    virtual_pointer.frame();
    event_queue.roundtrip(&mut app_data)?;
    thread::sleep(Duration::from_millis(100));

    // Release right button
    virtual_pointer.button(0, 0x111, ButtonState::Released);
    virtual_pointer.frame();
    event_queue.roundtrip(&mut app_data)?;

    println!("Example completed! Cleaning up...");

    // Clean up
    virtual_pointer.destroy();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_data_creation() {
        let app_data = AppData::new();
        assert!(app_data.virtual_pointer_manager.is_none());
        assert!(app_data.virtual_pointer.is_none());
        assert!(app_data.seat.is_none());
    }
}

// Helper functions for common operations
impl AppData {
    /// Move the virtual pointer by relative amounts
    pub fn move_relative(&self, dx: f64, dy: f64) -> Result<(), &'static str> {
        if let Some(vp) = &self.virtual_pointer {
            vp.motion(0, dx, dy);
            vp.frame();
            Ok(())
        } else {
            Err("Virtual pointer not initialized")
        }
    }

    /// Click a mouse button
    pub fn click_button(&self, button: u32) -> Result<(), &'static str> {
        if let Some(vp) = &self.virtual_pointer {
            // Press
            vp.button(0, button, ButtonState::Pressed);
            vp.frame();

            // Small delay would be needed in real usage
            // thread::sleep(Duration::from_millis(50));

            // Release
            vp.button(0, button, ButtonState::Released);
            vp.frame();
            Ok(())
        } else {
            Err("Virtual pointer not initialized")
        }
    }

    /// Scroll the mouse wheel
    pub fn scroll(&self, direction: f64) -> Result<(), &'static str> {
        if let Some(vp) = &self.virtual_pointer {
            vp.axis(0, Axis::VerticalScroll, direction);
            vp.frame();
            Ok(())
        } else {
            Err("Virtual pointer not initialized")
        }
    }
}

/*
To use this example, add these dependencies to your Cargo.toml:

[dependencies]
wayland-client = "0.31"
wayland-protocols-wlr = "0.2"
wayland-protocols = "0.31"

This example demonstrates:
1. Connecting to a Wayland compositor
2. Binding to the wlr-virtual-pointer protocol
3. Creating a virtual pointer device
4. Sending various pointer events:
   - Relative motion
   - Button presses/releases
   - Scroll wheel events
   - Absolute positioning
5. Proper cleanup

The virtual pointer events are sent to the compositor and will control
the actual system cursor, allowing you to programmatically control
mouse input in Wayland environments that support wlr-virtual-pointer
(like sway, wayfire, etc.).

Note: This requires a compositor that supports the wlr-virtual-pointer
protocol. Most wlroots-based compositors support this.
*/
