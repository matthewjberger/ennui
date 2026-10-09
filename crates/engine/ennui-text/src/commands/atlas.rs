use crate::data::Mark;
use crate::queries::field::cell_of;
use crate::resources::Glyphs;
use crate::theme::SUPERSAMPLE;
use crate::theme::{PAD, SHEET};
use cosmic_text::{CacheKey, SubpixelBin, SwashContent};

pub fn marked(glyphs: &mut Glyphs, key: CacheKey) -> Option<Mark> {
    let settled = CacheKey {
        x_bin: SubpixelBin::Zero,
        y_bin: SubpixelBin::Zero,
        ..key
    };
    if let Some(held) = glyphs.held.get(&settled) {
        return *held;
    }
    let mark = cut(glyphs, settled);
    glyphs.held.insert(settled, mark);
    mark
}

fn cut(glyphs: &mut Glyphs, key: CacheKey) -> Option<Mark> {
    let large = CacheKey {
        font_size_bits: (f32::from_bits(key.font_size_bits) * SUPERSAMPLE as f32).to_bits(),
        ..key
    };
    let drawn = glyphs.swash.get_image_uncached(&mut glyphs.fonts, large)?;
    if drawn.placement.width == 0 || drawn.placement.height == 0 {
        return None;
    }
    let coverage: Vec<u8> = match drawn.content {
        SwashContent::Mask => drawn.data.clone(),
        SwashContent::SubpixelMask => drawn
            .data
            .chunks(4)
            .map(|pixel| pixel[0].max(pixel[1]).max(pixel[2]))
            .collect(),
        SwashContent::Color => drawn.data.chunks(4).map(|pixel| pixel[3]).collect(),
    };
    let cell = cell_of(
        &coverage,
        drawn.placement.width,
        drawn.placement.height,
        drawn.placement.left,
        drawn.placement.top,
    );
    let at = match room(glyphs, cell.width, cell.height) {
        Some(at) => at,
        None => {
            glyphs.full =
                glyphs.full || (cell.width + PAD * 2 <= SHEET && cell.height + PAD * 2 <= SHEET);
            return None;
        }
    };
    for row in 0..cell.height {
        for column in 0..cell.width {
            let into = (((at[1] + row) * SHEET + at[0] + column) * 4) as usize;
            let alpha = cell.alphas[(row * cell.width + column) as usize];
            glyphs.sheet.pixels[into] = 255;
            glyphs.sheet.pixels[into + 1] = 255;
            glyphs.sheet.pixels[into + 2] = 255;
            glyphs.sheet.pixels[into + 3] = alpha;
        }
    }
    glyphs.dirty = true;
    let sheet = SHEET as f32;
    let inset = 0.5 / sheet;
    Some(Mark {
        low: [at[0] as f32 / sheet + inset, at[1] as f32 / sheet + inset],
        high: [
            (at[0] + cell.width) as f32 / sheet - inset,
            (at[1] + cell.height) as f32 / sheet - inset,
        ],
        left: cell.left as f32,
        top: cell.top as f32,
        width: cell.width as f32,
        height: cell.height as f32,
    })
}

pub(crate) fn restart(glyphs: &mut Glyphs) {
    glyphs.full = false;
    glyphs.held.clear();
    glyphs.sheet.pixels.fill(0);
    glyphs.walk = [PAD, PAD, 0];
    glyphs.dirty = true;
}

fn room(glyphs: &mut Glyphs, width: u32, height: u32) -> Option<[u32; 2]> {
    if width + PAD * 2 > SHEET || height + PAD * 2 > SHEET {
        return None;
    }
    if glyphs.walk[0] + width + PAD > SHEET {
        glyphs.walk[0] = PAD;
        glyphs.walk[1] += glyphs.walk[2] + PAD;
        glyphs.walk[2] = 0;
    }
    if glyphs.walk[1] + height + PAD > SHEET {
        return None;
    }
    let at = [glyphs.walk[0], glyphs.walk[1]];
    glyphs.walk[0] += width + PAD;
    glyphs.walk[2] = glyphs.walk[2].max(height);
    Some(at)
}
