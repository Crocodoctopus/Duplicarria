use crate::shared::net_event::NetEvent;
use crate::shared::*;
use array2d::FastArray2D;

pub fn request_chunks_from_server(
    (x, y, w, h): (f32, f32, f32, f32),
    chunks: &mut FastArray2D<(u16, u16)>,
    net_events: &mut Vec<NetEvent>,
) {
    // Map view to chunk space.
    const CHHUNK_LOAD_BUFFER_SIZE_PX: f32 = (CHUNK_LOAD_BUFFER_SIZE * TILE_SIZE) as f32;
    const CHUNK_SIZE_PX: f32 = (CHUNK_SIZE * TILE_SIZE) as f32;
    let x1 = ((x - CHHUNK_LOAD_BUFFER_SIZE_PX) / CHUNK_SIZE_PX) as usize;
    let y1 = ((y - CHHUNK_LOAD_BUFFER_SIZE_PX) / CHUNK_SIZE_PX) as usize;
    let x2 = ((x + w + CHHUNK_LOAD_BUFFER_SIZE_PX) / CHUNK_SIZE_PX).ceil() as usize;
    let y2 = ((y + h + CHHUNK_LOAD_BUFFER_SIZE_PX) / CHUNK_SIZE_PX).ceil() as usize;

    // Updating more chunks than exist is a bug.
    assert!(x2 - x1 <= chunks.size().0);
    assert!(y2 - y1 <= chunks.size().1);

    // Loop chunks in range, setting their new positions.
    chunks.for_each_sub_wrapping_mut(x1..x2, y1..y2, |x, y, cached_xy| {
        let new_xy = (x as u16, y as u16);
        if new_xy != *cached_xy {
            *cached_xy = new_xy;
            net_events.push(NetEvent::RequestChunk(x as u16, y as u16));
        }
    })
}
