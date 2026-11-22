use std::os::fd::AsRawFd;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Winsize {
    ws_row: u16,
    ws_col: u16,
    ws_xpixel: u16,
    ws_ypixel: u16,
}

pub(crate) fn get_terminal_width() -> Option<u16> {
    use libc::{TIOCGWINSZ, ioctl};
    use std::io::IsTerminal;

    // Try stdout first
    if std::io::stdout().is_terminal() {
        let mut ws = Winsize {
            ws_row: 0,
            ws_col: 0,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        if unsafe {
            ioctl(
                std::io::stdout().as_raw_fd(),
                TIOCGWINSZ,
                &mut ws as *mut Winsize,
            )
        } == 0
        {
            return Some(ws.ws_col);
        }
    }

    // Try stderr
    if std::io::stderr().is_terminal() {
        let mut ws = Winsize {
            ws_row: 0,
            ws_col: 0,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        if unsafe {
            ioctl(
                std::io::stderr().as_raw_fd(),
                TIOCGWINSZ,
                &mut ws as *mut Winsize,
            )
        } == 0
        {
            return Some(ws.ws_col);
        }
    }

    None
}

fn is_power_of_two_or_sum(n: u16) -> bool {
    if n == 0 {
        return false;
    }
    // Check if it's a single power of 2
    if (n & (n - 1)) == 0 {
        return true;
    }
    // Check if it's a sum of exactly two powers of 2
    // This means it has exactly 2 bits set
    n.count_ones() == 2
}

pub fn best_fit_width(term_width: u16, offset_width: u16) -> u16 {
    // Available space = terminal width - offset field - separators - ASCII section
    // offset_width + ": " + (hex bytes) + " | " + (ascii)
    // Each byte takes 3 chars in hex (XX + space), 1 in ASCII
    // So: offset_width + 2 + (width * 3) + 3 + width <= term_width
    // offset_width + 5 + (width * 4) <= term_width
    // width <= (term_width - offset_width - 5) / 4

    let available = if term_width > offset_width + 5 {
        (term_width - offset_width - 5) / 4
    } else {
        8 // fallback minimum
    };

    // Find largest valid width <= available
    for width in [64, 48, 32, 24, 16, 12, 8].iter() {
        if *width <= available && is_power_of_two_or_sum(*width) {
            return *width;
        }
    }
    8 // minimum fallback
}

pub fn best_fit_width_interleaved(term_width: u16, offset_width: u16) -> u16 {
    // Available space = terminal width - offset field (offset_width + ": ")
    // Each byte takes 3 characters: "XX " in hex line, " X " / "   " / "\x1b[41m  \x1b[0m " in ascii line
    let overhead = offset_width + 2;
    let available = if term_width > overhead {
        (term_width - overhead) / 3
    } else {
        8
    };

    for width in [64, 48, 32, 24, 16, 12, 8].iter() {
        if *width <= available && is_power_of_two_or_sum(*width) {
            return *width;
        }
    }
    8
}

pub fn print_ascii(bytes: &[u8]) {
    for &b in bytes {
        if b.is_ascii_graphic() || b == b' ' {
            print!("{}", b as char);
        } else {
            // ANSI: red background for non-printable
            print!("\x1b[41m \x1b[0m");
        }
    }
    println!();
}

pub fn print_hex_ascii(bytes: &[u8], start_offset: u64, width: usize, offset_width: usize) {
    // Print chunks with offset field (hexdump style)
    for (i, chunk) in bytes.chunks(width).enumerate() {
        let chunk_offset = start_offset + (i * width) as u64;

        // Print offset field (0-padded hex, no 0x prefix)
        print!("{:0width$x}: ", chunk_offset, width = offset_width);

        // Print hex bytes
        for &b in chunk {
            print!("{:02x} ", b);
        }

        // Padding to align ASCII column
        if chunk.len() < width {
            for _ in chunk.len()..width {
                print!("   ");
            }
        }

        // Separator
        print!("| ");

        // Print ASCII
        for &b in chunk {
            if b.is_ascii_graphic() || b == b' ' {
                print!("{}", b as char);
            } else {
                // ANSI: red background for non-printable
                print!("\x1b[41m \x1b[0m");
            }
        }
        println!();
    }
}

pub fn print_interleaved(bytes: &[u8], start_offset: u64, width: usize, offset_width: usize) {
    for (i, chunk) in bytes.chunks(width).enumerate() {
        let chunk_offset = start_offset + (i * width) as u64;

        // Line 1: Offset and hex bytes
        print!("{:0width$x}: ", chunk_offset, width = offset_width);
        for &b in chunk {
            print!("{:02x} ", b);
        }
        println!();

        // Line 2: Indentation and ASCII / red square
        print!("{:width$}  ", "", width = offset_width);
        for &b in chunk {
            if b.is_ascii_graphic() {
                // Printable graphic character centered in 3-char slot
                print!(" {} ", b as char);
            } else if b == b' ' {
                // Space: 3 spaces
                print!("   ");
            } else {
                // Non-printable: 2-character red square matching hex digits + 1 space
                print!("\x1b[41m  \x1b[0m ");
            }
        }
        println!();
    }
}
