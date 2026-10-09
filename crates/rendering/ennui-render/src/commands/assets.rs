use crate::data::{Image, TextureId};
use crate::resources::Images;

pub fn insert_image(images: &mut Images, image: Image) -> TextureId {
    let Some(slot) = images.free.pop() else {
        images.list.push(image);
        return TextureId(images.list.len() - 1);
    };
    images.list[slot] = image;
    images.dirty.push(slot);
    TextureId(slot)
}

pub fn remove_image(images: &mut Images, held: TextureId) {
    if images.free.contains(&held.0) {
        return;
    }
    let Some(image) = images.list.get_mut(held.0) else {
        return;
    };
    *image = Image::default();
    images.dirty.push(held.0);
    images.freed.push(held.0);
    images.free.push(held.0);
}
