use crate::handle::*;
use crate::*;
use bytes::BytesMut;
use std::mem::size_of;
use windows::{
    core::{Result, PCSTR},
    s,
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::Gdi::{
            BeginPaint, EndPaint, GetDC, GetDeviceCaps, PatBlt, StretchDIBits, BITMAPINFO,
            BITMAPINFOHEADER, BI_RGB, BLACKNESS, DIB_RGB_COLORS, HDC, PAINTSTRUCT, SRCCOPY,
            VREFRESH,
        },
        System::LibraryLoader::GetModuleHandleA,
        UI::Input::KeyboardAndMouse::*,
        UI::WindowsAndMessaging::*,
    },
};

const WINDOW_CLASS_NAME: PCSTR = s!("win32.Window");
const DISPLAY_OFFSET_X: i32 = 10;
const DISPLAY_OFFSET_Y: i32 = 10;
const BUFFER_WIDTH: i32 = 400;
const BUFFER_HEIGHT: i32 = 400;

pub struct Win32OffscreenBuffer {
    // Pixels always are 32-bits wide, Memory Order BB GG RR XX
    info: BITMAPINFO,
    pub bits: BytesMut,
    pub width: i32,
    pub height: i32,
}

impl Win32OffscreenBuffer {
    pub fn new(width: i32, height: i32) -> Result<Self> {
        println!("Win32OffscreenBuffer::new");

        let bytes_per_pixel: i32 = 4;
        let bitmap_memory_size: usize = ((width * height) * bytes_per_pixel).try_into().unwrap();

        let mut buffer = Self {
            info: Default::default(),
            bits: BytesMut::with_capacity(bitmap_memory_size),
            width,
            height,
        };
        buffer.info.bmiHeader.biWidth = width;
        buffer.info.bmiHeader.biHeight = -height; // - sign so origin is top left
        buffer.info.bmiHeader.biPlanes = 1;
        buffer.info.bmiHeader.biBitCount = 32; // 3 bytes for RGB (one each) and one byte for padding cus it needs to be aligned in blocks of 4 bytes
        buffer.info.bmiHeader.biCompression = BI_RGB;
        buffer.info.bmiHeader.biSize = (size_of::<BITMAPINFOHEADER>())
            .try_into()
            .expect("Error computing BITMAPINFOHEADER size");

        //let result = Box::new(buffer);

        Ok(buffer)
    }
}

pub struct Window {
    handle: HWND,
    pub buffer: Win32OffscreenBuffer,
    pub window_running: bool,
    pub refresh_rate: i32,
}

impl Window {
    pub fn new(width: u32, height: u32) -> Result<Box<Self>> {
        println!("Window::new");

        let buffer = Win32OffscreenBuffer::new(BUFFER_WIDTH, BUFFER_HEIGHT)
            .expect("Error allocating win 32 offscreen buffer");

        let instance = unsafe { GetModuleHandleA(None)? };
        let class = WNDCLASSA {
            style: CS_HREDRAW | CS_VREDRAW | CS_OWNDC,
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
            refresh_rate: 60,
        });

        let window = unsafe {
            CreateWindowExA(
                WS_EX_LEFT, // ms: WS_EX_NOREDIRECTIONBITMAP, hmh: 0
                WINDOW_CLASS_NAME,
                &s!("Poong"),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                width as i32,
                height as i32,
                HWND(0),
                HMENU(0),
                instance,
                Some(result.as_mut() as *mut _ as _),
            )
            .ok()? //NOTE(Fermin): Consider removing this trait
        };
        // unsafe { ShowWindow(window, SW_SHOW) };

        result.refresh_rate = unsafe { GetDeviceCaps(GetDC(window), VREFRESH) };

        Ok(result)
    }

    fn get_mouse_position(lparam: LPARAM) -> (isize, isize) {
        let x = lparam.0 & 0xffff;
        let y = (lparam.0 >> 16) & 0xffff;
        (x, y)
    }

    fn win32_display_buffer_in_window(&mut self, device_context: HDC) {
        unsafe {
            let mut client_rect: RECT = Default::default();
            GetClientRect(self.handle, &mut client_rect);
            let window_width = client_rect.right - client_rect.left;
            let window_height = client_rect.bottom - client_rect.top;

            PatBlt(
                device_context,
                0,
                0,
                window_width,
                DISPLAY_OFFSET_Y,
                BLACKNESS,
            );
            PatBlt(
                device_context,
                0,
                0,
                DISPLAY_OFFSET_X,
                window_height,
                BLACKNESS,
            );
            PatBlt(
                device_context,
                DISPLAY_OFFSET_X + self.buffer.width,
                0,
                window_width,
                window_height,
                BLACKNESS,
            );
            PatBlt(
                device_context,
                0,
                DISPLAY_OFFSET_Y + self.buffer.height,
                window_width,
                window_height,
                BLACKNESS,
            );

            StretchDIBits(
                device_context,
                DISPLAY_OFFSET_X,
                DISPLAY_OFFSET_Y,
                self.buffer.width,
                self.buffer.height,
                0,
                0,
                self.buffer.width,
                self.buffer.height,
                Some(self.buffer.bits.as_mut() as *mut _ as _),
                &self.buffer.info,
                DIB_RGB_COLORS,
                SRCCOPY,
            );
        }
    }

    fn win32_process_keyboard_message(new_state: &mut GameButtonState, is_down: bool) {
        if new_state.ended_down != is_down {
            new_state.ended_down = is_down;
            //TODO(Fermin): Half transitions logic
        }
    }

    pub fn win32_process_pending_messages(&mut self, input: &mut GameInput) {
        let mut message: MSG = Default::default();
        unsafe {
            while PeekMessageA(&mut message, HWND(0), 0, 0, PM_REMOVE).into() {
                match message.message {
                    WM_MOUSEMOVE => {
                        let (x, y) = Self::get_mouse_position(message.lParam);
                        input.cursor_pos.x = x as f32 - DISPLAY_OFFSET_X as f32;
                        input.cursor_pos.y = y as f32 - DISPLAY_OFFSET_Y as f32;
                        //println!("cursor x: {}, y: {}", input.cursor_pos.x, input.cursor_pos.y);
                    }
                    // NOTE(Fermin): Consider following the same logic for
                    // mouse button than keyboard buttons
                    WM_LBUTTONDOWN => {
                        input.mouse_buttons[0].ended_down = true;
                        println!("WM_LBUTTONDOWN");
                    }
                    WM_LBUTTONUP => {
                        input.mouse_buttons[0].ended_down = false;
                        println!("WM_LBUTTONUP");
                    }
                    WM_RBUTTONDOWN => {
                        input.mouse_buttons[1].ended_down = true;
                        println!("WM_RBUTTONDOWN");
                    }
                    WM_RBUTTONUP => {
                        input.mouse_buttons[1].ended_down = false;
                        println!("WM_RBUTTONUP");
                    }
                    WM_SYSKEYDOWN | WM_SYSKEYUP | WM_KEYDOWN | WM_KEYUP => {
                        let v_k_code: char = char::from_u32(message.wParam.0 as u32)
                            .expect("Failed to parse VKCode");

                        let was_down = message.lParam.0 & (1 << 30) != 0;
                        let is_down = (message.lParam.0 & (1 << 31)) == 0;
                        let alt_key_was_down = message.lParam.0 & (1 << 29) != 0;
                        //println!("key: {} was_down: {}", v_k_code, was_down);
                        //println!("key: {} is_down: {}", v_k_code, is_down);

                        if was_down != is_down {
                            if v_k_code == 'W' {
                                println!("W");
                                Self::win32_process_keyboard_message(
                                    &mut input.keyboard.buttons.move_up,
                                    is_down,
                                );
                            } else if v_k_code == 'A' {
                                println!("A");
                                Self::win32_process_keyboard_message(
                                    &mut input.keyboard.buttons.move_left,
                                    is_down,
                                );
                            } else if v_k_code == 'S' {
                                println!("S");
                                Self::win32_process_keyboard_message(
                                    &mut input.keyboard.buttons.move_down,
                                    is_down,
                                );
                            } else if v_k_code == 'D' {
                                println!("D");
                                Self::win32_process_keyboard_message(
                                    &mut input.keyboard.buttons.move_right,
                                    is_down,
                                );
                            } else if v_k_code as u16 == VK_ESCAPE.0 {
                                println!("Escape");
                                Self::win32_process_keyboard_message(
                                    &mut input.keyboard.buttons.back,
                                    is_down,
                                );
                            } else if v_k_code as u16 == VK_RETURN.0 {
                                println!("Return");
                                Self::win32_process_keyboard_message(
                                    &mut input.keyboard.buttons.start,
                                    is_down,
                                );
                            } else if v_k_code as u16 == VK_SPACE.0 {
                                println!("Space");
                                Self::win32_process_keyboard_message(
                                    &mut input.keyboard.buttons.jump,
                                    is_down,
                                );
                            }

                            if is_down {
                                if (v_k_code as u16 == VK_F4.0) && alt_key_was_down {
                                    println!("Alt+F4");
                                    self.window_running = false;
                                }
                            }
                        }
                    }
                    _ => {
                        TranslateMessage(&message);
                        DispatchMessageA(&message);
                    }
                }
            }

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
            }
            WM_CLOSE | WM_DESTROY => {
                println!("WM_CLOSE|WN_DESTROY");

                let this = GetWindowLongPtrA(window, GWLP_USERDATA) as *mut Self;
                if let Some(this) = this.as_mut() {
                    this.window_running = false;
                }
            }
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
                    let device_context = BeginPaint(window, &mut paint);
                    this.win32_display_buffer_in_window(device_context);
                    EndPaint(window, &paint);
                }
            }
            _ => (),
        }
        DefWindowProcA(window, message, wparam, lparam)
    }
}
