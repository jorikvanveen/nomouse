# wlr-virtual-pointer Rust Example

A comprehensive Rust example demonstrating how to use the `wlr-virtual-pointer` protocol to programmatically control mouse input in Wayland compositors.

## Overview

This project shows how to:
- Connect to a Wayland compositor
- Bind to the wlr-virtual-pointer protocol
- Create a virtual pointer device
- Send various pointer events (motion, clicks, scrolling)
- Properly manage the virtual pointer lifecycle

## Requirements

### System Requirements
- A Wayland compositor that supports the `wlr-virtual-pointer` protocol
- Linux system with Wayland development libraries

### Supported Compositors
This example works with compositors that implement the wlr-virtual-pointer protocol, including:
- **Sway** - Full support
- **Wayfire** - Full support
- **River** - Full support
- **Hyprland** - Full support
- **Most wlroots-based compositors**

**Note:** This will NOT work with:
- GNOME Wayland (uses different protocols)
- KDE Plasma Wayland (limited support)
- X11 environments

### Dependencies
The project uses these Rust crates:
- `wayland-client` - Core Wayland client library
- `wayland-protocols-wlr` - wlroots-specific protocol bindings
- `wayland-protocols` - Standard Wayland protocols

## Building and Running

### 1. Clone and Build
```bash
cd nomouse
cargo build
```

### 2. Run the Example
```bash
# Make sure you're in a Wayland session with a supported compositor
cargo run
```

### 3. Expected Output
When running successfully, you should see:
```
Starting wlr-virtual-pointer example
Found virtual pointer manager
Found seat
Creating virtual pointer...
Virtual pointer created successfully!
Moving pointer in a square pattern...
Simulating left mouse button click...
Simulating scroll wheel...
Moving to absolute position...
Performing right click...
Example completed! Cleaning up...
```

## What the Example Demonstrates

### 1. Relative Mouse Movement
```rust
// Move mouse by relative amounts
virtual_pointer.motion(0, (dx * 256.0) as i32, (dy * 256.0) as i32);
```

### 2. Mouse Button Clicks
```rust
// Left click (BTN_LEFT = 0x110)
virtual_pointer.button(0, 0x110, ButtonState::Pressed);
virtual_pointer.button(0, 0x110, ButtonState::Released);
```

### 3. Scroll Wheel Events
```rust
// Vertical scrolling
virtual_pointer.axis(0, Axis::VerticalScroll, 10.0 * 256.0);
```

### 4. Absolute Positioning
```rust
// Move to specific screen coordinates
virtual_pointer.motion_absolute(0, x * 256, y * 256, screen_width, screen_height);
```

## Common Mouse Button Codes

| Button | Code | Hex |
|--------|------|-----|
| Left | 272 | 0x110 |
| Right | 273 | 0x111 |
| Middle | 274 | 0x112 |
| Back | 275 | 0x113 |
| Forward | 276 | 0x114 |

## Coordinate System

### Fixed-Point Coordinates
Wayland uses fixed-point arithmetic for coordinates:
- Multiply by 256 to convert from float to fixed-point
- Coordinates are in compositor space (pixels)

### Relative vs Absolute Motion
- **Relative**: `motion()` - moves relative to current position
- **Absolute**: `motion_absolute()` - moves to specific screen coordinates

## Helper Functions

The example includes utility functions for common operations:

```rust
// Move mouse relatively
app_data.move_relative(50.0, -25.0)?;

// Click a button
app_data.click_button(0x110)?; // Left click

// Scroll wheel
app_data.scroll(5.0)?; // Scroll up
```

## Troubleshooting

### "Virtual pointer manager not available"
- Your compositor doesn't support wlr-virtual-pointer
- Try a wlroots-based compositor like Sway
- Check if you're running under Wayland (not X11)

### Permission Issues
Some compositors may require special permissions for virtual input devices. Check your compositor's documentation.

### Build Errors
Make sure you have Wayland development libraries installed:

**Ubuntu/Debian:**
```bash
sudo apt install libwayland-dev wayland-protocols
```

**Fedora:**
```bash
sudo dnf install wayland-devel wayland-protocols-devel
```

**Arch:**
```bash
sudo pacman -S wayland wayland-protocols
```

## Security Considerations

Virtual pointer protocols can be powerful and potentially dangerous:
- Only run trusted code that uses virtual pointers
- Some compositors may restrict virtual input for security
- Consider implementing rate limiting in production applications

## Advanced Usage

For more complex applications, consider:
- Event timing and synchronization
- Multiple virtual pointer devices
- Integration with other Wayland protocols
- Error handling and reconnection logic

## References

- [wlr-protocols](https://gitlab.freedesktop.org/wlroots/wlr-protocols) - Protocol specifications
- [wayland-rs](https://github.com/Smithay/wayland-rs) - Rust Wayland bindings
- [Wayland Book](https://wayland-book.com/) - Comprehensive Wayland guide

## License

This example is provided for educational purposes. Check the licenses of the dependencies for usage in your projects.