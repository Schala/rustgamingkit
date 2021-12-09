use std::collections::HashMap;

use ultraviolet::{
	rotor::Rotor3,
	vec::{
		Vec2,
		Vec3,
		Vec4
	}
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MatPropValueID {
	Alpha,
	Bump,
	Diffuse,
	Displacement,
	Emission,
	Metallic,
	Normal,
	Reflective,
	Roughness,
	Specular,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MatPropValue {
	Float(f32),
	Text(String),
	Vector2(Vec2),
	Vector3(Vec3),
	Vector4(Vec4),
}

/// [`HashMap`] type alias for material properties
pub type MaterialPropertyMap = HashMap<MatPropValueID, MatPropValue>;

#[derive(Clone, Debug)]
pub struct Material {
	pub name: String,
	pub properties: MaterialPropertyMap,
	pub shader: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UpAxis {
	Y,
	Z,
}

/// Node identification type
#[derive(Clone, Debug, PartialEq)]
pub enum ObjRef {
	Name(String),
	Number(u32),
}

#[derive(Clone, Debug, PartialEq)]
pub enum NodeData {
	Geometry(Mesh),
	Null,
}

/// Base type of the 3D environment
#[derive(Clone, Debug, PartialEq)]
pub struct Node {
	pub id: ObjRef,
	pub parent: Option<usize>,
	pub children: Vec<Node>,
	pub data: NodeData,
	pub translation: Vec3,
	pub rotation: Rotor3,
	pub scale: Vec3,
}

impl Node {
	pub fn new(id: ObjRef, parent: Option<usize>) -> Node {
		Node {
			id: id,
			parent: parent,
			children: vec![],
			data: NodeData::Null,
			translation: Vec3::zero(),
			rotation: Rotor3::identity(),
			scale: Vec3::one(),
		}
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct VertexGroup {
	pub id: Option<ObjRef>,
	pub children: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Vertex {
	pub groups: Option<Vec<usize>>,
	pub position: Vec4,
	pub normal: Option<Vec3>,
	pub uvw: Vec<Vec3>,
	pub color: Option<Vec4>,
}

impl Vertex {
	pub fn new(groups: Option<Vec<usize>>) -> Vertex {
		Vertex {
			groups: groups,
			position: Vec4::zero(),
			normal: None,
			uvw: vec![],
			color: None,
		}
	}
}

#[derive(Clone, Debug, PartialEq)]
pub enum Face {
	Triangle([usize; 3]),
	Quad([usize; 4]),
	Ngon(Vec<usize>),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mesh {
	pub vertex_groups: Vec<VertexGroup>,
	pub vertices: Vec<Vertex>,
	pub faces: Vec<Face>,
}

impl Mesh {
	/// Removes the specified vertex and any faces it's a part of
	pub fn drop_vertex(&mut self, v: Vertex) {
		let faces = self.find_vertex_faces(&v);
		for i in faces.iter() {
			self.faces.remove(*i);
		}

		/*// Remove references from possible groups
		if let Some(groups) = v.groups {
			for group in groups.iter() {
				for i in self.vertex_groups[group].children.iter_mut() {
					if v == self.vertices[*i] {
						self.vertex_groups[group].children.remove(*i);
					}
				}
			}
		}*/

		for i in 0..self.vertices.len() {
			if v == self.vertices[i] {
				self.vertices.remove(i);
			}
		}
	}

	/// Returns a list of face indices holding the specified vertex
	pub fn find_vertex_faces(&self, vert: &Vertex) -> Vec<usize> {
		let mut indices = vec![];

		for i in 0..self.faces.len() {
			match &self.faces[i] {
				Face::Triangle(t) => for v in t.iter() {
					if *vert == self.vertices[*v] {
						indices.push(i);
					}
				},
				Face::Quad(q) => for v in q.iter() {
					if *vert == self.vertices[*v] {
						indices.push(i);
					}
				},
				Face::Ngon(n) => for v in n.iter() {
					if *vert == self.vertices[*v] {
						indices.push(i);
					}
				}
			}
		}

		indices
	}
}

/// Top level of the 3D environment
#[derive(Clone, Debug)]
pub struct Scene {
	pub root: Node,
	pub materials: Vec<Material>,
	pub up: UpAxis,
}

impl Scene {
	pub fn new(root: Node) -> Scene {
		Scene {
			root: root,
			materials: vec![],
			up: UpAxis::Y,
		}
	}
}

pub fn vec4_to_rot3(v: Vec4) -> Rotor3 {
	Rotor3::from_quaternion_array([v.x, v.y, v.z, v.w])
}
