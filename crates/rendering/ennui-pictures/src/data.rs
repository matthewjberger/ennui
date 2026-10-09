ennui::tuning! {
    #[derive(Clone)]
    pub struct Image {
        width: u32 = 1,
        height: u32 = 1,
        pixels: Vec<u8> = vec![255, 255, 255, 255],
        srgb: bool = true,
        mips: Vec<Vec<u8>> = Vec::new(),
        mipped: bool = true,
    }
}
