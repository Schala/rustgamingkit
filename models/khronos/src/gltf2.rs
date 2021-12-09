use std::rc::{
	Rc,
	Weak
};

use ultraviolet::{
	mat::DMat4,
	vec::DVec4
};

mod gltf;
use gltf::*;

/// glTF 2 seems to hold references by numerical ID. We can resolve and swap these later.
pub enum Ref<T> {
	ID(u32),
	Ref(Weak<T>),
}

pub enum Generator {
	Collada2GLTF,
}

pub struct Asset {
	pub generator: Generator
	pub version: String,
}

pub struct MeshPrimAttrs {
	pub normal:
}

pub struct Mesh {
	pub mode: u32,
}

pub struct Node {
	pub children: Vec<Rc<Node>>,
	pub matrix: DMat4,
	pub camera: Ref<Camera>,
}

pub struct Accessor {
	pub component_type: u32,
	pub count: u32,
	pub max: AccessorValue,
	pub min: AccessorValue,
}

pub struct MetallicRoughness {
	pub base_color: DVec4,
	pub metallic: f64,
	pub roughness: f64,
}

pub enum AlphaMode {
	Opaque,
}

pub struct Material {
	pub metallic_roughness: MetallicRoughness,
	pub emission: DVec4,
	pub name: String,
}
