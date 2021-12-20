use byteorder::{
	BE,
	ReadBytesExt,
	WriteBytesExt
};

use std::{
	fs::{
		File,
		OpenOptions,
		read,
		write
	},
	io::{
		Read,
		Seek,
		SeekFrom,
		self,
		Write
	}
};

use thiserror::Error;

use rgk_core::tag4;

pub const MAGIC: [u8; 5] = [b'P', b'A', b'T', b'C', b'H'];
pub const EOF_MARKER: u32 = tag4!(b"\x00EOF");
pub const MAX_OFFSET: u32 = 16842750; // ~16 MB

#[derive(Clone, Debug, PartialEq)]
pub enum RecData {
	Uncompressed(Vec<u8>),
	RLE(u16, u8),
}

impl RecData {
	#[cfg(feature = "import")]
	fn read<R>(length: usize, buf: &mut R) -> Result<RecData, IPSImportError>
	where
		R: ReadBytesExt,
	{
		if length > 0 { // uncompressed
			let mut data = vec![0; length];
			buf.read_exact(&mut data)?;
			return Ok(RecData::Uncompressed(data));
		} else { // RLE encoded
			return Ok(RecData::RLE(buf.read_u16::<BE>()?, buf.read_u8()?));
		}
	}

	#[cfg(feature = "export")]
	pub fn write<W>(&self, buf: &mut W) -> io::Result<()>
	where
		W: WriteBytesExt,
	{
		match self {
			RecData::Uncompressed(data) => buf.write_all(data)?,
			RecData::RLE(count, value) => {
				buf.write_u16::<BE>(*count)?;

				for _ in 0..(*count as usize) {
					buf.write_u8(*value)?
				}
			},
		}

		Ok(())
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct Record {
	pub offset: u32,
	pub size: u16,
	pub data: RecData,
}

impl Record {
	#[cfg(feature = "import")]
	fn read<R>(offset: u32, buf: &mut R) -> Result<Record, IPSImportError>
	where
		R: ReadBytesExt,
	{
		let size = buf.read_u16::<BE>()?;

		Ok(Record {
			offset: offset,
			size: size,
			data: RecData::read(size as usize, buf)?,
		})
	}

	#[cfg(feature = "export")]
	pub fn write<W>(&self, buf: &mut W) -> io::Result<()>
	where
		W: WriteBytesExt,
	{
		buf.write_u24::<BE>(self.offset)?;
		buf.write_u16::<BE>(self.size)?;
		self.data.write(buf)
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct IPSPatch {
	pub magic: [u8; 5],
	pub records: Vec<Record>,
}

impl IPSPatch {
	/// Constructs a new patch from the source and target files
	#[cfg(feature = "export")]
	pub fn new<'a>(source: &'a str, target: &'a str) -> io::Result<IPSPatch> {
		let src = File::open(source)?;
		let tgt = File::open(target)?;
		let mut recs = vec![];

		let mut diff_buf = vec![];

		// we need to compare the offsets as well as the bytes
		for ((_, sb), (ti, tb)) in src.bytes().enumerate().zip(tgt.bytes().enumerate()) {
			let src_b = sb?;
			let tgt_b = tb?;

			if src_b != tgt_b {
				diff_buf.push((ti, tgt_b));
			} else {
				// examine diff buffer before we continue
				if diff_buf.len() > 0 {
					let mut last_byte = None;
					let mut rep_count = 0;
					let mut cache: Vec<(usize, u8)> = vec![]; // to cache non-repeating byte strings

					// try to use RLE compression
					for (offs, b) in diff_buf.iter() {
						if let Some(lb) = last_byte {
							if *b == lb {
								if cache.len() > 0 {
									// handle the cached byte string before continuing
									recs.push(Record {
										offset: cache[0].0 as u32,
										size: cache.len() as u16,
										data: RecData::Uncompressed(cache.iter()
											.map(|(_, cb)| *cb).collect()),
									});

									cache.clear();
								}

								if rep_count > 0 {
									rep_count += 1;
								} else {
									rep_count = 2;
								}
							} else {
								if rep_count > 0 {
									// push an RLE record
									recs.push(Record {
										offset: cache[0].0 as u32,
										size: 0,
										data: RecData::RLE(rep_count as u16, lb),
									});

									rep_count = 0;
									cache.push((*offs, *b)); // re-push since 'b' will have differentiated
								} else {
									cache.push((*offs, *b)); // just another byte
								}
							}
						} else {
							cache.push((*offs, *b));
						}

						last_byte = Some(*b);
					}

					if cache.len() > 0 {
						// take care of any data left in cache
						recs.push(Record {
							offset: cache[0].0 as u32,
							size: cache.len() as u16,
							data: RecData::Uncompressed(cache.iter().map(|(_, cb)| *cb).collect()),
						});
					}

					diff_buf.clear();
				}
			}
		}

		Ok(IPSPatch {
			magic: MAGIC,
			records: recs,
		})
	}

	/// Applies this patch to a file
	pub fn apply<'a, 'b>(&self, input: &'a str, output: &'b str) -> io::Result<()> {
		// copy the contents
		write(output, read(input)?)?;

		// patch the copy
		let mut patched = OpenOptions::new().append(true).open(output)?;
		for rec in self.records.iter() {
			let _ = patched.seek(SeekFrom::Start(rec.offset as u64))?;

			match &rec.data {
				RecData::Uncompressed(data) => patched.write_all(&data)?,
				RecData::RLE(length, value) => {
					for _ in 0..(*length as usize) {
						patched.write_u8(*value)?;
					}
				},
			};
		}

		Ok(())
	}

	#[cfg(feature = "import")]
	pub fn read<R>(buf: &mut R) -> Result<IPSPatch, IPSImportError>
	where
		R: ReadBytesExt,
	{
		let mut magic = [0; 5];
		buf.read_exact(&mut magic)?;
		if magic != MAGIC {
			return Err(IPSImportError::Magic(magic));
		}

		let mut recs = vec![];

		while let Ok(offset) = buf.read_u24::<BE>() {
			if offset > MAX_OFFSET {
				return Err(IPSImportError::Offset(offset));
			}

			if offset != EOF_MARKER {
				recs.push(Record::read(offset, buf)?);
			}
		}

		Ok(IPSPatch {
			magic: magic,
			records: recs,
		})
	}

	#[cfg(feature = "export")]
	pub fn write<W>(&self, buf: &mut W) -> io::Result<()>
	where
		W: WriteBytesExt,
	{
		buf.write_all(&self.magic)?;

		for rec in self.records.iter() {
			rec.write(buf)?;
		}

		buf.write_u24::<BE>(EOF_MARKER)
	}
}

#[cfg(feature = "import")]
#[derive(Debug, Error)]
pub enum IPSImportError {
	#[error("I/O error")]
	IO {
		#[from]
		source: io::Error,
	},
	#[error("Not an IPS Patch")]
	Magic([u8; 5]),
	#[error("Max offset exceeded: {0} bytes > 16842750 bytes")]
	Offset(u32),
}

#[cfg(feature = "export")]
#[test]
fn test_create_ips() {
	let ips = IPSPatch::new("test_data/test.dat", "test_data/test2.dat").unwrap();
	println!("{:#?}", ips);
}

#[cfg(feature = "export")]
#[test]
fn test_write_ips() {
	use std::fs::File;

	let ips = IPSPatch::new("test_data/test.dat", "test_data/test2.dat").unwrap();
	let mut f = File::create("test_data/test_write.ips").unwrap();
	ips.write(&mut f).unwrap();
}
