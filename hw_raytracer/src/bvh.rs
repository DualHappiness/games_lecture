use super::*;
use std::time::SystemTime;

type Node = Rc<RefCell<BVHBuildNode>>;
#[derive(Default)]
pub struct BVHBuildNode {
    pub bounds: Bound3,
    pub left: Option<Node>,
    pub right: Option<Node>,
    pub obj: Option<Rc<RefCell<dyn Object>>>,

    pub split_axis: i32,
    pub first_prim_offset: i32,
    pub n_primtives: i32,
}

pub struct BVHPrimitiveInfo {}

pub enum SplitMethod {
    NAIVE,
    SAH,
}

pub struct BVHAccel {
    max_prims_in_node: i32,
    split_method: SplitMethod,
    primitives: Vec<Rc<RefCell<dyn Object>>>,

    root: Option<Node>,
}

impl BVHAccel {
    pub fn new(
        primitives: &Vec<Rc<RefCell<dyn Object>>>,
        max_prims_in_node: i32,
        split_method: SplitMethod,
    ) -> Self {
        let mut ret = Self {
            primitives: primitives.iter().map(|obj| Rc::clone(obj)).collect(),
            max_prims_in_node: 255.min(max_prims_in_node),
            split_method,
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
}

fn get_intersection(node: &Option<Node>, ray: &Ray) -> Intersection {
    let dir_inv = Vector3f::new(
        1f32 / ray.direction.x,
        1f32 / ray.direction.y,
        1f32 / ray.direction.z,
    );

    let dir_is_neg = vec![
        ray.direction.x < 0f32,
        ray.direction.y < 0f32,
        ray.direction.z < 0f32,
    ];

    get_intersection_recurse(node, ray, &dir_inv, &dir_is_neg)
}

fn get_intersection_recurse(
    node: &Option<Node>,
    ray: &Ray,
    dir_inv: &Vector3f,
    dir_is_neg: &[bool],
) -> Intersection {
    match node {
        None => Intersection::default(),
        Some(node) => match node.borrow().bounds.intersect_ray(ray, dir_inv, dir_is_neg) {
            false => Intersection::default(),
            true => match &node.borrow().obj {
                None => {
                    let left =
                        get_intersection_recurse(&node.borrow().left, ray, dir_inv, dir_is_neg);
                    let right =
                        get_intersection_recurse(&node.borrow().right, ray, dir_inv, dir_is_neg);
                    match (left.happened, right.happened) {
                        (true, false) => left,
                        (false, true) => right,
                        (false, false) => Intersection::default(),
                        (true, true) => {
                            if left.distance < right.distance {
                                left
                            } else {
                                right
                            }
                        }
                    }
                }
                Some(obj) => obj.borrow().get_intersection(ray),
            },
        },
    }
}

fn recursive_build(objects: &[Rc<RefCell<dyn Object>>]) -> Node {
    let node: Node = Rc::new(RefCell::new(Default::default()));

    let mut bound = Default::default();
    for obj in objects {
        bound = union(&bound, &obj.borrow().get_bounds());
    }

    match objects.len() {
        1 => {
            node.borrow_mut().bounds = objects[0].borrow().get_bounds();
            node.borrow_mut().obj = Some(Rc::clone(&objects[0]));
        }
        2 => {
            let left = recursive_build(&objects[..1]);
            let right = recursive_build(&objects[1..]);

            node.borrow_mut().bounds = union(&left.borrow().bounds, &right.borrow().bounds);
            node.borrow_mut().left = Some(left);
            node.borrow_mut().right = Some(right);
        }
        _ => {
            let mut centroid_bounds = Default::default();
            for obj in objects {
                centroid_bounds =
                    union_point(&centroid_bounds, &obj.borrow().get_bounds().centroid());
            }

            let index = centroid_bounds.max_extent() as usize;
            let mut objects: Vec<_> = objects.iter().map(|obj| Rc::clone(&obj)).collect();
            objects.sort_by(|o1, o2| {
                if o1.borrow().get_bounds().centroid()[index]
                    < o2.borrow().get_bounds().centroid()[index]
                {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                }
            });

            let mid = objects.len() / 2;
            let left = recursive_build(&objects[..mid + 1]);
            let right = recursive_build(&objects[mid..]);

            node.borrow_mut().bounds = union(&left.borrow().bounds, &right.borrow().bounds);
            node.borrow_mut().left = Some(left);
            node.borrow_mut().right = Some(right);
        }
    }

    node
}
