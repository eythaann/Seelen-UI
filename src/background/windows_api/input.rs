use std::cmp::max;
use std::cmp::min;
use std::mem;
use std::str::Chars;
use std::thread::sleep;
use std::time::Duration;

use phf::phf_map;
use phf::phf_set;
use seelen_core::Point;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
use windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics;
use windows::Win32::UI::WindowsAndMessaging::SetCursorPos;
use windows::Win32::UI::WindowsAndMessaging::SM_CXSCREEN;
use windows::Win32::UI::WindowsAndMessaging::SM_CYSCREEN;

use crate::error::Result;

const VIRTUAL_KEYS: phf::Map<&'static str, VIRTUAL_KEY> = phf_map! {
    "CONTROL" => VK_CONTROL, "CTRL" => VK_CONTROL, "LCONTROL" => VK_LCONTROL, "LCTRL" => VK_LCONTROL, "RCONTROL" => VK_RCONTROL, "RCTRL" => VK_RCONTROL,
    "ALT" => VK_MENU, "MENU" => VK_MENU, "LALT" => VK_LMENU, "LMENU" => VK_LMENU, "RALT" => VK_RMENU, "RMENU" => VK_RMENU,
    "SHIFT" => VK_SHIFT, "LSHIFT" => VK_LSHIFT, "RSHIFT" => VK_RSHIFT, "APPS" => VK_APPS,
    "WIN" => VK_LWIN, "WINDOWS" => VK_LWIN, "LWIN" => VK_LWIN, "LWINDOWS" => VK_LWIN, "RWIN" => VK_RWIN, "RWINDOWS" => VK_RWIN,
    "LBUTTON" => VK_LBUTTON, "RBUTTON" => VK_RBUTTON, "MBUTTON" => VK_MBUTTON, "XBUTTON1" => VK_XBUTTON1, "XBUTTON2" => VK_XBUTTON2,
    "CANCEL" => VK_CANCEL, "BACK" => VK_BACK, "TAB" => VK_TAB, "RETURN" => VK_RETURN, "ENTER" => VK_RETURN, "PAUSE" => VK_PAUSE, "CAPITAL" => VK_CAPITAL,
    "ESCAPE" => VK_ESCAPE, "ESC" => VK_ESCAPE, "SPACE" => VK_SPACE,
    "PRIOR" => VK_PRIOR, "PAGE_UP" => VK_PRIOR, "NEXT" => VK_NEXT, "PAGE_DOWN" => VK_NEXT, "HOME" => VK_HOME, "END" => VK_END,
    "LEFT" => VK_LEFT, "UP" => VK_UP, "RIGHT" => VK_RIGHT, "DOWN" => VK_DOWN, "PRINT" => VK_PRINT,
    "INSERT" => VK_INSERT, "DELETE" => VK_DELETE,
    "F1" => VK_F1, "F2" => VK_F2, "F3" => VK_F3, "F4" => VK_F4, "F5" => VK_F5, "F6" => VK_F6, "F7" => VK_F7, "F8" => VK_F8, "F9" => VK_F9, "F10" => VK_F10,
    "F11" => VK_F11, "F12" => VK_F12, "F13" => VK_F13, "F14" => VK_F14, "F15" => VK_F15, "F16" => VK_F16, "F17" => VK_F17, "F18" => VK_F18, "F19" => VK_F19,
    "F20" => VK_F20, "F21" => VK_F21, "F22" => VK_F22, "F23" => VK_F23, "F24" => VK_F24,
};

const HOLD_KEYS: phf::Set<&'static str> = phf_set! {
    "CONTROL", "CTRL", "LCONTROL", "LCTRL", "RCONTROL", "RCTRL",
    "ALT", "MENU", "LALT", "LMENU", "RALT", "RMENU",
    "SHIFT", "LSHIFT", "RSHIFT", "APPS",
    "WIN", "WINDOWS", "LWIN", "LWINDOWS", "RWIN", "RWINDOWS"
};

const KEYEVENTF_KEYDOWN: KEYBD_EVENT_FLAGS = KEYBD_EVENT_FLAGS(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputItem {
    HoldKey(VIRTUAL_KEY),
    VirtualKey(VIRTUAL_KEY),
    Character(char),
}

impl InputItem {
    fn is_holdkey(&self) -> bool {
        matches!(self, Self::HoldKey(_))
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Input {
    holdkeys: Vec<VIRTUAL_KEY>,
    items: Vec<InputItem>,
}

impl Input {
    fn new() -> Self {
        Self {
            holdkeys: Vec::new(),
            items: Vec::new(),
        }
    }

    fn has_holdkey(&self) -> bool {
        !self.holdkeys.is_empty()
    }

    fn has_items(&self) -> bool {
        !self.items.is_empty()
    }

    fn is_holdkey_only(&self) -> bool {
        !self.holdkeys.is_empty() && self.items.is_empty()
    }

    fn push(&mut self, item: InputItem) {
        if let InputItem::HoldKey(key) = item {
            if !self.holdkeys.contains(&key) {
                self.holdkeys.push(key);
            }
        } else {
            self.items.push(item);
        }
    }

    fn push_all(&mut self, items: &Vec<InputItem>) {
        for item in items {
            self.push(*item);
        }
    }

    fn create_inputs(&self) -> Result<Vec<INPUT>> {
        let mut inputs: Vec<INPUT> = Vec::new();

        for holdkey in &self.holdkeys {
            let input = Self::create_virtual_key(*holdkey, KEYEVENTF_KEYDOWN);
            inputs.push(input);
        }

        for item in &self.items {
            match item {
                InputItem::VirtualKey(key) => {
                    inputs.push(Self::create_virtual_key(*key, KEYEVENTF_KEYDOWN));
                    inputs.push(Self::create_virtual_key(*key, KEYEVENTF_KEYUP));
                }
                InputItem::Character(ch) => {
                    let mut buffer = [0; 2];
                    let chars = ch.encode_utf16(&mut buffer);
                    for ch_u16 in chars {
                        let keys = Self::create_char_key(*ch_u16, self.has_holdkey());
                        inputs.extend(keys);
                    }
                }
                _ => (),
            }
        }

        for holdkey in self.holdkeys.iter().rev() {
            let input = Self::create_virtual_key(*holdkey, KEYEVENTF_KEYUP);
            inputs.push(input);
        }

        Ok(inputs)
    }

    fn create_virtual_key(key: VIRTUAL_KEY, flags: KEYBD_EVENT_FLAGS) -> INPUT {
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: key,
                    wScan: unsafe { MapVirtualKeyW(u32::from(key.0), MAPVK_VK_TO_VSC) } as u16,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }
    }

    fn create_char_key(ch: u16, hold_mode: bool) -> Vec<INPUT> {
        // let code = ch as i32;
        let vk: i16 = if ch < 256 {
            unsafe { VkKeyScanW(ch) }
        } else {
            -1
        };

        if vk == -1 {
            // Unicode
            vec![
                INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VIRTUAL_KEY(0),
                            wScan: ch,
                            dwFlags: KEYEVENTF_UNICODE,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                },
                INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VIRTUAL_KEY(0),
                            wScan: ch,
                            dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                },
            ]
        } else {
            // ASCII
            let key: VIRTUAL_KEY = VIRTUAL_KEY((vk & 0xFF) as _);
            if hold_mode {
                vec![
                    INPUT {
                        r#type: INPUT_KEYBOARD,
                        Anonymous: INPUT_0 {
                            ki: KEYBDINPUT {
                                wVk: key,
                                wScan: 0,
                                dwFlags: KEYEVENTF_KEYDOWN,
                                time: 0,
                                dwExtraInfo: 0,
                            },
                        },
                    },
                    INPUT {
                        r#type: INPUT_KEYBOARD,
                        Anonymous: INPUT_0 {
                            ki: KEYBDINPUT {
                                wVk: key,
                                wScan: 0,
                                dwFlags: KEYEVENTF_KEYUP,
                                time: 0,
                                dwExtraInfo: 0,
                            },
                        },
                    },
                ]
            } else {
                let mut shift: bool = (vk >> 8 & 0x01) != 0;
                let state = unsafe { GetKeyState(VK_CAPITAL.0 as _) };
                if (state & 0x01) != 0
                    && ((ch >= 'a' as u16 && ch <= 'z' as u16)
                        || (ch >= 'A' as u16 && ch <= 'Z' as u16))
                {
                    shift = !shift;
                };
                let mut char_inputs: Vec<INPUT> = Vec::new();
                if shift {
                    char_inputs.push(INPUT {
                        r#type: INPUT_KEYBOARD,
                        Anonymous: INPUT_0 {
                            ki: KEYBDINPUT {
                                wVk: VK_SHIFT,
                                wScan: 0,
                                dwFlags: KEYEVENTF_KEYDOWN,
                                time: 0,
                                dwExtraInfo: 0,
                            },
                        },
                    });
                }
                char_inputs.push(INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: key,
                            wScan: 0,
                            dwFlags: KEYEVENTF_KEYDOWN,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                });
                char_inputs.push(INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: key,
                            wScan: 0,
                            dwFlags: KEYEVENTF_KEYUP,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                });
                if shift {
                    char_inputs.push(INPUT {
                        r#type: INPUT_KEYBOARD,
                        Anonymous: INPUT_0 {
                            ki: KEYBDINPUT {
                                wVk: VK_SHIFT,
                                wScan: 0,
                                dwFlags: KEYEVENTF_KEYUP,
                                time: 0,
                                dwExtraInfo: 0,
                            },
                        },
                    });
                };
                char_inputs
            }
        }
    }
}

fn parse_input(expression: &str) -> Result<Vec<Input>> {
    let mut inputs: Vec<Input> = Vec::new();

    let mut expr = expression.chars();
    while let Some((items, is_holdkey)) = next_input(&mut expr)? {
        if let Some(prev) = inputs.last_mut() {
            // if !is_holdkey && (prev.is_holdkey_only() || !prev.has_holdkey()) {
            if (is_holdkey && !prev.has_items())
                || (!is_holdkey && (!prev.has_holdkey() || prev.is_holdkey_only()))
            {
                prev.push_all(&items);
                continue;
            }
        }

        let mut input = Input::new();
        input.push_all(&items);

        inputs.push(input);
    }

    Ok(inputs)
}

fn next_input(expr: &mut Chars<'_>) -> Result<Option<(Vec<InputItem>, bool)>> {
    if let Some(ch) = expr.next() {
        let next = match ch {
            '{' => {
                let item = read_special_item(expr)?;
                Some((vec![item], item.is_holdkey()))
            }
            '(' => {
                let items = read_group_items(expr)?;
                Some((items, false))
            }
            _ => Some((vec![InputItem::Character(ch)], false)),
        };
        Ok(next)
    } else {
        Ok(None)
    }
}

fn read_special_item(expr: &mut Chars<'_>) -> Result<InputItem> {
    let mut token = String::new();
    let mut matched = false;
    for ch in expr.by_ref() {
        if ch == '}' && !token.is_empty() {
            matched = true;
            break;
        } else {
            token.push(ch);
        }
    }

    if matched {
        if token == "(" || token == ")" || token == "{" || token == "}" {
            Ok(InputItem::Character(token.chars().next().unwrap()))
        } else {
            let token = token.to_uppercase();
            if let Some(key) = VIRTUAL_KEYS.get(&token) {
                if HOLD_KEYS.contains(&token) {
                    Ok(InputItem::HoldKey(*key))
                } else {
                    Ok(InputItem::VirtualKey(*key))
                }
            } else {
                Err("Error Input Format".into())
            }
        }
    } else {
        Err("Error Input Format".into())
    }
}

fn read_group_items(expr: &mut Chars<'_>) -> Result<Vec<InputItem>> {
    let mut items: Vec<InputItem> = Vec::new();
    let mut matched = false;

    while let Some((next, _)) = next_input(expr)? {
        if next.len() == 1 && next[0] == InputItem::Character(')') {
            matched = true;
            break;
        }

        items.extend(next);
    }

    if matched {
        Ok(items)
    } else {
        Err("Error Input Format".into())
    }
}

/// Simulate typing keys on keyboard.
#[derive(Debug, Default)]
pub struct Keyboard {
    interval: u64,
    holdkeys: Vec<VIRTUAL_KEY>,
}

#[allow(dead_code)]
impl Keyboard {
    /// Create a keyboard to simulate typing keys.
    pub fn new() -> Self {
        Self {
            interval: 0,
            holdkeys: Vec::new(),
        }
    }

    /// Set the interval time between keys.
    ///
    /// `interval` is the time number of milliseconds, `0` is default value.
    pub fn interval(mut self, interval: u64) -> Self {
        self.interval = interval;
        self
    }

    /// Simulates typing `keys` on keyboard.
    ///
    /// `{}` is used for some special keys. For example: `{ctrl}{alt}{delete}`, `{shift}{home}`.
    ///
    /// `()` is used for group keys. For example: `{ctrl}(AB)` types `Ctrl+A+B`.
    ///
    /// `{` `}` `(` `)` can be quoted by `{}`. For example: `{{}Hi,{(}rust!{)}{}}` types `{Hi,(rust)}`.
    pub fn send_keys(&self, keys: &str) -> Result<()> {
        let inputs = parse_input(keys)?;
        for ref input in inputs {
            // self.send_keyboard(input)?;
            let input_keys = input.create_inputs()?;
            self.send_keyboard(&input_keys)?;
        }

        Ok(())
    }

    /// Simulates starting to hold `keys` on keyboard. Only holdkeys are allowed.
    ///
    /// The `keys` will be released when `end_hold_keys()` is invoked.
    pub fn begin_hold_keys(&mut self, keys: &str) -> Result<()> {
        let mut holdkeys: Vec<VIRTUAL_KEY> = Vec::new();

        let inputs = parse_input(keys)?;
        for input in inputs {
            if input.has_items() {
                return Err("Error holdkeys".into());
            }

            holdkeys.extend(input.holdkeys);
        }

        if holdkeys.is_empty() {
            return Err("Error holdkeys".into());
        }

        let mut holdkey_inputs: Vec<INPUT> = Vec::new();
        for holdkey in &holdkeys {
            holdkey_inputs.push(Input::create_virtual_key(*holdkey, KEYEVENTF_KEYDOWN));
        }
        // send_input(&holdkey_inputs.as_slice())?;
        self.send_keyboard(&holdkey_inputs)?;

        self.holdkeys.extend(holdkeys);

        Ok(())
    }

    /// Stop holding keys on keyboard.
    pub fn end_hold_keys(&mut self) -> Result<()> {
        if self.holdkeys.is_empty() {
            Ok(())
        } else {
            let mut holdkey_inputs = Vec::new();
            for holdkey in self.holdkeys.iter().rev() {
                holdkey_inputs.push(Input::create_virtual_key(*holdkey, KEYEVENTF_KEYUP));
            }
            self.holdkeys.clear();

            // send_input(&holdkey_inputs.as_slice())
            self.send_keyboard(&holdkey_inputs)
        }
    }

    // fn send_keyboard(&self, input: &Input) -> Result<()> {
    //     let input_keys = input.create_inputs()?;
    //     if self.interval == 0 {
    //         send_input(&input_keys.as_slice())
    //     } else {
    //         for input_key in &input_keys {
    //             let input_key_slice: [INPUT; 1] = [input_key.clone()];
    //             send_input(&input_key_slice)?;

    //             self.wait();
    //         }

    //         Ok(())
    //     }
    // }

    fn send_keyboard(&self, input_keys: &[INPUT]) -> Result<()> {
        // let input_keys = input.create_inputs()?;
        if self.interval == 0 {
            send_input(input_keys)
        } else {
            for input_key in input_keys {
                let input_key_slice: [INPUT; 1] = [*input_key];
                send_input(&input_key_slice)?;

                self.wait();
            }

            Ok(())
        }
    }

    fn wait(&self) {
        if self.interval > 0 {
            sleep(Duration::from_millis(self.interval));
        }
    }
}

impl Drop for Keyboard {
    fn drop(&mut self) {
        if !self.holdkeys.is_empty() {
            let mut holdkey_inputs: Vec<INPUT> = Vec::new();
            for holdkey in self.holdkeys.iter().rev() {
                holdkey_inputs.push(Input::create_virtual_key(*holdkey, KEYEVENTF_KEYUP));
            }

            if send_input(holdkey_inputs.as_slice()).is_ok() {
                self.holdkeys.clear();
            }
        }
    }
}

/// Simulate mouse event.
#[derive(Debug)]
pub struct Mouse {
    interval: u64,
    move_time: u64,
    auto_move: bool,
    holdkeys: Vec<VIRTUAL_KEY>,
}

impl Default for Mouse {
    fn default() -> Self {
        Self {
            interval: 100,
            move_time: 500,
            auto_move: true,
            holdkeys: Vec::new(),
        }
    }
}

#[allow(dead_code)]
impl Mouse {
    /// Creates a `Mouse` to simulate mouse event.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the interval time between events.
    ///
    /// `interval` is the time number of milliseconds, `100` is default value.
    pub fn interval(mut self, interval: u64) -> Self {
        self.interval = interval;
        self
    }

    /// Sets the mouse move time in millionseconds. `500` is default value.
    pub fn move_time(mut self, move_time: u64) -> Self {
        self.move_time = move_time;
        self
    }

    /// Sets whether move the cursor to the click point automatically. Default is `true`.
    pub fn auto_move(mut self, auto_move: bool) -> Self {
        self.auto_move = auto_move;
        self
    }

    /// Sets the holdkeys when mouse clicks.
    ///
    /// The holdkeys is quoted by `{}`. For example: `{Shift}`, `{Ctrl}{Alt}`.
    pub fn holdkeys(mut self, holdkeys: &str) -> Self {
        self.holdkeys.clear();

        let mut expr = holdkeys.chars();
        while let Some((items, is_holdkey)) = next_input(&mut expr).unwrap() {
            if is_holdkey {
                for item in items {
                    if let InputItem::HoldKey(key) = item {
                        self.holdkeys.push(key);
                    }
                }
            }
        }

        self
    }

    /// Retrieves the position of the mouse cursor, in screen coordinates.
    pub fn get_cursor_pos() -> Result<Point> {
        let mut pos = windows::Win32::Foundation::POINT::default();
        unsafe { GetCursorPos(&mut pos)? };
        Ok(Point::new(pos.x, pos.y))
    }

    /// Moves the cursor to the specified screen coordinates.
    pub fn set_cursor_pos(pos: Point) -> Result<()> {
        unsafe { SetCursorPos(pos.x, pos.y)? };
        Ok(())
    }

    /// Moves the cursor from current position to the `target` position.
    ///
    /// # Examples
    ///
    /// ```
    /// use uiautomation::inputs::Mouse;
    /// use uiautomation::types::Point;
    ///
    /// let mouse = Mouse::new().move_time(800);
    /// mouse.move_to(Point::new(10, 20)).unwrap();
    /// mouse.move_to(Point::new(1000,800)).unwrap();
    /// ```
    pub fn move_to(&self, target: &Point) -> Result<()> {
        let (width, height) = get_screen_size()?;
        let x = min(max(0, target.x), width);
        let y = min(max(0, target.y), height);
        let target = Point::new(x, y);

        if self.move_time > 0 {
            let source = Self::get_cursor_pos()?;
            let delta_x = target.x - source.x;
            let delta_y = target.y - source.y;

            let delta = max(delta_x.abs(), delta_y.abs());
            let steps = delta / 20;
            if steps > 1 {
                let step_x = delta_x / steps;
                let step_y = delta_y / steps;
                let interval = Duration::from_millis(self.move_time / steps as u64);
                for i in 1..steps {
                    let pos = Point::new(source.x + step_x * i, source.y + step_y * i);
                    Self::set_cursor_pos(pos)?;
                    sleep(interval);
                }
            }
        }

        Self::set_cursor_pos(target)
    }

    /// Simulates a mouse click event.
    ///
    /// # Examples
    /// ```
    /// use uiautomation::inputs::Mouse;
    ///
    /// let mouse = Mouse::new();
    /// let pos = Mouse::get_cursor_pos().unwrap();
    /// mouse.click(pos).unwrap();
    /// ```
    pub fn click(&self, pos: Point) -> Result<()> {
        if self.auto_move {
            self.move_to(&pos)?;
        }

        self.before_click()?;
        self.mouse_event(pos.x, pos.y, MOUSEEVENTF_LEFTDOWN)?;
        self.mouse_event(pos.x, pos.y, MOUSEEVENTF_LEFTUP)?;
        self.after_click()?;

        Ok(())
    }

    /// Simulates a mouse double click event.
    ///
    /// # Examples
    /// ```
    /// use uiautomation::inputs::Mouse;
    ///
    /// let mouse = Mouse::new();
    /// let pos = Mouse::get_cursor_pos().unwrap();
    /// mouse.double_click(pos).unwrap();
    /// ```
    pub fn double_click(&self, pos: Point) -> Result<()> {
        if self.auto_move {
            self.move_to(&pos)?;
        }

        self.before_click()?;

        self.mouse_event(pos.x, pos.y, MOUSEEVENTF_LEFTDOWN)?;
        self.mouse_event(pos.x, pos.y, MOUSEEVENTF_LEFTUP)?;

        sleep(Duration::from_millis(max(200, self.interval)));

        self.mouse_event(pos.x, pos.y, MOUSEEVENTF_LEFTDOWN)?;
        self.mouse_event(pos.x, pos.y, MOUSEEVENTF_LEFTUP)?;

        self.after_click()?;

        Ok(())
    }

    /// Simulates a right mouse click event.
    ///
    /// # Examples
    /// ```
    /// use uiautomation::inputs::Mouse;
    ///
    /// let mouse = Mouse::new();
    /// let pos = Mouse::get_cursor_pos().unwrap();
    /// mouse.right_click(pos).unwrap();
    /// ```
    pub fn right_click(&self, pos: Point) -> Result<()> {
        if self.auto_move {
            self.move_to(&pos)?;
        }

        self.before_click()?;
        self.mouse_event(pos.x, pos.y, MOUSEEVENTF_RIGHTDOWN)?;
        self.mouse_event(pos.x, pos.y, MOUSEEVENTF_RIGHTUP)?;
        self.after_click()?;

        Ok(())
    }

    fn before_click(&self) -> Result<()> {
        for holdkey in &self.holdkeys {
            let key_input = [Input::create_virtual_key(*holdkey, KEYEVENTF_KEYDOWN)];
            send_input(&key_input)?;
            self.wait();
        }

        Ok(())
    }

    fn after_click(&self) -> Result<()> {
        for holdkey in &self.holdkeys {
            let key_input = [Input::create_virtual_key(*holdkey, KEYEVENTF_KEYUP)];
            send_input(&key_input)?;
            self.wait();
        }

        Ok(())
    }

    fn mouse_event(&self, x: i32, y: i32, flags: MOUSE_EVENT_FLAGS) -> Result<()> {
        let input = [INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: x,
                    dy: y,
                    mouseData: 0,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }];
        send_input(&input)?;
        self.wait();

        Ok(())
    }

    fn wait(&self) {
        if self.interval > 0 {
            sleep(Duration::from_millis(self.interval));
        }
    }
}

/// Retrieves the `(width, height)` size of the primary screen.
pub fn get_screen_size() -> Result<(i32, i32)> {
    let width = unsafe { GetSystemMetrics(SM_CXSCREEN) };
    if width == 0 {
        return Err("Failed to get screen width".into());
    }

    let height = unsafe { GetSystemMetrics(SM_CYSCREEN) };
    if height == 0 {
        return Err("Failed to get screen height".into());
    }

    Ok((width, height))
}

fn send_input(inputs: &[INPUT]) -> Result<()> {
    let sent = unsafe { SendInput(inputs, mem::size_of::<INPUT>() as _) };

    if sent == inputs.len() as u32 {
        Ok(())
    } else {
        Err("Failed to send input".into())
    }
}
