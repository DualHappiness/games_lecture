use super::*;
use std::time::SystemTime;

type Node = Rc<RefCell<SAHNode>>;
#[derive(Default)]
struct SAHNode {
    pub bounds: Bound3,
    pub left: Option<Node>,
    pub right: Option<Node>,
    // ! 改成列表效果更好
    pub obj: Option<Rc<RefCell<dyn Object>>>,
}

pub struct SAHAccel {
    primitives: Vec<Rc<RefCell<dyn Object>>>,
    root: Option<Node>,
}

impl SAHAccel {
    pub fn new(
        primitives: &Vec<Rc<RefCell<dyn Object>>>,
        _: i32,
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
}

fn get_intersection(node: &Option<Node>, ray: &Ray) -> Intersection {
    if let Some(node) = node {
        if node.borrow().bounds.intersect_ray(
            ray,
            &ray.direction_inv,
            &ray.direction.iter().map(|&v| v > 0f32).collect::<Vec<_>>(),
        ) {
            if let Some(obj) = &node.borrow().obj {
                return obj.borrow().get_intersection(ray);
            }

            let left = get_intersection(&node.borrow().left, ray);
            let right = get_intersection(&node.borrow().right, ray);
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

fn recursive_build(objects: &[Rc<RefCell<dyn Object>>]) -> Node {
    let node: Node = Rc::new(RefCell::new(Default::default()));

    let mut bound = Default::default();
    for obj in objects {
        bound = union(&bound, &obj.borrow().get_bounds());
    }

    match objects.len() {
        0 => (),
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
            let mut cost = f32::MAX;
            let mut best = None;

            let bucket_size = 5;
            let c_trav = 1f32;
            let c_insect = 2f32;
            for (i, axis) in bound.diagonal().iter().enumerate() {
                // x, y, z axis
                let bucket_len = axis / bucket_size as f32;
                // let mut bucket_bound = vec![Bound3::default(); bucket_size];
                let mut bucket_objs = vec![vec![]; bucket_size];
                for obj in objects {
                    let index = ((obj.borrow().get_bounds().centroid()[i] - bound.p_min[i])
                        / bucket_len) as usize;
                    // bucket_bound[index] = union(&bucket_bound[index], &obj.borrow().get_bounds());
                    bucket_objs[index].push(Rc::clone(obj));
                }

                for k in 0..bucket_size {
                    let count_a = bucket_objs[..k].iter().map(|v| v.len()).sum::<usize>() as f32;
                    if count_a == 0f32 {
                        continue;
                    }
                    let count_b = bucket_objs[k..].iter().map(|v| v.len()).sum::<usize>() as f32;
                    let local_cost = c_trav
                        + (k + 1) as f32 / bucket_size as f32 * c_insect * count_a
                        + (1f32 - (k + 1) as f32 / bucket_size as f32) * c_insect * count_b;
                    if local_cost < cost {
                        cost = local_cost;
                        best = Some((
                            bucket_objs[..k].iter().fold(vec![], |mut ret, v| {
                                v.iter().for_each(|obj| ret.push(Rc::clone(obj)));
                                ret
                            }),
                            bucket_objs[k..].iter().fold(vec![], |mut ret, v| {
                                v.iter().for_each(|obj| ret.push(Rc::clone(obj)));
                                ret
                            }),
                        ));
                    }
                }
            }

            node.borrow_mut().bounds = bound;
            let (left_objs, right_objs) = best.unwrap();
            node.borrow_mut().left = Some(recursive_build(&left_objs));
            node.borrow_mut().right = Some(recursive_build(&right_objs));
        }
    }

    node
}
