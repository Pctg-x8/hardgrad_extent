
#[repr(C)] pub struct Position(pub f32, pub f32, pub f32, pub f32);		// x, y, z, w
#[repr(C)] pub struct Color(pub f32, pub f32, pub f32, pub f32);		// r, g, b, a
#[repr(C)] pub struct Vertex(pub Position, pub Color);
#[repr(C)] pub struct TexCoordinate(pub f32, pub f32, pub f32, pub f32);	// u, v, w, 1
#[repr(C)] pub struct TexturedPos(pub Position, pub TexCoordinate);
