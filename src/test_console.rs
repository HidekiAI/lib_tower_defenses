


use std::io::Error;
use std::mem;

#[cfg(target_os = "windows")]
extern crate winapi;

#[cfg(target_os = "windows")]
use winapi::um::wincon::{GetConsoleScreenBufferInfo, CONSOLE_SCREEN_BUFFER_INFO};
#[cfg(target_os = "windows")]
use winapi::um::processenv::GetStdHandle;
#[cfg(target_os = "windows")]
use winapi::um::winbase::STD_OUTPUT_HANDLE;

#[cfg(target_os = "linux")]
extern crate libc;

#[cfg(target_os = "linux")]
use libc::{ioctl, winsize, STDOUT_FILENO, TIOCGWINSZ};

fn main() {
    let result = get_terminal_size();
    match result {
        Ok((columns, rows)) => {
            println!("columns: {}", columns);
            println!("rows: {}", rows);
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn get_terminal_size() -> Result<(u16, u16), Error> {
    #[cfg(target_os = "windows")]
    {
        let stdout = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) };
        if stdout.is_null() {
            return Err(Error::last_os_error());
        }

        let mut csbi: CONSOLE_SCREEN_BUFFER_INFO = unsafe { mem::zeroed() };
        if unsafe { GetConsoleScreenBufferInfo(stdout, &mut csbi) } == 0 {
            return Err(Error::last_os_error());
        }

        let columns = (csbi.srWindow.Right - csbi.srWindow.Left + 1) as u16;
        let rows = (csbi.srWindow.Bottom - csbi.srWindow.Top + 1) as u16;

        Ok((columns, rows))
    }

    #[cfg(target_os = "linux")]
    {
        let mut size: winsize = unsafe { mem::zeroed() };
        if unsafe { ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut size) } == -1 {
            return Err(Error::last_os_error());
        }

        Ok((size.ws_col as u16, size.ws_row as u16))
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        Err(Error::new(
            ErrorKind::Other,
            "Unsupported platform",
        ))
    }
}
