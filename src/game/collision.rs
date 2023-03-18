pub use crate::array2d::*;
pub use crate::common::*;
pub use crate::game::humanoid::*;
pub use crate::game::tile::*;

pub fn resolve_humanoid_tile_collision_y(
    physics: &mut HumanoidPhysics,
    tiles: &impl Index2d<usize, Output = Tile>,
) {
    /*
    // Add HUMANOID_HEIGHT if movement was down.
    let correction = if last_y < physics.y {
        HUMANOID_HEIGHT
    } else {
        0
    };

    // Convert y and last_y to tile space.
    let last_y = ifdiv(last_y as usize + correction, TILE_SIZE);
    let y = ifdiv(physics.y as usize + correction, TILE_SIZE);

    // Order them.
    let y1 = y.min(last_y);
    let y2 = y.max(last_y);

    //
    let x1 = ifdiv(physics.x as usize, TILE_SIZE);
    let x2 = icdiv(physics.x as usize + HUMANOID_WIDTH, TILE_SIZE);
*/
}
