use mime::Mime;

use std::collections::HashMap;

use std::rc::{
	Rc,
	Weak
};

use ultraviolet::{
	mat::Mat4,
	vec::{
		Vec3,
		Vec4
	}
};

mod gltf;
use gltf::*;

mod khr_materials_common;
use khr_materials_common::*;

/// glTF stores its references as string identifiers. Upon resolution,
/// we can swap these out with a weak reference to the actual data.
#[derive(Clone, Debug, PartialEq)]
pub enum Ref<T> {
	ID(String),
	Ref(Weak<T>),
}

pub struct Accessor {
	pub buffer_view: Ref<BufferView>,
	pub byte_offset: i32,
	pub byte_stride: i32,
	pub component_type: u32,
	pub count: u32,
	pub min: Option<AccessorValue>,
	pub max: Option<AccessorValue>,
}

pub enum AssetProfileAPI {
	WebGL,
}

pub struct AssetProfile {
	pub api: AssetProfileAPI,
	pub version: String,
}

pub struct Asset {
	pub generator: String,
	pub premultiplied_alpha: bool,
	pub profile: AssetProfile,
}

pub struct BufferView {
	pub buffer: String,
	pub byte_length: u32,
	pub byte_offset: i32,
	pub target: u32,
}

pub enum BufferType {
	Array,
}

pub struct Buffer {
	pub id: String,
	pub kind: BufferType,
	pub uri: Mime,
}

pub struct MaterialValues {
	pub ambient: Vec4,
	pub diffuse: Vec4,
	pub emission: Vec4,
	pub shininess: f32,
	pub specular: Vec4,
}

pub struct Material {
	pub name: String,
	pub technique: Ref<Technique>,
	pub values: MaterialValues,
}

pub struct MeshPrimAttrs {
	pub normal: Ref<Accessor>,
	pub position: Ref<Accessor>,
}

pub struct MeshPrimitive {
	pub attrs: MeshPrimAttrs,
	pub indices: Ref<Accessor>,
	pub material: Ref<Material>,
	pub mode: u32,
}

pub struct Mesh {
	pub name: String,
	pub primitives: Vec<MeshPrimitive>,
}

pub struct Node {
	pub children: Vec<Rc<Node>>,
	pub matrix: Mat4,
	pub meshes: Option<Vec<Rc<Mesh>>>,
	pub name: String,
}

pub struct Program {
	pub attrs: TechVarMap,
	pub frag_shader: Ref<Shader>,
	pub vert_shader: Ref<Shader>,
}

pub struct Scene {
	pub name: String,
	pub nodes: Vec<Rc<Node>>,
}

pub struct Shader {
	pub kind: u32,
	pub uri: Mime,
}

pub enum TechParamSemantic {
	ModelView,
	ModelViewInverseTranspose,
	Normal,
	Position,
	Projection,
}

pub struct TechParam {
	pub node: Ref<Node>,
	pub semantic: TechParamSemantic,
	pub kind: u32,
	pub value: Vec3,
}

pub type StateMap = HashMap<String, Vec<u32>>;

pub type TechVarMap = HashMap<String, Ref<TechParam>>;

pub struct Technique {
	pub attrs: TechVarMap,
	pub parameters: Vec<TechParam>,
	pub program: Ref<Program>,
	pub states: StateMap,
	pub uniforms: TechVarMap,
}

pub mod import {
	use json::{
		from,
		JsonValue,
		parse,
		self
	};

	use thiserror::Error;

	#[derive(Debug, Error, PartialEq)]
	pub enum GLTF1ImportError {
		#[error("JSON parsing error")]
		Json(json::Error),
	}

	pub fn gltf(obj: &JsonValue) -> json::Result {
	}
}
