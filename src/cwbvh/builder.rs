use std::time::Instant;

use crate::{
    aabb::Aabb,
    bvh2::reinsertion::ReinsertionOptimizer,
    cwbvh::{bvh2_to_cwbvh::bvh2_to_cwbvh, CwBvh},
    splits::split_aabbs_preset,
    triangle::Triangle,
    Boundable, BvhBuildParams,
};

/// Build a cwbvh from the given list of Triangles.
pub fn build_cwbvh_from_tris(
    triangles: &[Triangle],
    config: BvhBuildParams,
    core_build_time: &mut f32,
) -> CwBvh {
    let mut aabbs = Vec::with_capacity(triangles.len());
    let mut indices = Vec::with_capacity(triangles.len());
    let mut largest_half_area = 0.0;
    let mut avg_half_area = 0.0;

    for (i, tri) in triangles.iter().enumerate() {
        let a = tri.v0;
        let b = tri.v1;
        let c = tri.v2;
        let mut aabb = Aabb::empty();
        aabb.extend(a).extend(b).extend(c);
        let half_area = aabb.half_area();
        largest_half_area = half_area.max(largest_half_area);
        avg_half_area += half_area;
        aabbs.push(aabb);
        indices.push(i as u32);
    }
    avg_half_area /= triangles.len() as f32;

    let start_time = Instant::now();

    if config.pre_split {
        split_aabbs_preset(
            &mut aabbs,
            &mut indices,
            triangles,
            avg_half_area,
            largest_half_area,
        );
    }

    let mut bvh2 = config.ploc_search_distance.build(
        &aabbs,
        indices,
        config.sort_precision,
        config.search_depth_threshold,
    );
    ReinsertionOptimizer::run(&mut bvh2, config.reinsertion_batch_ratio, None);
    let cwbvh = bvh2_to_cwbvh(&bvh2, config.max_prims_per_leaf);

    *core_build_time += start_time.elapsed().as_secs_f32();

    {
        #[cfg(debug_assertions)]
        bvh2.validate(triangles, false, config.pre_split);
        cwbvh.validate(config.pre_split, false, triangles);
    }

    cwbvh
}

/// Build a cwbvh from the given list of Boundable primitives.
/// `pre_split` in BvhBuildParams is ignored in this case.
// TODO: we could optionally do imprecise basic Aabb splits.
pub fn build_cwbvh<T: Boundable>(
    primitives: &[T],
    config: BvhBuildParams,
    core_build_time: &mut f32,
) -> CwBvh {
    let mut aabbs = Vec::with_capacity(primitives.len());
    let mut indices = Vec::with_capacity(primitives.len());

    for (i, primitive) in primitives.iter().enumerate() {
        indices.push(i as u32);
        aabbs.push(primitive.aabb());
    }

    let start_time = Instant::now();

    let mut bvh2 = config.ploc_search_distance.build(
        &aabbs,
        indices,
        config.sort_precision,
        config.search_depth_threshold,
    );
    ReinsertionOptimizer::run(&mut bvh2, config.reinsertion_batch_ratio, None);
    let cwbvh = bvh2_to_cwbvh(&bvh2, config.max_prims_per_leaf);

    *core_build_time += start_time.elapsed().as_secs_f32();

    cwbvh
}