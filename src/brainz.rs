use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::path;

use crate::pit;
use crate::snak;

struct Node {
    pos: usize,
    cost: usize,
}

impl Node {
    fn get_neighbors(&self, width: &usize, max: &usize) -> [usize; 4] {
        let row: usize = self.pos / width * width;
        [
            (self.pos + max - width) % max,
            (self.pos + width) % max,
            row + (self.pos + width - 2) % width,
            row + (self.pos + 2) % width,
        ]
    }
}

impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pos.hash(state)
    }
}

impl Eq for Node {}

impl Ord for Node {
    fn cmp(&self, other: &Node) -> Ordering {
        self.cost.cmp(&other.cost).reverse()
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Node) -> bool {
        self.pos == other.pos
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Node) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn manhattan_dist(width: &usize, pos1: usize, pos2: usize) -> usize {
    (pos1 / width).abs_diff(pos2 / width) + (pos1 % width).abs_diff(pos2 % width)
}

pub fn build_path(came_from: HashMap<usize, usize>, current: usize) -> Vec<usize> {
    let mut path: Vec<usize> = vec![current];
    let mut tmp = current;
    while came_from.contains_key(&tmp) {
        tmp = came_from[&tmp];
        path.push(tmp);
    }
    path.reverse();
    path
}

pub fn path_find<F>(
    start: usize,
    goal: usize,
    width: &usize,
    max: &usize,
    is_collision: F,
) -> Option<(HashMap<usize, usize>, usize)>
where
    F: Fn(usize) -> bool,
{
    let mut open_nodes: BinaryHeap<Node> = BinaryHeap::new();
    let mut came_from: HashMap<usize, usize> = HashMap::new();
    let mut g_score: HashMap<usize, usize> = HashMap::new();
    g_score.insert(start, 0);
    open_nodes.push(Node {
        pos: start,
        cost: manhattan_dist(width, start, goal),
    });

    while !open_nodes.is_empty() {
        let current = open_nodes.pop().unwrap();
        if current.pos == goal {
            return Some((came_from, goal));
        }

        for neighbor in current.get_neighbors(width, max) {
            if is_collision(neighbor) && neighbor != goal {
                continue;
            }

            let walls = current
                .get_neighbors(width, max)
                .iter()
                .fold(0 as u8, |p, x| if is_collision(*x) { p + 1 } else { p });

            let e = 5; //- (walls > 1) as usize;

            let new_g_score = g_score[&current.pos] + 1;

            if new_g_score < *g_score.get(&neighbor).unwrap_or(&usize::MAX) {
                came_from.insert(neighbor, current.pos);
                if !g_score.contains_key(&neighbor) {
                    open_nodes.push(Node {
                        pos: neighbor,
                        cost: new_g_score + manhattan_dist(width, neighbor, goal) * e,
                    })
                }
                g_score.insert(neighbor, new_g_score);
            }
        }
    }
    None
}

pub fn safe_path(pit: &mut pit::Pit, snak: &snak::Snak, munch: usize) -> Option<Vec<usize>> {
    let (head, tail) = snak.state();
    let pit_size = pit.get_size();
    let max = pit_size.flatten();

    let safe = path_find(*head, *tail, &pit_size.cols, &max, |x| {
        pit.is_collision(x) == pit::Collision::Ded
    });

    if let Some((came_from_back, goal_tail)) = path_find(*tail, munch, &pit_size.cols, &max, |x| {
        pit.is_collision(x) == pit::Collision::Ded
    }) {
        if goal_tail != munch {
            return Some(build_path(came_from_back, goal_tail));
        };

        let mut tmp = munch;
        let mut back_path: HashSet<usize> = HashSet::new();
        while came_from_back.contains_key(&tmp) && head != tail {
            tmp = came_from_back[&tmp];
            // pit.set_path(&vec![tmp]);
            back_path.insert(tmp);
        }
        back_path.remove(&munch);

        let (came_from, goal_head) = path_find(*head, munch, &pit_size.cols, &max, |x| {
            pit.is_collision(x) == pit::Collision::Ded || back_path.contains(&x)
        })
        .or(safe)?;

        Some(build_path(came_from, goal_head))
    } else {
        Some(build_path(safe?.0, *tail))
    }
}
