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
            BITMAPINFO, PatBlt, BLACKNESS, SRCCOPY, DIB_RGB_COLORS,
            StretchDIBits, GetDC
        }
    },
};
use crate::handle::CheckHandle;
use bytes::{ BytesMut, BufMut };

const WINDOW_CLASS_NAME: PCSTR = s!("win32.Window");

pub struct Win32OffscreenBuffer {
    // Pixels always are 32-bits wide, Memory Order BB GG RR XX
    info: BITMAPINFO,
    bits: BytesMut,
    width: i32,
    height: i32,
}

impl Win32OffscreenBuffer {
    pub fn new(width: i32, height: i32) -> Result<Box<Self>> {
        println!("Win32OffscreenBuffer::new");

        let bytes_per_pixel:i32 = 4;
        let bitmap_memory_size: usize = ((width*height)*bytes_per_pixel)
            .try_into()
            .unwrap();

        let mut buffer = Self {
            info: Default::default(),
            bits: BytesMut::with_capacity(bitmap_memory_size),
            width,
            height,
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
    pub window_running: bool,
    cursor_coords: Vector2,
}

impl Window {
    pub fn new(width: u32, height: u32) -> Result<Box<Self>> {
        println!("Window::new");

        let buffer = Win32OffscreenBuffer::new(200, 200)
            .expect("Error allocating win 32 offscreen buffer");

        let instance = unsafe { GetModuleHandleA(None)? };
        let class = WNDCLASSA {
            style: CS_HREDRAW|CS_VREDRAW|CS_OWNDC,
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
            window_running: true,
            cursor_coords: Vector2 { X: (0.0), Y: (0.0) }
        });

        let _window = unsafe {
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

    fn render_gradient(&mut self) {
        let r: i32 = self.cursor_coords.X as i32;
        let g: i32 = self.cursor_coords.Y as i32;
        let b: i32 = 150;
        let pixels_in_buffer: i32 = self.buffer.width * self.buffer.height;

        self.buffer.bits.clear();
        for _ in 0..pixels_in_buffer {
            // NOTE(Fermin): Pixel -> BB GG RR AA
            let color: i32 = (b << 24) | (g << 16) | (r << 8) | 255;
            self.buffer.bits.put_i32(color);
        }
    }

    fn get_mouse_position(lparam: LPARAM) -> (isize, isize) {
        let x = lparam.0 & 0xffff;
        let y = (lparam.0 >> 16) & 0xffff;
        (x, y)
    }

    fn win32_display_buffer_in_window(&mut self, device_context: HDC) {
        let offset_x: i32 = self.cursor_coords.X as i32 - self.buffer.width/2;
        let offset_y: i32 = self.cursor_coords.Y as i32 - self.buffer.height/2;

        unsafe {
            let mut client_rect: RECT = Default::default();
            GetClientRect(self.handle, & mut client_rect);
            let window_width = client_rect.right - client_rect.left;
            let window_height = client_rect.bottom - client_rect.top;

            PatBlt(device_context, 0, 0, window_width, offset_y, BLACKNESS); 
            PatBlt(device_context, 0, 0, offset_x, window_height, BLACKNESS); 
            PatBlt(device_context, offset_x + self.buffer.width, 0, window_width, window_height, BLACKNESS); 
            PatBlt(device_context, 0, offset_y + self.buffer.height, window_width, window_height, BLACKNESS); 

            StretchDIBits(device_context,
                          offset_x, offset_y, self.buffer.width, self.buffer.height,
                          0, 0, self.buffer.width, self.buffer.height,
                          Some(self.buffer.bits.as_mut() as *mut _ as _),
                          &self.buffer.info,
                          DIB_RGB_COLORS, SRCCOPY);
        }
    }


    pub fn win32_process_pending_messages(&mut self) {
        let mut message: MSG = Default::default();
        unsafe { 
            while PeekMessageA(&mut message, HWND(0), 0, 0, PM_REMOVE).into()
            {
                match message.message {
                    WM_MOUSEMOVE => {
                        let (x, y) = Self::get_mouse_position(message.lParam);
                        let point = Vector2 {
                            X: x as f32,
                            Y: y as f32,
                        };
                        self.cursor_coords = point;
                        println!("WM_MOUSEMOVE, x = {}, y = {}", point.X, point.Y);
                    },
                    WM_LBUTTONDOWN => {
                        println!("WM_LBUTTONDOWN");
                    },
                    WM_LBUTTONUP => {
                        println!("WM_LBUTTONUP");
                    },
                    WM_RBUTTONDOWN => {
                        println!("WM_RBUTTONDOWN");
                    },
                    WM_RBUTTONUP => {
                        println!("WM_RBUTTONUP");
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

            self.render_gradient();
            self.win32_display_buffer_in_window(GetDC(self.handle));
        }
    }

    unsafe extern "system" fn win32_main_window_callback(
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match message {
            WM_NCCREATE => {
                println!("CREATE");

                let cs = lparam.0 as *const CREATESTRUCTW;
                let this = (*cs).lpCreateParams as *mut Self;
                (*this).handle = window;

                SetWindowLongPtrA(window, GWLP_USERDATA, this as _);
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

                /* NOTE(Fermin): Not sure if this is needed, will leave in case it is
                let mut client_rect: RECT = Default::default();
                GetClientRect(window, & mut client_rect);
                let width = client_rect.right - client_rect.left;
                let height = client_rect.bottom - client_rect.top;
                */

                let this = GetWindowLongPtrA(window, GWLP_USERDATA) as *mut Self;
                if let Some(this) = this.as_mut() {
                    let mut paint: PAINTSTRUCT = Default::default();
                    let device_context = BeginPaint(window, & mut paint);
                    this.win32_display_buffer_in_window(device_context);
                    EndPaint(window, &paint);
                }
            }
            _ => ()
        }
        DefWindowProcA(window, message, wparam, lparam)
    }
}

