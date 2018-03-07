
use std::vec::Vec;
use std::ops::{Deref, Range, DerefMut};
use std::mem::transmute;
pub struct V8(Vec<u8>);

impl V8 {
	pub fn with_capacity(capacity: usize) -> V8{
		V8(Vec::with_capacity(capacity))
	}

	pub fn new() -> V8{
		V8(Vec::new())
	}
}

pub trait DataView {
	fn set_lu8(&mut self, u8, usize);

	fn set_lu16(&mut self, u16, usize);

	fn set_lu32(&mut self, u32, usize);

	fn set_lu64(&mut self, u64, usize);

	fn set_li8(&mut self, i8, usize);

	fn set_li16(&mut self, i16, usize);

	fn set_li32(&mut self, i32, usize);

	fn set_li64(&mut self, i64, usize);

	fn set_lf32(&mut self, f32, usize);

	fn set_lf64(&mut self, f64, usize);

	fn set_bu8(&mut self, u8, usize);

	fn set_bu16(&mut self, u16, usize);

	fn set_bu32(&mut self, u32, usize);

	fn set_bu64(&mut self, u64, usize);

	fn set_bi8(&mut self, i8, usize);

	fn set_bi16(&mut self, i16, usize);

	fn set_bi32(&mut self, i32, usize);

	fn set_bi64(&mut self, i64, usize);

	fn set_bf32(&mut self, f32, usize);

	fn set_bf64(&mut self, f64, usize);

	fn get_lu8(&mut self, usize) -> u8;

	fn get_lu16(&mut self, usize) -> u16;

	fn get_lu32(&mut self, usize) -> u32;

	fn get_lu64(&mut self, usize) -> u64;

	fn get_li8(&mut self, usize) -> i8;

	fn get_li16(&mut self, usize) -> i16;

	fn get_li32(&mut self, usize) -> i32;

	fn get_li64(&mut self, usize) -> i64;

	fn get_lf32(&mut self, usize) -> f32;

	fn get_lf64(&mut self, usize) -> f64;

	fn get_bu8(&mut self, usize) -> u8;

	fn get_bu16(&mut self, usize) -> u16;

	fn get_bu32(&mut self, usize) -> u32;

	fn get_bu64(&mut self, usize) -> u64;

	fn get_bi8(&mut self, usize) -> i8;

	fn get_bi16(&mut self, usize) -> i16;

	fn get_bi32(&mut self, usize) -> i32;

	fn get_bi64(&mut self, usize) -> i64;

	fn get_bf32(&mut self, usize) -> f32;

	fn get_bf64(&mut self, usize) -> f64;

	fn set(&mut self, &[u8], usize);
	fn move_part(&mut self, Range<usize>, usize);
}

impl Deref for V8 {
    type Target = Vec<u8>;

    fn deref(&self) -> &Vec<u8> {
        &self.0
    }
}

impl DerefMut for V8 {
	fn deref_mut(&mut self) -> &mut Vec<u8>{
		&mut self.0
	}
}

impl DataView for V8 {
	fn set_lu8(&mut self, v: u8, offset: usize){
		unsafe { 
			let l = self.len();
			self.set_len(l + 1);
			*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut u8) = v.to_le() 
		}
	}

	fn set_lu16(&mut self, v: u16, offset: usize){
		unsafe {
			let l = self.len();
			self.set_len(l + 2);
			*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut u16) = v.to_le()
		}
	}

	fn set_lu32(&mut self, v: u32, offset: usize){
		unsafe {
			let l = self.len();
			self.set_len(l + 4);
			*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut u32) = v.to_le()
		}
	}

	fn set_lu64(&mut self, v: u64, offset: usize){
		unsafe {
			let l = self.len();
			self.set_len(l + 8);
			*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut u64) = v.to_le()
		}
	}

	fn set_li8(&mut self, v: i8, offset: usize){
		unsafe {
			let l = self.len();
			self.set_len(l + 1);
			*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut i8) = v.to_le()
		}
	}

	fn set_li16(&mut self, v: i16, offset: usize){
		unsafe {
			let l = self.len();
			self.set_len(l + 2);
			*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut i16) = v.to_le()
		}
	}

	fn set_li32(&mut self, v: i32, offset: usize){
		unsafe {
			let l = self.len();
			self.set_len(l + 4);
			*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut i32) = v.to_le()
		}
	}

	fn set_li64(&mut self, v: i64, offset: usize){
		unsafe {
			let l = self.len();
			self.set_len(l + 8);
			*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut i64) = v.to_le()
		}
	}

	fn set_lf32(&mut self, v: f32, offset: usize){
		unsafe {
			let l = self.len();
			self.set_len(l + 4);
			*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut u32) = transmute::<f32, u32>(v).to_le()
		}
	}

	fn set_lf64(&mut self, v: f64, offset: usize){
		unsafe {
			let l = self.len();
			self.set_len(l + 8);
			*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut u64) = transmute::<f64, u64>(v).to_le()
		}
	}

	fn set_bu8(&mut self, v: u8, offset: usize){
		unsafe {
			let l = self.len();
			self.set_len(l + 1);
			*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut u8) = v.to_be()
		}
	}

	fn set_bu16(&mut self, v: u16, offset: usize){
		unsafe {
			let l = self.len();
			self.set_len(l + 2);
			*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut u16) = v.to_be()
		}
	}

	fn set_bu32(&mut self, v: u32, offset: usize){
		unsafe {
			let l = self.len();
			self.set_len(l + 4);
			*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut u32) = v.to_be()
		}
	}

	fn set_bu64(&mut self, v: u64, offset: usize){
		unsafe {
			let l = self.len();
			self.set_len(l + 8);
			*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut u64) = v.to_be()
		}
	}

	fn set_bi8(&mut self, v: i8, offset: usize){
		unsafe {
			let l = self.len();
			self.set_len(l + 1);
			*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut i8) = v.to_be()
		}
	}

	fn set_bi16(&mut self, v: i16, offset: usize){
		unsafe {
			let l = self.len();
			self.set_len(l + 2);
			*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut i16) = v.to_be()
		}
	}

	fn set_bi32(&mut self, v: i32, offset: usize){
		unsafe {
			let l = self.len();
			self.set_len(l + 4);
			*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut i32) = v.to_be()
		}
	}

	fn set_bi64(&mut self, v: i64, offset: usize){
		unsafe {
			let l = self.len();
			self.set_len(l + 8);
			*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut i64) = v.to_be()
		}
	}

	fn set_bf32(&mut self, v: f32, offset: usize){
		unsafe {
			let l = self.len();
			self.set_len(l + 4);
			*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut u32) = transmute::<f32, u32>(v).to_be()
		}
	}

	fn set_bf64(&mut self, v: f64, offset: usize){
		unsafe {
			let l = self.len();
			self.set_len(l + 8);
			*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut u64) = transmute::<f64, u64>(v).to_be()
		}
	}

	fn get_lu8(&mut self, offset: usize) -> u8{
		unsafe { *(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut u8) }
	}

	fn get_lu16(&mut self, offset: usize) -> u16{
		unsafe { u16::from_le(*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut u16)) }
	}

	fn get_lu32(&mut self, offset: usize) -> u32{
		unsafe { u32::from_le(*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut u32)) }
	}

	fn get_lu64(&mut self, offset: usize) -> u64{
		unsafe { u64::from_le(*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut u64)) }
	}

	fn get_li8(&mut self, offset: usize) -> i8{
		unsafe { i8::from_le(*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut i8)) }
	}

	fn get_li16(&mut self, offset: usize) -> i16{
		unsafe { i16::from_le(*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut i16)) }
	}

	fn get_li32(&mut self, offset: usize) -> i32{
		unsafe { i32::from_le(*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut i32)) }
	}

	fn get_li64(&mut self, offset: usize) -> i64{
		unsafe { i64::from_le(*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut i64)) }
	}

	fn get_lf32(&mut self, offset: usize) -> f32{
		unsafe { transmute::<u32, f32>(u32::from_le(*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut u32))) }
	}

	fn get_lf64(&mut self, offset: usize) -> f64{
		unsafe { transmute::<u64, f64>(u64::from_le(*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut u64)))  }
	}

	fn get_bu8(&mut self, offset: usize) -> u8{
		unsafe { *(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut u8) }
	}

	fn get_bu16(&mut self, offset: usize) -> u16{
		unsafe { u16::from_be(*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut u16)) }
	}

	fn get_bu32(&mut self, offset: usize) -> u32{
		unsafe { u32::from_be(*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut u32)) }
	}

	fn get_bu64(&mut self, offset: usize) -> u64{
		unsafe { u64::from_be(*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut u64)) }
	}

	fn get_bi8(&mut self, offset: usize) -> i8{
		unsafe { *(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut i8) }
	}

	fn get_bi16(&mut self, offset: usize) -> i16{
		unsafe { i16::from_be(*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut i16)) }
	}

	fn get_bi32(&mut self, offset: usize) -> i32{
		unsafe { i32::from_be(*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut i32)) }
	}

	fn get_bi64(&mut self, offset: usize) -> i64{
		unsafe { i64::from_be(*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut i64)) }
	}

	fn get_bf32(&mut self, offset: usize) -> f32{
		unsafe { transmute::<u32, f32>(u32::from_be(*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut u32))) }
	}

	fn get_bf64(&mut self, offset: usize) -> f64{
		unsafe { transmute::<u64, f64>(u64::from_be(*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut u64)))  }
	}
	
	fn set(&mut self, data: &[u8], offset: usize) {
		unsafe{ 
			let len = self.len();
			let dl = data.len();
			if len < offset + dl{
				self.set_len(offset + dl);
			}
			data.as_ptr().copy_to(self.as_mut_ptr().wrapping_offset(offset as isize), dl)
		}
	}

	fn move_part(&mut self, range: Range<usize>, offset: usize) {
		unsafe{
			let len = self.len();
			let dl = range.end - range.start;
			if len < offset + dl{
				self.set_len(offset + dl);
			}
			let src = self.as_mut_ptr();
			src.wrapping_offset(range.start as isize).copy_to(src.wrapping_offset(offset as isize), dl)
		}
	}
}