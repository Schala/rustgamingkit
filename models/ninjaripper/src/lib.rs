pub mod rip;

use bitflags::bitflags;
use std::fs;

use ultraviolet::vec::{
	Vec2,
	Vec3
};

use meshio_core::{
	io_ext::ReadBinExt,
	IOError,
	scene::{
		Face,
		Mesh,
		Node,
		NodeData,
		ObjRef,
		Scene,
		Vertex
	}
};

use rip::*;

bitflags! {
	pub struct ImportFlag: u32 {
		const FLIP_X = 1;
		const FLIP_WINDING = 2;
		const USE_NORMALS = 4;
		const UV_FLIP_Y = 8;
		const USE_WEIGHTS = 16;
		const USE_SHADERS = 32;
		const SKIP_UNUSED_ATTRS = 64;
		const SKIP_UNUSED_TEXTURES = 128;
		const SKIP_UNTEXTURED = 256;
		const DETECT_DUPLICATES = 512;
		const CROSS_DUPLICATES = 1024;
		const IGNORE_MISSING_TEXTURES = 2048;
		const SKIP_DUPLICATES = 4096;
		const OVERRIDE_ATTR_TYPES = 8192;
	}
}

impl Default for ImportFlag {
	fn default() -> Self {
		ImportFlag::USE_NORMALS | ImportFlag::USE_WEIGHTS | ImportFlag::SKIP_UNUSED_ATTRS |
			ImportFlag::SKIP_UNUSED_TEXTURES
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct ImportCfg {
	pub flags: ImportFlag,
	pub normal_int_div: i32,
	pub normal_mul: Vec3,
	pub normal_scale_add: Vec3,
	pub uv_int_div: i32,
	pub uv_mul: Vec2,
	pub uv_scale_add: Vec2,
	pub pos_attr_override_index: usize,
	pub normal_attr_override_index: usize,
	pub uv_attr_override_indices: Vec<usize>,
	pub color_attr_override_indices: Vec<usize>,
	pub blend_attr_override_indices: Vec<usize>,
	pub weight_attr_override_indices: Vec<usize>,
}

impl Default for ImportCfg {
	fn default() -> Self {
		Self {
			flags: ImportFlag::default(),
			normal_int_div: 255,
			normal_mul: Vec3::one(),
			normal_scale_add: Vec3::zero(),
			uv_int_div: 255,
			uv_mul: Vec2::one(),
			uv_scale_add: Vec2::zero(),
			pos_attr_override_index: 0,
			normal_attr_override_index: 1,
			uv_attr_override_indices: vec![2],
			color_attr_override_indices: vec![],
			blend_attr_override_indices: vec![],
			weight_attr_override_indices: vec![],
		}
	}
}

#[cfg(feature = "import")]
pub fn read(filepath: &str, cfg: ImportCfg, ) -> Result<Scene, IOError> {
	if let Ok(input) = fs::read(filepath) {
		let model_imp = import::rip(&mut input.as_slice());

		if let Ok(model) = model_imp {
			let mut root = Node::new(ObjRef::Name(filepath.to_string()), None);

			let positions = model.get_attr("SV_POSITION", 1);
			let normals = model.get_attr("NORMAL", 1);
			let colors = model.get_attr("COLOR", 1);

			let uvs: Vec<Option<&Attribute>> = (1..=(model.header.num_texs as usize)).into_iter()
				.map(|i| model.get_attr("TEXCOORD", i)).collect();

			let mut mesh = Mesh::default();

			// Gather vertex data
			for i in 0..(model.header.num_verts as usize) {
				let mut vert = Vertex::new(None);

				if let Some(attr) = positions {
					let AttrData::Vertex(v) = attr.data[i];
					vert.position = v;
				}

				if let Some(attr) = normals {
					let AttrData::Vertex(v) = attr.data[i];
					vert.normal = Some(Vec3::new(v[0], v[1], v[2]));
				}

				if let Some(attr) = colors {
					let AttrData::Vertex(v) = attr.data[i];
					vert.color = Some(v.abs());
				}

				for uv in uvs.iter() {
					if let Some(attr) = uv {
						let AttrData::Vertex(v) = attr.data[i];
						vert.uvw.push(Vec3::new(v[0], v[1], v[2]));
					}
				}

				mesh.vertices.push(vert);
			}

			// Apply X axis invert if specified
			if cfg.flags & ImportFlag::FLIP_X != ImportFlag::empty() {
				for v in mesh.vertices.iter_mut() {
					v.position.x *= -1.0;
					if let Some(ref mut n) = v.normal {
						n.x *= 1.0;
					}
				}
			}

			mesh.faces = model.faces.iter().map(|face| {
				Face::Triangle([face[0], face[1], face[2]])
			}).collect();

			root.data = NodeData::Geometry(mesh);

			let scene = Scene::new(root);

			return Ok(scene);
		}

		return Err(IOError {
			msg: "Unable to import model".to_string(),
		});

		/*if let Err(e) = model_imp {
			return Err(IOError {
				msg: format!("{}", e),
			});
		}*/
	} else {
		Err(IOError {
			msg: "Unable to load file".to_string(),
		})
	}
}
