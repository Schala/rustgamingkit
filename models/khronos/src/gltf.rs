use ultraviolet::{
	mat::{
		Mat2,
		Mat3,
		Mat4
	},
	vec::{
		Vec2,
		Vec3,
		Vec4
	}
};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AccessorValue {
	Matrix2x2(Mat2),
	Matrix3x3(Mat3),
	Matrix4x4(Mat4),
	Scalar(f32),
	Vector2(Vec2),
	Vector3(Vec3),
	Vector4(Vec4),
}

pub enum ComponentType {
	Int8 = 5120,
	UInt8,
	Int16,
	UInt16,
	Float = 5126,
}

pub struct CameraType {
	pub aspect_ratio: u32,
	pub yfov: f32,
	pub zfar: f32,
	pub znear: f32,
}

pub struct Camera {
	pub name: String,
	pub kind: (String, CameraType),
}
