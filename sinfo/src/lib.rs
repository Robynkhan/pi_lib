/**
 * 结构体信息
 */

extern crate atom;
extern crate bon;

use std::vec::Vec;
use std::collections::HashMap;
use std::sync::Arc;

use atom::Atom;
use bon::{WriteBuffer, ReadBuffer, Encode, Decode, ReadBonErr};

// 枚举结构体字段的所有类型
#[derive(Debug)]
pub enum EnumType {
	Bool,
	U8,
	U16,
	U32,
	U64,
	U128,
	U256,
	Usize,
	I8,
	I16,
	I32,
	I64,
	I128,
	I256,
	Isize,
	F32,
	F64,
	BigI,
	Str,
	Bin,
	Arr(Arc<EnumType>),
	Map(Arc<EnumType>, Arc<EnumType>),
	Struct(Arc<StructInfo>),
	Option(Arc<EnumType>),
	Enum(Arc<EnumInfo>)
}

impl Encode for EnumType{
	fn encode(&self, bb:&mut WriteBuffer){
		match self{
			&EnumType::Bool => 0.encode(bb),
			&EnumType::U8 => 1.encode(bb),
			&EnumType::U16 => 2.encode(bb),
			&EnumType::U32 => 3.encode(bb),
			&EnumType::U64 => 4.encode(bb),
			&EnumType::U128 => 5.encode(bb),
			&EnumType::U256 => 6.encode(bb),
			&EnumType::Usize => 7.encode(bb),
			&EnumType::I8 => 8.encode(bb),
			&EnumType::I16 => 9.encode(bb),
			&EnumType::I32 => 10.encode(bb),
			&EnumType::I64 => 11.encode(bb),
			&EnumType::I128 => 12.encode(bb),
			&EnumType::I256 => 13.encode(bb),
			&EnumType::Isize => 14.encode(bb),
			&EnumType::F32 => 15.encode(bb),
			&EnumType::F64 => 16.encode(bb),
			&EnumType::BigI => 17.encode(bb),
			&EnumType::Str => 18.encode(bb),
			&EnumType::Bin => 19.encode(bb),
			&EnumType::Arr(ref v) => {20.encode(bb); v.encode(bb);},
			&EnumType::Map(ref k, ref v) => {21.encode(bb); k.encode(bb); v.encode(bb);},
			&EnumType::Struct(ref v) => {22.encode(bb); v.encode(bb);},
			&EnumType::Option(ref v) => {23.encode(bb); v.encode(bb);},
			&EnumType::Enum(ref v) => {24.encode(bb); v.encode(bb);},
		};
	}
}

impl Decode for EnumType{
	fn decode(bb:&mut ReadBuffer) -> Result<EnumType, ReadBonErr> {
		let t = u8::decode(bb)?;
		match t{
			0 => Ok(EnumType::Bool),
			1 => Ok(EnumType::U8),
			2 => Ok(EnumType::U16),
			3 => Ok(EnumType::U32),
			4 => Ok(EnumType::U64),
			5 => Ok(EnumType::U128),
			6 => Ok(EnumType::U256),
			7 => Ok(EnumType::Usize),
			8 => Ok(EnumType::I8),
			9 => Ok(EnumType::I16),
			10 => Ok(EnumType::I32),
			11 => Ok(EnumType::I64),
			12 => Ok(EnumType::I128),
			13 => Ok(EnumType::I256),
			14 => Ok(EnumType::Isize),
			15 => Ok(EnumType::F32),
			16 => Ok(EnumType::F64),
			17 => Ok(EnumType::BigI),
			18 => Ok(EnumType::Str),
			19 => Ok(EnumType::Bin),
			20 => Ok(EnumType::Arr(Arc::new(EnumType::decode(bb)?))),
			21 => Ok(EnumType::Map(Arc::new(EnumType::decode(bb)?), Arc::new(EnumType::decode(bb)?))),
			22 => Ok(EnumType::Struct(Arc::new(StructInfo::decode(bb)?))),
			23 => Ok(EnumType::Option(Arc::new(EnumType::decode(bb)?))),
			24 => Ok(EnumType::Enum(Arc::new(EnumInfo::decode(bb)?))),
			_ => panic!("EnumType is not exist:{}", t)
		}
	}
}

/**
* 自定义对象序列化元信息
*/
#[derive(Debug)]
pub struct StructInfo {
	pub name: Atom,
	pub name_hash: u32,
	pub notes: Option<HashMap<Atom, Atom>>,
	pub fields: Vec<FieldInfo>,
}

impl StructInfo {
	/**
	* 构建自定义对象序列化元信息
	* @param name 自定义对象名称
	* @param name_hash 自定义对象名称hash
	* @returns 返回自定义对象序列化元信息
	*/
	pub fn new(name:Atom, name_hash:u32) -> Self {
		StructInfo {
			name:name,
			name_hash: name_hash,
			notes: None,
			fields: Vec::new(),
		}
	}
	pub fn get_note(&self, key: &Atom) -> Option<&Atom> {
		match self.notes {
			Some(ref map) => map.get(key),
			_ => None
		}
	}
}

impl Encode for StructInfo{
	fn encode(&self, bb: &mut WriteBuffer){
		self.name.encode(bb);
		self.name_hash.encode(bb);
        self.notes.encode(bb);
        self.fields.encode(bb);
	}
}

impl Decode for StructInfo{
	fn decode(bb: &mut ReadBuffer) -> Result<StructInfo, ReadBonErr> {
		Ok(StructInfo{
			name: Atom::decode(bb)?,
			name_hash: u32::decode(bb)?,
			notes: Option::decode(bb)?,
			fields: Vec::decode(bb)?,
		})
	}
}

#[derive(Debug)]
pub struct FieldInfo {
	pub name: Atom,
	pub ftype: EnumType,
	pub notes: Option<HashMap<Atom, Atom>>,
}


impl FieldInfo{
	pub fn get_note(&self, key: &Atom) -> Option<&Atom> {
		match self.notes {
			Some(ref map) => map.get(key),
			_ => None
		}
	}
}
impl Encode for FieldInfo{
	fn encode(&self, bb: &mut WriteBuffer){
		self.name.encode(bb);
		self.ftype.encode(bb);
		self.notes.encode(bb);
	}
}

impl Decode for FieldInfo{
	fn decode(bb: &mut ReadBuffer) -> Result<FieldInfo, ReadBonErr> {
		let n = Atom::decode(bb)?;
		Ok(FieldInfo{
			name: n,
			ftype: EnumType::decode(bb)?,
			notes: Option::decode(bb)?,
		})
	}
}

#[derive(Debug)]
pub struct EnumInfo {
	pub name: Atom,
	pub name_hash: u32,
	pub notes: Option<HashMap<Atom, Atom>>,
	pub members: Vec<Option<EnumType>>,
}

impl EnumInfo {
	pub fn new(name:Atom, name_hash:u32) -> Self {
		EnumInfo {
			name:name,
			name_hash: name_hash,
			notes: None,
			members: Vec::new(),
		}
	}
}

impl Encode for EnumInfo{
	fn encode(&self, bb: &mut WriteBuffer){
		self.name.encode(bb);
		self.name_hash.encode(bb);
        self.notes.encode(bb);
        self.members.encode(bb);
	}
}

impl Decode for EnumInfo{
	fn decode(bb: &mut ReadBuffer) -> Result<EnumInfo, ReadBonErr>{
		let n = Atom::decode(bb)?;
		Ok(EnumInfo{
			name: n,
			name_hash: u32::decode(bb)?,
			notes: Option::decode(bb)?,
			members: Vec::decode(bb)?,
		})
	}
}

