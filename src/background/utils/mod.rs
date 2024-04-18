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

/* pub fn decompress_u128(hex_str: &str) -> u128 {
    u128::from_str_radix(hex_str, 16).expect("could not decompress u128")
} */

pub fn pascal_to_kebab(input: &str) -> String {
    let mut kebab_case = String::new();
    let mut prev_char_lowercase = false;
    for c in input.chars() {
        if c.is_uppercase() {
            if prev_char_lowercase {
                kebab_case.push('-');
            }
            kebab_case.push(c.to_ascii_lowercase());
            prev_char_lowercase = false;
        } else {
            kebab_case.push(c);
            prev_char_lowercase = true;
        }
    }
    kebab_case
}

pub fn kebab_to_pascal(input: &str) -> String {
    let mut pascal_case = String::new();
    let mut prev_char_dash = false;
    for c in input.chars() {
        if c == '-' {
            prev_char_dash = true;
        } else {
            if prev_char_dash || pascal_case.is_empty() {
                pascal_case.push(c.to_ascii_uppercase());
                prev_char_dash = false;
            } else {
                pascal_case.push(c);
            }
        }
    }
    pascal_case
}
