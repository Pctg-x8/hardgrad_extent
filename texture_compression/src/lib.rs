/// Texture Compression crate: Various Algorithms of Texture Compression implemented by Rustlang

pub mod block_compression;
pub use block_compression::{BC4, BC5};

pub trait CompressionAlgorithm
{
	fn compress(source: &[u8], size: (usize, usize)) -> Vec<u8>;
}
