use std::mem::size_of;
use windows::{
    core::{Result, PCSTR },
    s,
    Foundation::Numerics::Vector2,
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM, RECT,
        },
        System::LibraryLoader::GetModuleHandleA,
        UI::WindowsAndMessaging::*,
        Graphics::Gdi::{
            BI_RGB, HDC, BITMAPINFOHEADER, EndPaint, BeginPaint, PAINTSTRUCT,
            BITMAPINFO, PatBlt, BLACKNESS
        }
    },
};
use crate::handle::CheckHandle;

const WINDOW_CLASS_NAME: PCSTR = s!("win32.Window");

pub struct Win32OffscreenBuffer {
    // Pixels always are 32-bits wide, Memory Order BB GG RR XX
    info: BITMAPINFO,
    width: i32,
    height: i32,
    pitch: i32, // pitch is the number of bytes in each row
    bytes_per_pixel: i32,
}

impl Win32OffscreenBuffer {
    pub fn new(width: i32, height: i32) -> Result<Box<Self>> {
        println!("Win32OffscreenBuffer::new");

        let mut buffer = Self {
            info: Default::default(),
            width,
            height,
            bytes_per_pixel: 4,
            pitch: 4 * width,
        };
        buffer.info.bmiHeader.biWidth = width;
        buffer.info.bmiHeader.biHeight = -height;  // - sign so origin is top left
        buffer.info.bmiHeader.biPlanes = 1;
        buffer.info.bmiHeader.biBitCount = 32; // 3 bytes for RGB (one each) and one byte for padding cus it needs to be aligned in blocks of 4 bytes
        buffer.info.bmiHeader.biCompression = BI_RGB;
        buffer.info.bmiHeader.biSize = (size_of::<BITMAPINFOHEADER>())
            .try_into()
            .expect("Error computing BITMAPINFOHEADER size");

        let result = Box::new(buffer);

        Ok(result)
    }
}

pub struct Window {
    handle: HWND,
    buffer: Box<Win32OffscreenBuffer>,
    pub window_running: bool
}

impl Window {
    pub fn new(width: u32, height: u32) -> Result<Box<Self>> {
        println!("Window::new");

        let buffer = Win32OffscreenBuffer::new(width.try_into().unwrap(), height.try_into().unwrap())
            .expect("Error allocating win 32 offscreen buffer");

        let instance = unsafe { GetModuleHandleA(None)? };
        let class = WNDCLASSA {
            style: CS_HREDRAW|CS_VREDRAW,
            hCursor: unsafe { LoadCursorW(HINSTANCE(0), IDC_ARROW).ok().unwrap() },
            hInstance: instance,
            lpszClassName: WINDOW_CLASS_NAME,
            lpfnWndProc: Some(Self::win32_main_window_callback),
            ..Default::default()
        };
        assert_ne!(unsafe { RegisterClassA(&class) }, 0);

        let mut result = Box::new(Self {
            handle: HWND(0),
            buffer,
            window_running: true
        });

        let window = unsafe {
            CreateWindowExA(
                WS_EX_LEFT, // ms: WS_EX_NOREDIRECTIONBITMAP, hmh: 0
                WINDOW_CLASS_NAME,
                &s!("win32_window_binch"),
                WS_OVERLAPPEDWINDOW|WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                width as i32,
                height as i32,
                HWND(0),
                HMENU(0),
                instance,
                Some(result.as_mut() as *mut _ as _)
                )
                .ok()?
        };
        // unsafe { ShowWindow(window, SW_SHOW) };
        
        Ok(result)
    }

    pub fn win32_display_buffer_in_window(
        device_context: HDC,
        window_width: i32,
        window_height: i32 
    ) {
        println!("win32_display_buffer_in_window");
        unsafe { PatBlt(device_context, 0, 0, window_width, window_height, BLACKNESS); }
    }

    pub fn win32_process_pending_messages(&mut self) {
        let mut message: MSG = Default::default();
        unsafe { 
            while PeekMessageA(&mut message, HWND(0), 0, 0, PM_REMOVE).into()
            {
                match message.message {
                    WM_MOUSEMOVE => {
                        let (x, y) = get_mouse_position(message.lParam);
                        let point = Vector2 {
                            X: x as f32,
                            Y: y as f32,
                        };
                        println!("WM_MOUSEMOVE, x = {}, y = {}", point.X, point.Y);
                    },
                    WM_SYSKEYDOWN => println!("WM_SYSKEYDOWN"),
                    WM_SYSKEYUP => println!("WM_SYSKEYUP"),
                    WM_KEYDOWN => println!("WM_KEYDOWN"),
                    WM_KEYUP => println!("WM_KEYUP"),
                    _ => {
                        TranslateMessage(&message);
                        DispatchMessageA(&message);
                    }
                }
            }
        }
    }

    unsafe extern "system" fn win32_main_window_callback(
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
        ) -> LRESULT {
        let mut result:LRESULT = windows::Win32::Foundation::LRESULT(0);
        match message {
            WM_NCCREATE => {
                println!("CREATE");

                let cs = lparam.0 as *const CREATESTRUCTW;
                let this = (*cs).lpCreateParams as *mut Self;
                (*this).handle = window;

                SetWindowLongPtrA(window, GWLP_USERDATA, this as _);

                result = DefWindowProcA(window, message, wparam, lparam)
            },
            WM_CLOSE | WM_DESTROY => {
                println!("WM_CLOSE|WN_DESTROY");

                let this = GetWindowLongPtrA(window, GWLP_USERDATA) as *mut Self;
                if let Some(this) = this.as_mut() {
                    this.window_running = false;
                }
            },
            WM_SYSKEYDOWN => println!("Keyboard input came in through a non-dispatch message"),
            WM_SYSKEYUP => println!("Keyboard input came in through a non-dispatch message"),
            WM_KEYDOWN => println!("Keyboard input came in through a non-dispatch message"),
            WM_KEYUP => println!("Keyboard input came in through a non-dispatch message"),
            WM_PAINT => {
                println!("WM_PAINT");

                let mut paint: PAINTSTRUCT = Default::default();
                let device_context = BeginPaint(window, & mut paint);
                let mut client_rect: RECT = Default::default();
                GetClientRect(window, & mut client_rect);
                let width = client_rect.right - client_rect.left;
                let height = client_rect.bottom - client_rect.top;
                Self::win32_display_buffer_in_window(device_context, width, height);
                EndPaint(window, &paint);
            }
            other => {
                result = DefWindowProcA(window, other, wparam, lparam)
            }
        }

        result
    }
}

fn get_mouse_position(lparam: LPARAM) -> (isize, isize) {
    let x = lparam.0 & 0xffff;
    let y = (lparam.0 >> 16) & 0xffff;
    (x, y)
}
