use super::*;
use std::time::SystemTime;

type Node = Rc<BVHBuildNode>;
#[derive(Default)]
struct BVHBuildNode {
    pub bounds: Bound3,
    pub left: Option<Node>,
    pub right: Option<Node>,
    pub obj: Option<Rc<dyn Object>>,

    pub area: f32,
}

pub struct BVHPrimitiveInfo {}

pub enum SplitMethod {
    NAIVE,
    SAH,
}

pub struct BVHAccel {
    primitives: Vec<Rc<dyn Object>>,

    root: Option<Node>,
}

impl BVHAccel {
    pub fn new(
        primitives: &Vec<Rc<dyn Object>>,
        _max_prims_in_node: i32,
        _split_method: SplitMethod,
    ) -> Self {
        let mut ret = Self {
            primitives: primitives.iter().map(|obj| Rc::clone(obj)).collect(),
            root: None,
        };

        let start = SystemTime::now();
        if ret.primitives.is_empty() {
            return ret;
        }

        ret.root = Some(recursive_build(&ret.primitives));
        let elapsed = start.elapsed().unwrap();
        let hrs = elapsed.as_secs() / 3600;
        let mins = elapsed.as_secs() / 60 - hrs * 60;
        let secs = elapsed.as_secs() - mins * 60 - hrs * 3600;
        println!(
            "\rBVH Generation Complete: \nTime Taken: {} hrs, {} mins, {} secs\n\n",
            hrs, mins, secs
        );

        ret
    }

    pub fn intersect(&self, ray: &Ray) -> Intersection {
        get_intersection(&self.root, ray)
    }

    pub fn sample(&self, pos: &mut Intersection, pdf: &mut f32) {
        let area = self.root.as_ref().unwrap().area;
        let p = get_random_float().sqrt() * area;
        get_sample(&self.root, p, pos, pdf);
        *pdf /= area;
    }
}

fn get_sample(node: &Option<Node>, p: f32, pos: &mut Intersection, pdf: &mut f32) {
    if let Some(node) = node {
        if let Some(obj) = &node.obj {
            obj.sample(pos, pdf);
            *pdf *= node.area;
        } else {
            let left_area = node.left.as_ref().unwrap().area;
            if p < left_area {
                get_sample(&node.left, p, pos, pdf);
            } else {
                get_sample(&node.right, p - left_area, pos, pdf);
            }
        }
    }
}

fn get_intersection(node: &Option<Node>, ray: &Ray) -> Intersection {
    if let Some(node) = node {
        if node.bounds.intersect_ray(
            ray,
            &ray.direction_inv,
            &ray.direction.iter().map(|&v| v > 0f32).collect::<Vec<_>>(),
        ) {
            if let Some(obj) = &node.obj {
                return obj.get_intersection(ray);
            }

            let left = get_intersection(&node.left, ray);
            let right = get_intersection(&node.right, ray);
            return match (left.happened, right.happened) {
                (true, false) => left,
                (false, true) => right,
                (false, false) => Intersection::default(),
                _ => {
                    if left.distance < right.distance {
                        left
                    } else {
                        right
                    }
                }
            };
        }
    }

    return Intersection::default();
}

fn recursive_build(objects: &[Rc<dyn Object>]) -> Node {
    let mut node = BVHBuildNode::default();
    match objects.len() {
        1 => {
            node.bounds = objects[0].get_bounds();
            node.obj = Some(Rc::clone(&objects[0]));
            node.area = objects[0].get_area();
        }
        2 => {
            let left = recursive_build(&objects[..1]);
            let right = recursive_build(&objects[1..]);

            node.area = left.area + right.area;
            node.bounds = union(&left.bounds, &right.bounds);
            node.left = Some(left);
            node.right = Some(right);
        }
        _ => {
            let mut centroid_bounds = Default::default();
            for obj in objects {
                centroid_bounds = union_point(&centroid_bounds, &obj.get_bounds().centroid());
            }

            let index = centroid_bounds.max_extent() as usize;
            let mut objects: Vec<_> = objects.iter().map(|obj| Rc::clone(&obj)).collect();
            objects.sort_by(|o1, o2| {
                if o1.get_bounds().centroid()[index] < o2.get_bounds().centroid()[index] {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                }
            });

            let mid = objects.len() / 2;
            let left = recursive_build(&objects[..mid + 1]);
            let right = recursive_build(&objects[mid..]);

            node.bounds = union(&left.bounds, &right.bounds);
            node.area = left.area + right.area;
            node.left = Some(left);
            node.right = Some(right);
        }
    }

    Rc::from(node)
}
