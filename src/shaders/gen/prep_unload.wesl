import constants::GEN_SIDE;
import package::core::{pos, player::Player};
import package::gen::core::UnloadParams;

@group(0) @binding(0) var<storage, read> player: Player;
@group(0) @binding(1) var<storage, read_write> indirect_args: array<u32, 6>;
@group(0) @binding(2) var<storage, read_write> unload_params: UnloadParams;

@compute @workgroup_size(1, 1, 1)
fn prep_unload() {
    let prev_chunk = pos::chunk(vec3i(floor(player.prev_pos)));
    let curr_chunk = pos::chunk(vec3i(floor(player.pos)));
    let delta = curr_chunk - prev_chunk;
    let half_side = i32(GEN_SIDE / 2u);
    var origin = vec3i(0);
    var depths = vec3u(0);
    var is_unload_needed = false;

    let dx = delta.x;
    let dy = delta.y;
    let dz = delta.z;

    if dx != 0 {
        if dx > 0 {
            origin.x = prev_chunk.x - half_side;
            depths.x = u32(dx);
        } else {
            origin.x = curr_chunk.x + half_side;
            depths.x = u32(-dx);
        }
        is_unload_needed = true;
    }
    if dy != 0 {
        if dy > 0 {
            origin.y = prev_chunk.y - half_side;
            depths.y = u32(dy);
        } else {
            origin.y = curr_chunk.y + half_side;
            depths.y = u32(-dy);
        }
        is_unload_needed = true;
    }
    if dz != 0 {
        if dz > 0 {
            origin.z = prev_chunk.z - half_side;
            depths.z = u32(dz);
        } else {
            origin.z = curr_chunk.z + half_side;
            depths.z = u32(-dz);
        }
        is_unload_needed = true;
    }

    if !is_unload_needed {
        indirect_args[0] = 0u;
        return;
    }

    unload_params.origin = origin;
    unload_params.depths = depths;

    let groups = (GEN_SIDE + 15u) / 16u;
    indirect_args[0] = groups;
    indirect_args[1] = groups;
    indirect_args[2] = 1u;
}
