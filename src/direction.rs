#[derive(Clone, Copy, Debug)]
pub enum Direction2D {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
}

impl Direction2D {
    pub fn random_order() -> [Direction2D; 4] {
        let mut directions = [
            Direction2D::Up,
            Direction2D::Down,
            Direction2D::Left,
            Direction2D::Right,
        ];

        fastrand::shuffle(&mut directions);

        directions
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Direction3D {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
    In = 4,
    Out = 5,
}

impl Direction3D {
    pub fn random_order() -> [Direction3D; 6] {
        let mut directions = [
            Direction3D::Up,
            Direction3D::Down,
            Direction3D::Left,
            Direction3D::Right,
            Direction3D::In,
            Direction3D::Out,
        ];

        fastrand::shuffle(&mut directions);

        directions
    }
}
