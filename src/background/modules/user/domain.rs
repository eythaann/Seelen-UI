#[allow(dead_code)]
pub enum PictureQuality {
    Quality1080,
    Quality448,
    Quality424,
    Quality240,
    Quality208,
    Quality192,
    Quality96,
    Quality64,
    Quality48,
    Quality40,
    Quality32,
}

impl PictureQuality {
    pub fn as_str(&self) -> &'static str {
        match self {
            PictureQuality::Quality1080 => "Image1080",
            PictureQuality::Quality192 => "Image192",
            PictureQuality::Quality208 => "Image208",
            PictureQuality::Quality240 => "Image240",
            PictureQuality::Quality32 => "Image32",
            PictureQuality::Quality40 => "Image40",
            PictureQuality::Quality424 => "Image424",
            PictureQuality::Quality448 => "Image448",
            PictureQuality::Quality48 => "Image48",
            PictureQuality::Quality64 => "Image64",
            PictureQuality::Quality96 => "Image96",
        }
    }
}
