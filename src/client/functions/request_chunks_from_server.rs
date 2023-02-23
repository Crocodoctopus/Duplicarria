use crate::common::*;
use crate::game::net::*;
use crate::game::tile::*;
use array2d::FastArray2D;

pub fn request_chunks_from_server(
    (x, y): (usize, usize),
    (w, h): (usize, usize),
    chunks: &mut FastArray2D<(u16, u16)>,
    outbound: &mut Vec<NetEvent>,
) {
    const CHUNK_LOAD_BUFFER_SIZE_PX: usize = CHUNK_LOAD_BUFFER_SIZE * CHUNK_SIZE;
    const CHUNK_SIZE_PX: usize = TILE_SIZE * CHUNK_SIZE;
    let x1 = ifdiv(x.saturating_sub(CHUNK_LOAD_BUFFER_SIZE_PX), CHUNK_SIZE_PX);
    let y1 = ifdiv(y.saturating_sub(CHUNK_LOAD_BUFFER_SIZE_PX), CHUNK_SIZE_PX);
    let x2 = icdiv(x + w + CHUNK_LOAD_BUFFER_SIZE_PX, CHUNK_SIZE_PX);
    let y2 = icdiv(y + h + CHUNK_LOAD_BUFFER_SIZE_PX, CHUNK_SIZE_PX);
    chunks.for_each_sub_wrapping_mut(x1..x2, y1..y2, |x, y, cached_xy| {
        let new_xy = (x as u16, y as u16);
        if new_xy != *cached_xy {
            *cached_xy = new_xy;
            outbound.push(NetEvent::RequestChunk(x as u16, y as u16));
        }
    });
}
