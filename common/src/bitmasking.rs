
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BitMaskDirection {
    Edges,
    Corners,
    CornersAndEdges,
}

struct IPoint {
    pub x: i32,
    pub y: i32,
}

const DIRS_EDGES: [IPoint; 4] = [
    IPoint { x: 0, y: -1 }, // 1, top
    IPoint { x: 1, y: 0 },  // 2, right
    IPoint { x: 0, y: 1 },  // 4, bottom
    IPoint { x: -1, y: 0 }, // 8, left
];

const DIRS_CORNERS: [IPoint; 4] = [
    IPoint { x: 1, y: -1 },  // 1, top-right
    IPoint { x: 1, y: 1 },   // 2, bottom-right
    IPoint { x: -1, y: 1 },  // 4, bottom-left
    IPoint { x: -1, y: -1 }, // 8, top-left
];

const DIRS_BOTH: [IPoint; 8] = [
    IPoint { x: 1, y: -1 },  // 1, top-right
    IPoint { x: 1, y: 1 },   // 2, bottom-right
    IPoint { x: -1, y: 1 },  // 4, bottom-left
    IPoint { x: -1, y: -1 }, // 8, top-left
    IPoint { x: 0, y: -1 },  // 16, top
    IPoint { x: 1, y: 0 },   // 32, right
    IPoint { x: 0, y: 1 },   // 64, bottom
    IPoint { x: -1, y: 0 },  // 128, left
];

fn get_dirs_arr(dir: BitMaskDirection) -> &'static [IPoint] {
    match dir {
        BitMaskDirection::Edges => &DIRS_EDGES,
        BitMaskDirection::Corners => &DIRS_CORNERS,
        BitMaskDirection::CornersAndEdges => &DIRS_BOTH,
    }
}

pub fn create_bitmap_from<T, P>(world: &[Vec<T>], predicate: P) -> Vec<Vec<bool>>
where
    P: Fn(&T) -> bool,
{
    let h = world.len();
    let w = world[0].len();
    let mut res = vec![vec![false; w]; h];

    for y in 0..h {
        for x in 0..w {
            res[y][x] = predicate(&world[y][x]);
        }
    }

    res
}

pub fn calc_bitmask(bitmap: &[Vec<bool>], dir: BitMaskDirection) -> Vec<Vec<u32>> {
    let h = bitmap.len();
    let w = bitmap[0].len();

    let dirs = get_dirs_arr(dir);
    let mut res = vec![vec![0; w]; h];

    // Edges:
    //      1
    //   +------+
    // 8 |      | 2
    //   |      |
    //   +------+
    //      4

    // Corners:
    //          1
    // 8 +------+
    //   |      |
    //   |      |
    //   +------+ 2
    //   4

    // Both:
    //        1   2
    // 128 +------+
    //     |      | 4
    // 64  |      |
    //     +------+ 8
    //    32   16

    for y in 0..h {
        for x in 0..w {
            let bit = bitmap[y][x];
            let mut new_byte: u32 = 0;

            if bit {
                for (i, dir) in dirs.iter().enumerate() {
                    let p = IPoint {
                        x: x as i32 + dir.x,
                        y: y as i32 + dir.y,
                    };

                    if is_not_oob(p, w, h) {
                        new_byte |= 1 << i;
                    }
                }
            }

            res[y][x] = new_byte;
        }
    }

    res
}

#[inline]
fn is_not_oob(p: IPoint, w: usize, h: usize) -> bool {
    p.x >= 0 && p.y >= 0 && p.x < w as i32 && p.y < h as i32
}
