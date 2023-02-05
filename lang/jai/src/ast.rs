use bitflags::bitflags;
use std::marker::PhantomData;

bitflags! {
	pub(crate) struct VarFlags: u8 {
	}
}

#[derive(Clone, Debug)]
pub(crate) struct Array<T: DataType> {
	pub(crate) len: usize,
	pub(crate) items: Vec<T>,
}

#[derive(Clone, Debug)]
pub(crate) struct Pointer<T: DataType> {
	pub(crate) addr: usize,
	phantom: PhantomData<T>, // ensures type consistency
}

#[derive(Clone, Debug)]
pub(crate) struct Struct {
	pub(crate) align: usize,
	pub(crate) fields: Vec<Variable>,
}

#[derive(Clone, Debug)]
pub(crate) struct Variable {
	pub(crate) name: String,
	pub(crate) data: Box<dyn DataType>,
}

pub(crate) trait DataType {
}

impl<T: DataType> DataType for Array<T> {
}

impl DataType for bool {
}

impl DataType for char {
}

impl DataType for f32 {
}

impl DataType for f64 {
}

impl DataType for i8 {
}

impl DataType for i16 {
}

impl DataType for i32 {
}

impl DataType for i64 {
}

impl DataType for Pointer {
}

impl DataType for Struct {
}

impl DataType for u8 {
}

impl DataType for u16 {
}

impl DataType for u32 {
}

impl DataType for u64 {
}
