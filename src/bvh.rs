use crate::utility::*;
use crate::hittable::Hittable;
use crate::material::MaterialId;
use crate::render::SceneData;

// ------------------------------------------- Bounding volume hieracrchy -------------------------------------------

type NodeId = u32;
type LeafId = u32;

#[derive(Debug, Clone)]
enum BvhNode {
    Branch {aabb: AABB, left: NodeId, right: NodeId},
    Leaf {aabb: AABB, leaf: LeafId},
}

impl BvhNode {
    fn bounding_box(&self) -> &AABB {
        match self {
            Self::Leaf {aabb, ..} => aabb,
            Self::Branch {aabb, ..} => aabb,
        }
    }
}

#[derive(Clone)]
pub struct Bvh {
    /// Content of the leaf nodes to be indexed by LeafId
    leaves: Vec<Hittable>,
    /// Tree structure to be index by NodeId
    nodes: Vec<BvhNode>,
    /// Id of the root node
    root: NodeId,
}

fn make_bvh(content: &mut [(LeafId, AABB)], sort_axis: usize, nodes: &mut Vec<BvhNode>) 
    -> NodeId
{
    match content.len() {
        0 => unreachable!(),
        1 => {
            let (leaf, aabb) = content[0].clone();
            nodes.push(BvhNode::Leaf {leaf, aabb});
            (nodes.len() - 1) as NodeId
        }
        _ => {
            let (left_content, right_content) = split(content, sort_axis);
            let left = make_bvh(left_content, (sort_axis + 1) % 3, nodes);
            let right = make_bvh(right_content, (sort_axis + 1) % 3, nodes);
            let aabb = nodes[left as usize].bounding_box()
                .union(nodes[right as usize].bounding_box());
            nodes.push(BvhNode::Branch {left, right, aabb});
            (nodes.len() - 1) as NodeId
        }
    }
}

fn split(content: &mut [(LeafId, AABB)], sort_axis: usize) -> (&mut [(LeafId, AABB)], &mut [(LeafId, AABB)]) {
    // Sort by bounding box centroid
    content.sort_unstable_by(|(_, x_bb), (_, y_bb)| {
        let x_center = 0.5 * (x_bb.min[sort_axis] + x_bb.max[sort_axis]);
        let y_center = 0.5 * (y_bb.min[sort_axis] + y_bb.max[sort_axis]);
        x_center.partial_cmp(&y_center).unwrap()
    });
    // Split at the median without allocating a new vector
    content.split_at_mut(content.len() / 2)
}

impl Bvh {
    pub fn new(hittables: Vec<Hittable>, scene_data: &SceneData) -> Self {
        let mut content = hittables.iter().enumerate().map(|(id, x)| (id as LeafId, x.bounding_box(scene_data)))
            .collect::<Vec<_>>();
        
        let mut nodes = Vec::new();
        let root = make_bvh(&mut content, 0, &mut nodes);

        // nodes.iter().enumerate().for_each(|(id, n)| match n {
        //     BvhNode::Leaf {..} => println!("#{}: Leaf ({:?})", id, content[id].1),
        //     BvhNode::Branch {left, right, ..} => println!("#{}: Branch (#{}, #{})", id, left, right),
        // });
        // println!("Recap: {} branches, {} leaves",
        //     nodes.iter().filter(|n| matches!(n, BvhNode::Branch {..})).count(),
        //     nodes.iter().filter(|n| matches!(n, BvhNode::Leaf {..})).count(),
        // );
        // println!("{}", std::mem::size_of::<BvhNode>());
        
        Bvh {
            leaves: hittables,
            nodes, root
        }
    }

    fn hit_node(&self, ray: &RayExpanded, node: NodeId, scene_data: &SceneData) -> Option<(Hit, MaterialId)> {
        match &self.nodes[node as usize] {
            BvhNode::Leaf {aabb, leaf} => {
                if aabb.collide(ray) {
                    self.leaves[*leaf as usize].hit(&ray.inner, scene_data)
                } else {
                    None
                }
            },
            BvhNode::Branch {aabb, left, right} => {
                if aabb.collide(ray) {
                    let mut hit = None;
                    let mut ray = ray.clone();
                    if let Some(new_hit) = self.hit_node(&ray, *left, scene_data) {
                        ray.inner.t_max = new_hit.0.t;
                        hit.replace(new_hit);
                    }
                    if let Some(new_hit) = self.hit_node(&ray, *right, scene_data) {
                        hit.replace(new_hit);
                    }
                    hit
                } else {
                    None
                }
            },
        }
    }

    pub fn hit(&self, ray: &Ray, scene_data: &SceneData) -> Option<(Hit, MaterialId)> {
        let ray = ray.clone().expand();
        self.hit_node(&ray, self.root, scene_data)
    }
}