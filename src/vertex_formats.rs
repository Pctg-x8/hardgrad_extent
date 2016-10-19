
use interlude::*;

#[repr(C)] #[derive(Clone)] pub struct Color(pub f32, pub f32, pub f32, pub f32);		// r, g, b, a
#[repr(C)] #[derive(Clone)] pub struct Vertex(pub Position, pub Color);
#[repr(C)] #[derive(Clone)] pub struct TexCoordinate(pub f32, pub f32, pub f32, pub f32);	// u, v, w, 1
#[repr(C)] #[derive(Clone)] pub struct TexturedPos(pub Position, pub TexCoordinate);
