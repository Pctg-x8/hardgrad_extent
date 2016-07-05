
#[repr(C)] pub struct Position(pub f32, pub f32, pub f32, pub f32);		// x, y, z, w
#[repr(C)] pub struct Color(pub f32, pub f32, pub f32, pub f32);		// r, g, b, a
#[repr(C)] pub struct Vertex(pub Position, pub Color);
