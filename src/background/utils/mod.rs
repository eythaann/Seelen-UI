use windows::Win32::Foundation::RECT;

pub fn filename_from_path(path: &str) -> String {
    path.split('\\').last().unwrap_or_default().to_string()
}

pub fn are_overlaped(rect1: &RECT, rect2: &RECT) -> bool {
    let x_overlap = !(rect1.right <= rect2.left || rect2.right <= rect1.left);
    let y_overlap = !(rect1.bottom <= rect2.top || rect2.bottom <= rect1.top);
    x_overlap && y_overlap
}

pub fn compress_u128(num: u128) -> String {
    format!("{:x}", num)
}

pub fn decompress_u128(hex_str: &str) -> u128 {
    u128::from_str_radix(hex_str, 16).expect("could not decompress u128")
}