
#[derive(Clone, Copy, Debug)]
pub struct DensityVolume {
    pub size_x: usize,
    pub size_y: usize,
    pub size_z: usize,
    pub min_x: i32,
    pub min_y: i32,
    pub min_z: i32,
}

impl DensityVolume {
    pub fn new(
        size_x: usize,
        size_y: usize,
        size_z: usize,
        min_x: i32,
        min_y: i32,
        min_z: i32,
    ) -> Self {
        Self {
            size_x,
            size_y,
            size_z,
            min_x,
            min_y,
            min_z,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.size_x * self.size_y * self.size_z
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn index(&self, x: usize, y: usize, z: usize) -> usize {
        (z * self.size_x + x) * self.size_y + y
    }

    #[inline]
    pub fn block_x(&self, x: usize) -> i32 {
        self.min_x + x as i32
    }

    #[inline]
    pub fn block_y(&self, y: usize) -> i32 {
        self.min_y + y as i32
    }

    #[inline]
    pub fn block_z(&self, z: usize) -> i32 {
        self.min_z + z as i32
    }
}
