use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
use std::io;
use std::mem;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SerError {
    // 0
    #[error("Value is updated")]
    ValueIsUpdated,

    // 1
    #[error("Io error")]
    IoError(std::io::Error),

    // 2
    #[error("Length overflow")]
    LengthOverflow,

    // 3
    #[error("Offset overflow")]
    OffsetOverflow,

    // 4
    #[error("Invalid field access")]
    InvalidFieldAccess,

    // 5
    #[error("Invalid cursor type")]
    InvalidCursorType,
}

impl From<std::io::Error> for SerError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

#[cfg_attr(test, derive(BorshSerialize))]
#[derive(Debug)]
pub enum UpdatableValue<T>
where
    T: BorshDeserialize + BorshSerialize,
{
    Origin(u16, u16),
    Updated(T),
}

impl<T> UpdatableValue<T>
where
    T: BorshDeserialize + BorshSerialize,
{
    pub fn origin(start: usize, len: usize) -> Self {
        Self::Origin(start as u16, len as u16)
    }

    pub fn updated(value: T) -> Self {
        Self::Updated(value)
    }

    pub fn get_offset_and_len(&self) -> Result<(usize, usize), SerError> {
        match &self {
            UpdatableValue::Origin(origin_offset, origin_len) => {
                Ok((*origin_offset as usize, *origin_len as usize))
            }
            UpdatableValue::Updated(_) => Err(SerError::ValueIsUpdated),
        }
    }

    pub fn get(&self, src: &[u8]) -> Result<T, SerError> {
        let (offset, len) = self.get_offset_and_len()?;
        Ok(T::try_from_slice(&src[offset..(offset + len)])?)
    }
}
pub trait Writable {
    fn size(&self) -> usize;
    fn write(self, src: &[u8], dest: &mut [u8], offset: usize) -> usize;
}

#[cfg_attr(test, derive(BorshSerialize))]
#[derive(Debug)]
pub enum CursorType {
    Bool,
    U8,
    U16,
    U32,
    Usize,
    U64,
    String,
    Struct(Vec<CursorType>), // the fields
    Vec(Box<CursorType>),    // the items
    StaticVec,               // items with fixed size
    Option(Box<CursorType>), // the inner type
    Enum(Vec<CursorType>),   // the variants
    Pubkey,
    Empty,
}

impl CursorType {
    pub fn mk_vec(item_type: CursorType) -> Self {
        Self::Vec(Box::new(item_type))
    }

    pub fn mk_option(inner_type: CursorType) -> Self {
        Self::Option(Box::new(inner_type))
    }

    pub fn mk_struct(field_types: Vec<CursorType>) -> Self {
        Self::Struct(field_types)
    }

    pub fn mk_enum(variants_types: Vec<CursorType>) -> Self {
        Self::Enum(variants_types)
    }
}

#[cfg_attr(test, derive(BorshSerialize))]
#[derive(Debug)]
pub enum Cursor {
    Bool(PrimitiveCursor<bool>),
    U8(PrimitiveCursor<u8>),
    U16(PrimitiveCursor<u16>),
    U32(PrimitiveCursor<u32>),
    Usize(PrimitiveCursor<usize>),
    U64(PrimitiveCursor<u64>),
    String(StringCursor),
    Struct(StructCursor),
    Vec(VecCursor),
    StaticVec(StaticVecCursor),
    Option(OptionCursor),
    Enum(EnumCursor),
    Pubkey(PubkeyCursor),
    Empty(EmptyCursor),
}

impl Cursor {
    pub fn new(cursor_type: &CursorType, src: &[u8], offset: usize) -> (Self, usize) {
        #[cfg(test)]
        println!("{} - new cursor: {:?}", offset, cursor_type);
        match cursor_type {
            CursorType::Bool => {
                let (c, offset) = PrimitiveCursor::<bool>::new(src, offset);
                (Cursor::Bool(c), offset)
            }
            CursorType::U8 => {
                let (c, offset) = PrimitiveCursor::<u8>::new(src, offset);
                (Cursor::U8(c), offset)
            }
            CursorType::U16 => {
                let (c, offset) = PrimitiveCursor::<u16>::new(src, offset);
                (Cursor::U16(c), offset)
            }
            CursorType::U32 => {
                let (c, offset) = PrimitiveCursor::<u32>::new(src, offset);
                (Cursor::U32(c), offset)
            }
            CursorType::Usize => {
                let (c, offset) = PrimitiveCursor::<usize>::new(src, offset);
                (Cursor::Usize(c), offset)
            }
            CursorType::U64 => {
                let (c, offset) = PrimitiveCursor::<u64>::new(src, offset);
                (Cursor::U64(c), offset)
            }
            CursorType::String => {
                let (c, offset) = StringCursor::new(src, offset);
                (Cursor::String(c), offset)
            }
            CursorType::Struct(field_types) => {
                let (c, offset) = StructCursor::new(field_types, src, offset);
                (Cursor::Struct(c), offset)
            }
            CursorType::Vec(item_type) => {
                let (c, offset) = VecCursor::new(&item_type, src, offset);
                (Cursor::Vec(c), offset)
            }
            CursorType::StaticVec => {
                let (c, offset) = StaticVecCursor::new(src, offset);
                (Cursor::StaticVec(c), offset)
            }
            CursorType::Option(inner_type) => {
                let (c, offset) = OptionCursor::new(&inner_type, src, offset);
                (Cursor::Option(c), offset)
            }
            CursorType::Enum(variants) => {
                let (c, offset) = EnumCursor::new(&variants, src, offset);
                (Cursor::Enum(c), offset)
            }
            CursorType::Pubkey => {
                let (c, offset) = PubkeyCursor::new(src, offset);
                (Cursor::Pubkey(c), offset)
            }
            CursorType::Empty => {
                let c = EmptyCursor {};
                (Cursor::Empty(c), 0)
            }
        }
    }

    pub fn size(&self) -> usize {
        match self {
            Cursor::Bool(c) => c.size(),
            Cursor::U8(c) => c.size(),
            Cursor::U16(c) => c.size(),
            Cursor::U32(c) => c.size(),
            Cursor::Usize(c) => c.size(),
            Cursor::U64(c) => c.size(),
            Cursor::String(c) => c.size(),
            Cursor::Struct(c) => c.size(),
            Cursor::Vec(c) => c.size(),
            Cursor::StaticVec(c) => c.size(),
            Cursor::Option(c) => c.size(),
            Cursor::Enum(c) => c.size(),
            Cursor::Pubkey(c) => c.size(),
            Cursor::Empty(c) => c.size(),
        }
    }

    pub fn write(self, src: &[u8], dest: &mut [u8], offset: usize) -> usize {
        #[cfg(test)]
        println!("{} - write: {:?}", offset, self);
        match self {
            Cursor::Bool(c) => c.write(src, dest, offset),
            Cursor::U8(c) => c.write(src, dest, offset),
            Cursor::U16(c) => c.write(src, dest, offset),
            Cursor::U32(c) => c.write(src, dest, offset),
            Cursor::Usize(c) => c.write(src, dest, offset),
            Cursor::U64(c) => c.write(src, dest, offset),
            Cursor::String(c) => c.write(src, dest, offset),
            Cursor::Struct(c) => c.write(src, dest, offset),
            Cursor::Vec(c) => c.write(src, dest, offset),
            Cursor::StaticVec(c) => c.write(src, dest, offset),
            Cursor::Option(c) => c.write(src, dest, offset),
            Cursor::Enum(c) => c.write(src, dest, offset),
            Cursor::Pubkey(c) => c.write(src, dest, offset),
            Cursor::Empty(c) => c.write(src, dest, offset),
        }
    }
}

impl<'a> TryFrom<&'a Cursor> for &'a PrimitiveCursor<u8> {
    type Error = SerError;

    fn try_from(cursor: &'a Cursor) -> Result<Self, SerError> {
        match cursor {
            Cursor::U8(c) => Ok(c),
            _ => Err(SerError::InvalidCursorType),
        }
    }
}

impl<'a> TryFrom<&'a mut Cursor> for &'a mut PrimitiveCursor<u8> {
    type Error = SerError;

    fn try_from(cursor: &'a mut Cursor) -> Result<Self, SerError> {
        match cursor {
            Cursor::U8(c) => Ok(c),
            _ => Err(SerError::InvalidCursorType),
        }
    }
}

impl<'a> TryFrom<&'a Cursor> for &'a PrimitiveCursor<u16> {
    type Error = SerError;

    fn try_from(cursor: &'a Cursor) -> Result<Self, SerError> {
        match cursor {
            Cursor::U16(c) => Ok(c),
            _ => Err(SerError::InvalidCursorType),
        }
    }
}

impl<'a> TryFrom<&'a mut Cursor> for &'a mut PrimitiveCursor<u16> {
    type Error = SerError;

    fn try_from(cursor: &'a mut Cursor) -> Result<Self, SerError> {
        match cursor {
            Cursor::U16(c) => Ok(c),
            _ => Err(SerError::InvalidCursorType),
        }
    }
}

impl<'a> TryFrom<&'a Cursor> for &'a PrimitiveCursor<u32> {
    type Error = SerError;

    fn try_from(cursor: &'a Cursor) -> Result<Self, SerError> {
        match cursor {
            Cursor::U32(c) => Ok(c),
            _ => Err(SerError::InvalidCursorType),
        }
    }
}

impl<'a> TryFrom<&'a mut Cursor> for &'a mut PrimitiveCursor<u32> {
    type Error = SerError;

    fn try_from(cursor: &'a mut Cursor) -> Result<Self, SerError> {
        match cursor {
            Cursor::U32(c) => Ok(c),
            _ => Err(SerError::InvalidCursorType),
        }
    }
}

impl<'a> TryFrom<&'a Cursor> for &'a PrimitiveCursor<u64> {
    type Error = SerError;

    fn try_from(cursor: &'a Cursor) -> Result<Self, SerError> {
        match cursor {
            Cursor::U64(c) => Ok(c),
            _ => Err(SerError::InvalidCursorType),
        }
    }
}

impl<'a> TryFrom<&'a mut Cursor> for &'a mut PrimitiveCursor<u64> {
    type Error = SerError;

    fn try_from(cursor: &'a mut Cursor) -> Result<Self, SerError> {
        match cursor {
            Cursor::U64(c) => Ok(c),
            _ => Err(SerError::InvalidCursorType),
        }
    }
}

impl<'a> TryFrom<&'a Cursor> for &'a PrimitiveCursor<usize> {
    type Error = SerError;

    fn try_from(cursor: &'a Cursor) -> Result<Self, SerError> {
        match cursor {
            Cursor::Usize(c) => Ok(c),
            _ => Err(SerError::InvalidCursorType),
        }
    }
}

impl<'a> TryFrom<&'a mut Cursor> for &'a mut PrimitiveCursor<usize> {
    type Error = SerError;

    fn try_from(cursor: &'a mut Cursor) -> Result<Self, SerError> {
        match cursor {
            Cursor::Usize(c) => Ok(c),
            _ => Err(SerError::InvalidCursorType),
        }
    }
}

impl<'a> TryFrom<&'a Cursor> for &'a PrimitiveCursor<bool> {
    type Error = SerError;

    fn try_from(cursor: &'a Cursor) -> Result<Self, SerError> {
        match cursor {
            Cursor::Bool(c) => Ok(c),
            _ => Err(SerError::InvalidCursorType),
        }
    }
}

impl<'a> TryFrom<&'a mut Cursor> for &'a mut PrimitiveCursor<bool> {
    type Error = SerError;

    fn try_from(cursor: &'a mut Cursor) -> Result<Self, SerError> {
        match cursor {
            Cursor::Bool(c) => Ok(c),
            _ => Err(SerError::InvalidCursorType),
        }
    }
}

/// A cursor refers to a primitive type
/// The size of primitive types don't change
#[cfg_attr(test, derive(BorshSerialize))]
#[derive(Debug)]
pub struct PrimitiveCursor<T>
where
    T: BorshDeserialize + BorshSerialize + std::fmt::Debug,
{
    value: T,
}

impl<T> Writable for PrimitiveCursor<T>
where
    T: BorshDeserialize + BorshSerialize + std::fmt::Debug,
{
    fn size(&self) -> usize {
        mem::size_of::<T>()
    }

    fn write(self, _src: &[u8], dest: &mut [u8], offset: usize) -> usize {
        let len = mem::size_of::<T>();
        let buf = &mut dest[offset..(offset + len)];
        let mut w = io::Cursor::new(buf);
        borsh::to_writer(&mut w, &self.value).unwrap();
        len
    }
}

impl<T> PrimitiveCursor<T>
where
    T: BorshDeserialize + BorshSerialize + std::fmt::Debug,
{
    pub fn new(data: &[u8], offset: usize) -> (Self, usize) {
        let len = mem::size_of::<T>();
        let buf = &data[offset..(offset + len)];
        let value = T::try_from_slice(&buf).unwrap();
        (Self { value }, len)
    }
    pub fn get(&self) -> &T {
        &self.value
    }
    pub fn set(&mut self, value: T) {
        self.value = value;
    }
}

#[cfg_attr(test, derive(BorshSerialize))]
#[derive(Debug)]
pub struct StringCursor {
    value: UpdatableValue<String>,
}

impl<'a> TryFrom<&'a Cursor> for &'a StringCursor {
    type Error = SerError;

    fn try_from(cursor: &'a Cursor) -> Result<Self, SerError> {
        match cursor {
            Cursor::String(c) => Ok(c),
            _ => Err(SerError::InvalidCursorType),
        }
    }
}

impl<'a> TryFrom<&'a mut Cursor> for &'a mut StringCursor {
    type Error = SerError;

    fn try_from(cursor: &'a mut Cursor) -> Result<Self, SerError> {
        match cursor {
            Cursor::String(c) => Ok(c),
            _ => Err(SerError::InvalidCursorType),
        }
    }
}

impl Writable for StringCursor {
    fn size(&self) -> usize {
        match self.value {
            UpdatableValue::Origin(_, origin_len) => origin_len as usize,
            UpdatableValue::Updated(ref s) => s.len() + 4,
        }
    }

    fn write(self, src: &[u8], dest: &mut [u8], offset: usize) -> usize {
        match self.value {
            UpdatableValue::Origin(origin_offset, origin_len) => {
                let origin_offset = origin_offset as usize;
                dest[offset..(offset + origin_len as usize)].copy_from_slice(
                    &src[(origin_offset as usize)..(origin_offset as usize + origin_len as usize)],
                );
                origin_len as usize
            }
            UpdatableValue::Updated(ref s) => {
                let len = s.len();
                let buf = &mut dest[offset..(offset + len + 4)];
                let mut w = io::Cursor::new(buf);
                borsh::to_writer(&mut w, &s).unwrap();
                len + 4
            }
        }
    }
}

impl StringCursor {
    fn new(data: &[u8], offset: usize) -> (Self, usize) {
        let len = u32::try_from_slice(&data[offset..(offset + 4)]).unwrap();
        let value = UpdatableValue::origin(offset, len as usize + 4);
        (Self { value }, len as usize + 4)
    }

    pub fn get<'a>(&'a self, data: &'a [u8]) -> &'a str {
        match self.value {
            UpdatableValue::Origin(origin_offset, origin_len) => {
                let origin_offset = origin_offset as usize;
                let buf = &data[(origin_offset + 4)..(origin_offset + origin_len as usize)];
                std::str::from_utf8(buf).unwrap()
            }
            UpdatableValue::Updated(_) => panic!("String is updated"),
        }
    }

    fn set<S: Into<String>>(&mut self, value: S) {
        self.value = UpdatableValue::Updated(value.into());
    }
}

/// A cursor refers to a struct which can be further interpreted as
/// a list of cursors.  Since the size of struct might change,
/// we have a extra buffer for saving bytes which doesn't fit in the
/// original buffer.
#[cfg_attr(test, derive(BorshSerialize))]
#[derive(Debug)]
pub struct StructCursor {
    cursors: Vec<Cursor>,
}

impl<'a> TryFrom<&'a Cursor> for &'a StructCursor {
    type Error = SerError;

    fn try_from(cursor: &'a Cursor) -> Result<Self, SerError> {
        match cursor {
            Cursor::Struct(c) => Ok(c),
            _ => Err(SerError::InvalidCursorType),
        }
    }
}

impl<'a> TryFrom<&'a mut Cursor> for &'a mut StructCursor {
    type Error = SerError;

    fn try_from(cursor: &'a mut Cursor) -> Result<Self, SerError> {
        match cursor {
            Cursor::Struct(c) => Ok(c),
            _ => Err(SerError::InvalidCursorType),
        }
    }
}

impl Writable for StructCursor {
    fn size(&self) -> usize {
        let mut len = 0;
        for cursor in self.cursors.iter() {
            len += cursor.size();
        }
        len
    }

    fn write(self, src: &[u8], dest: &mut [u8], offset: usize) -> usize {
        let mut len = 0;
        for cursor in self.cursors {
            len += cursor.write(src, dest, offset + len);
        }
        len
    }
}

impl StructCursor {
    fn new(cursor_types: &[CursorType], data: &[u8], mut offset: usize) -> (Self, usize) {
        let mut cursors = Vec::new();
        let mut total_len = 0;
        for ct in cursor_types.iter() {
            let (c, len) = Cursor::new(ct, data, offset);
            offset += len;
            total_len += len;
            cursors.push(c);
        }
        (Self { cursors }, total_len)
    }

    pub fn get(&self, field_index: u8) -> Result<&Cursor, SerError> {
        if let Some(cursor) = self.cursors.get(field_index as usize) {
            Ok(cursor)
        } else {
            panic!("Index access out of bound");
        }
    }

    pub fn get_mut(&mut self, field_index: u8) -> Result<&mut Cursor, SerError> {
        if let Some(cursor) = self.cursors.get_mut(field_index as usize) {
            Ok(cursor)
        } else {
            panic!("Index access out of bound");
        }
    }
}

/// We use u8 here to save some bytes
/// since we don't have a list longer than 256.
#[cfg_attr(test, derive(BorshSerialize))]
#[derive(Debug)]
pub struct VecCursor {
    offset: u16,
    cursors: Vec<Cursor>,
    add: Vec<Vec<u8>>, // the new items to insert
}

impl<'a> TryFrom<&'a Cursor> for &'a VecCursor {
    type Error = SerError;

    fn try_from(cursor: &'a Cursor) -> Result<Self, SerError> {
        match cursor {
            Cursor::Vec(c) => Ok(c),
            _ => Err(SerError::InvalidCursorType),
        }
    }
}

impl<'a> TryFrom<&'a mut Cursor> for &'a mut VecCursor {
    type Error = SerError;

    fn try_from(cursor: &'a mut Cursor) -> Result<Self, SerError> {
        match cursor {
            Cursor::Vec(c) => Ok(c),
            _ => Err(SerError::InvalidCursorType),
        }
    }
}

impl Writable for VecCursor {
    fn size(&self) -> usize {
        let mut len = 4;
        for cursor in self.cursors.iter() {
            len += cursor.size();
        }
        for add in self.add.iter() {
            len += add.len();
        }
        len
    }

    fn write(self, src: &[u8], dest: &mut [u8], offset: usize) -> usize {
        let mut w = io::Cursor::new(&mut dest[(offset as usize)..(offset as usize + 4)]);
        borsh::to_writer(&mut w, &(self.cursors.len() as u32 + self.add.len() as u32)).unwrap();
        let mut len = 4;
        for cursor in self.cursors {
            len += cursor.write(src, dest, offset + len);
        }
        for add in self.add {
            let add_len = add.len();
            dest[(offset + len)..(offset + len + add_len)].copy_from_slice(&add);
            len += add_len;
        }

        len
    }
}

impl VecCursor {
    fn new(item_type: &CursorType, data: &[u8], offset: usize) -> (Self, usize) {
        let mut cnt = u32::try_from_slice(&data[offset..(offset + 4)]).unwrap();
        let mut total_len = 0;
        let mut cursors = Vec::with_capacity(cnt as usize);
        while cnt > 0 {
            let (c, len) = Cursor::new(item_type, data, offset + 4 + total_len);
            cursors.push(c);
            total_len += len;
            cnt -= 1;
        }
        (
            Self {
                offset: offset as u16,
                cursors,
                add: Default::default(),
            },
            4 + total_len,
        )
    }

    fn push<T: BorshSerialize>(&mut self, new_item: &T) {
        self.add.push(borsh::to_vec(new_item).unwrap());
    }

    fn delete(&mut self, index: usize) {
        self.cursors.remove(index);
    }

    pub fn get_cursor(&self, cursor_index: usize) -> &Cursor {
        if let Some(cursor) = self.cursors.get(cursor_index) {
            cursor
        } else {
            panic!("Index access out of bound");
        }
    }

    pub fn get_cursor_mut(&mut self, cursor_index: usize) -> &mut Cursor {
        if let Some(cursor) = self.cursors.get_mut(cursor_index) {
            cursor
        } else {
            panic!("Index access out of bound");
        }
    }
}

#[cfg_attr(test, derive(BorshSerialize))]
#[derive(Debug)]
pub enum StaticVecValue {
    Origin(u16, u16), // offset, length
    NewValue(Vec<u8>),
}

#[cfg_attr(test, derive(BorshSerialize))]
#[derive(Debug)]
pub struct StaticVecCursor {
    value: StaticVecValue,
}

impl<'a> TryFrom<&'a Cursor> for &'a StaticVecCursor {
    type Error = SerError;

    fn try_from(cursor: &'a Cursor) -> Result<Self, SerError> {
        match cursor {
            Cursor::StaticVec(c) => Ok(c),
            _ => Err(SerError::InvalidCursorType),
        }
    }
}

impl<'a> TryFrom<&'a mut Cursor> for &'a mut StaticVecCursor {
    type Error = SerError;

    fn try_from(cursor: &'a mut Cursor) -> Result<Self, SerError> {
        match cursor {
            Cursor::StaticVec(c) => Ok(c),
            _ => Err(SerError::InvalidCursorType),
        }
    }
}

impl Writable for StaticVecCursor {
    fn size(&self) -> usize {
        match &self.value {
            StaticVecValue::Origin(_, len) => 4 + *len as usize,
            StaticVecValue::NewValue(v) => 4 + v.len(),
        }
    }

    fn write(self, src: &[u8], dest: &mut [u8], offset: usize) -> usize {
        match self.value {
            StaticVecValue::Origin(origin_offset, len) => {
                let origin_offset = origin_offset as usize;
                let len = len as usize + 4;
                dest[offset..(offset + len)]
                    .copy_from_slice(&src[origin_offset..(origin_offset + len)]);
                len
            }
            StaticVecValue::NewValue(v) => {
                let len = v.len();
                let mut w = io::Cursor::new(&mut dest[offset..(offset + len + 4)]);
                borsh::to_writer(&mut w, &v).unwrap();
                len + 4
            }
        }
    }
}

impl StaticVecCursor {
    pub fn new(data: &[u8], offset: usize) -> (Self, usize) {
        let len = u32::try_from_slice(&data[offset..(offset + 4)]).unwrap();
        (
            Self {
                value: StaticVecValue::Origin(offset as u16, len as u16),
            },
            len as usize + 4,
        )
    }

    pub fn get<'a, 'b>(&'a self, data: &'b [u8]) -> &'b [u8] {
        match &self.value {
            StaticVecValue::Origin(offset, len) => {
                let offset = *offset as usize;
                let len = *len as usize;
                &data[(offset + 4)..(offset + 4 + len)]
            }
            _ => panic!("data is already updated"),
        }
    }

    pub fn set(&mut self, value: Vec<u8>) {
        self.value = StaticVecValue::NewValue(value);
    }
}

#[cfg_attr(test, derive(BorshSerialize))]
#[derive(Debug)]
pub struct OptionCursor {
    offset: u16,
    inner: Option<Box<Cursor>>,
}

impl<'a> TryFrom<&'a Cursor> for &'a OptionCursor {
    type Error = SerError;

    fn try_from(cursor: &'a Cursor) -> Result<Self, SerError> {
        match cursor {
            Cursor::Option(c) => Ok(c),
            _ => Err(SerError::InvalidCursorType),
        }
    }
}

impl<'a> TryFrom<&'a mut Cursor> for &'a mut OptionCursor {
    type Error = SerError;

    fn try_from(cursor: &'a mut Cursor) -> Result<Self, SerError> {
        match cursor {
            Cursor::Option(c) => Ok(c),
            _ => Err(SerError::InvalidCursorType),
        }
    }
}

impl Writable for OptionCursor {
    fn size(&self) -> usize {
        if let Some(inner_cursor) = self.inner.as_ref() {
            1 + inner_cursor.size()
        } else {
            1
        }
    }

    fn write(self, src: &[u8], dest: &mut [u8], offset: usize) -> usize {
        if let Some(inner_cursor) = self.inner {
            dest[offset as usize] = 1;
            inner_cursor.write(src, dest, offset + 1) + 1
        } else {
            dest[offset] = 0;
            1
        }
    }
}

impl OptionCursor {
    fn new(inner_type: &CursorType, data: &[u8], offset: usize) -> (Self, usize) {
        if data[offset] == 0 {
            (
                Self {
                    offset: offset as u16,
                    inner: None,
                },
                1,
            )
        } else {
            let (c, len) = Cursor::new(inner_type, data, offset + 1);
            (
                Self {
                    offset: offset as u16,
                    inner: Some(Box::new(c)),
                },
                1 + len,
            )
        }
    }

    // TODO: remove a inner cursor or create a inner cursor

    pub fn get_inner(&self) -> Option<&Cursor> {
        if let Some(c) = self.inner.as_ref() {
            Some(c)
        } else {
            None
        }
    }

    pub fn get_inner_mut(&mut self) -> Option<&mut Cursor> {
        if let Some(c) = self.inner.as_mut() {
            Some(c)
        } else {
            None
        }
    }
}

#[cfg_attr(test, derive(BorshSerialize))]
#[derive(Debug)]
pub struct EnumCursor {
    value: UpdatableValue<Vec<u8>>,
}

impl<'a> TryFrom<&'a Cursor> for &'a EnumCursor {
    type Error = SerError;

    fn try_from(cursor: &'a Cursor) -> Result<Self, SerError> {
        match cursor {
            Cursor::Enum(c) => Ok(c),
            _ => Err(SerError::InvalidCursorType),
        }
    }
}

impl<'a> TryFrom<&'a mut Cursor> for &'a mut EnumCursor {
    type Error = SerError;

    fn try_from(cursor: &'a mut Cursor) -> Result<Self, SerError> {
        match cursor {
            Cursor::Enum(c) => Ok(c),
            _ => Err(SerError::InvalidCursorType),
        }
    }
}

impl Writable for EnumCursor {
    fn size(&self) -> usize {
        match &self.value {
            UpdatableValue::Origin(_, len) => *len as usize,
            UpdatableValue::Updated(v) => v.len(),
        }
    }

    fn write(self, src: &[u8], dest: &mut [u8], offset: usize) -> usize {
        match self.value {
            UpdatableValue::Origin(origin_offset, len) => {
                let origin_offset = origin_offset as usize;
                let len = len as usize;
                dest[offset..(offset + len)].copy_from_slice(
                    &src[(origin_offset as usize)..(origin_offset as usize + len as usize)],
                );
                len
            }
            UpdatableValue::Updated(v) => {
                let len = v.len();
                dest[offset..(offset + len)].copy_from_slice(&v);
                len
            }
        }
    }
}

impl EnumCursor {
    fn new(variants: &[CursorType], data: &[u8], offset: usize) -> (Self, usize) {
        let descriminator = data[offset];
        let cursor_type = variants.get(descriminator as usize).unwrap();
        let (_, inner_size) = Cursor::new(cursor_type, data, offset + 1);
        (
            Self {
                value: UpdatableValue::origin(offset, inner_size + 1),
            },
            inner_size + 1,
        )
    }

    pub fn get<T: BorshDeserialize>(&self, data: &[u8]) -> Result<T, SerError> {
        let (offset, len) = self.value.get_offset_and_len()?;
        Ok(T::try_from_slice(&data[offset..(offset + len)])?)
    }

    pub fn set<T: BorshSerialize>(&mut self, value: &T) {
        let v = borsh::to_vec(value).unwrap();
        self.value = UpdatableValue::updated(v);
    }
}

#[cfg_attr(test, derive(BorshSerialize, BorshDeserialize))]
#[derive(Debug)]
pub struct PubkeyCursor {
    offset: u16,
    new_value: Option<Pubkey>,
}

impl<'a> TryFrom<&'a Cursor> for &'a PubkeyCursor {
    type Error = SerError;

    fn try_from(cursor: &'a Cursor) -> Result<Self, SerError> {
        match cursor {
            Cursor::Pubkey(c) => Ok(c),
            _ => Err(SerError::InvalidCursorType),
        }
    }
}

impl<'a> TryFrom<&'a mut Cursor> for &'a mut PubkeyCursor {
    type Error = SerError;

    fn try_from(cursor: &'a mut Cursor) -> Result<Self, SerError> {
        match cursor {
            Cursor::Pubkey(c) => Ok(c),
            _ => Err(SerError::InvalidCursorType),
        }
    }
}

impl Writable for PubkeyCursor {
    fn size(&self) -> usize {
        32
    }

    fn write(self, src: &[u8], dest: &mut [u8], offset: usize) -> usize {
        if let Some(value) = self.new_value {
            let mut w = io::Cursor::new(&mut dest[offset..(offset + 32)]);
            borsh::to_writer(&mut w, &value).unwrap();
        } else {
            dest[offset..(offset + 32)]
                .copy_from_slice(&src[(self.offset as usize)..(self.offset as usize + 32)])
        }
        32
    }
}

impl PubkeyCursor {
    fn new(data: &[u8], offset: usize) -> (Self, usize) {
        let pk = Pubkey::try_from_slice(&data[offset..(offset + 32)]);
        (
            Self {
                offset: offset as u16,
                new_value: None,
            },
            32,
        )
    }

    pub fn get(&self, data: &[u8]) -> Pubkey {
        Pubkey::try_from_slice(&data[(self.offset as usize)..(self.offset as usize + 32)]).unwrap()
    }

    fn set(&mut self, value: Pubkey) {
        self.new_value = Some(value);
    }
}

#[cfg_attr(test, derive(BorshSerialize))]
#[derive(Debug)]
pub struct EmptyCursor {}

impl Writable for EmptyCursor {
    fn size(&self) -> usize {
        0
    }

    fn write(self, _src: &[u8], _dest: &mut [u8], _offset: usize) -> usize {
        0
    }
}

#[allow(unused)]
impl EmptyCursor {
    fn new(_offset: usize) -> (Self, usize) {
        (Self {}, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(BorshDeserialize, BorshSerialize)]
    struct Primitives {
        x: u8,
        y: u64,
    }

    #[test]
    fn primitive_test() -> anyhow::Result<()> {
        let s = Primitives {
            x: 1,
            y: 1000000000,
        };
        let mut v = borsh::to_vec(&s).unwrap();
        let (mut sc, _) = StructCursor::new(&[CursorType::U8, CursorType::U64], &v, 0);
        sc.get_mut(0)?
            .as_cursor::<&mut PrimitiveCursor<u8>>()?
            .set(0);
        sc.get_mut(1)?
            .as_cursor::<&mut PrimitiveCursor<u64>>()?
            .set(42);
        let new_size = sc.size();
        let mut v2 = vec![0u8; new_size];
        sc.write(&v, &mut v2, 0);
        let s2 = Primitives::try_from_slice(&v2).unwrap();
        assert_eq!(s2.x, 0);
        assert_eq!(s2.y, 42);
        Ok(())
    }

    #[derive(BorshDeserialize, BorshSerialize)]
    struct StateWithString {
        w: String,
        x: u8,
        y: String,
        z: u64,
    }

    #[test]
    fn string_test() -> anyhow::Result<()> {
        let s = StateWithString {
            w: "foo".into(),
            x: 1,
            y: "hello".into(),
            z: 43,
        };
        let mut v = borsh::to_vec(&s).unwrap();
        println!("v = {:?}", v);
        let d = &mut v;
        let (mut sc, _) = StructCursor::new(
            &[
                CursorType::String,
                CursorType::U8,
                CursorType::String,
                CursorType::U64,
            ],
            d,
            0,
        );
        sc.get_mut(1)?.as_cursor::<PrimitiveCursor<u8>>()?.set(0);
        sc.get_mut(2)?
            .as_cursor::<StringCursor>()?
            .set("Hello world");
        sc.get_mut(3)?.as_cursor::<PrimitiveCursor<u8>>()?.set(42);

        let new_size = sc.size();
        println!("new size: {}", new_size);
        let mut v2 = vec![0u8; new_size];
        let offset = sc.write(&v, &mut v2, 0);
        let s2 = StateWithString::try_from_slice(&v2).unwrap();
        assert_eq!(s2.w, "foo".to_string());
        assert_eq!(s2.x, 0);
        assert_eq!(s2.y, "Hello world".to_string());
        assert_eq!(s2.z, 42);
        Ok(())
    }

    // #[derive(Debug, BorshDeserialize, BorshSerialize, PartialEq, Eq)]
    // struct StateWithVec {
    //     v: Vec<u8>,
    // }

    // #[test]
    // fn vec_test() -> anyhow::Result<()> {
    //     let s = StateWithVec { v: vec![1, 2, 3] };
    //     let mut v = borsh::to_vec(&s)?;
    //     println!("v = {:?}", v);
    //     let d = &mut v;
    //     let (mut sc, _) = StructCursor::new(&[CursorType::Vec(Box::new(CursorType::U8))], d, 0);
    //     println!("sc: {:?}", sc);

    //     let Cursor::Vec(vc) = sc.get_cursor_mut(0) else {
    //         panic!("expect a vec cursor");
    //     };
    //     let Cursor::U8(c) = vc.get_cursor(2) else {
    //         panic!("expect an u8 curosr");
    //     };
    //     vc.push(&12u8);
    //     let new_size = sc.size();
    //     println!("new size: {}", new_size);
    //     let mut v2 = vec![0u8; new_size];
    //     let offset = sc.write(&v, &mut v2, 0);
    //     println!("v = {:?}", v2);
    //     let s2 = StateWithVec::try_from_slice(&v2).unwrap();
    //     assert_eq!(
    //         s2,
    //         StateWithVec {
    //             v: vec![1, 2, 3, 12]
    //         }
    //     );
    //     Ok(())
    // }

    // #[derive(BorshDeserialize, BorshSerialize)]
    // struct StateWithOption {
    //     v: Option<Primitives>,
    // }

    // #[test]
    // fn option_test() -> anyhow::Result<()> {
    //     let s = StateWithOption {
    //         v: Some(Primitives { x: 1, y: 2 }),
    //     };
    //     let mut v = borsh::to_vec(&s)?;
    //     let d = &mut v;
    //     let (mut sc, _) = StructCursor::new(
    //         &[CursorType::Option(Box::new(CursorType::Struct(vec![
    //             CursorType::U8,
    //             CursorType::U64,
    //         ])))],
    //         d,
    //         0,
    //     );
    //     println!("sc: {:?}", sc);
    //     let Cursor::Option(oc) = sc.get_cursor_mut(0) else {
    //         panic!("expect a vec cursor");
    //     };
    //     let Some(Cursor::Struct(sc1)) = oc.get_inner_mut() else {
    //         panic!("expect a struct cursor");
    //     };
    //     let Cursor::U8(c) = sc1.get_cursor_mut(0) else {
    //         panic!("expect an u8 cursor");
    //     };
    //     c.set(42);
    //     let new_size = sc.size();
    //     println!("new size: {}", new_size);
    //     let mut v2 = vec![0u8; new_size];
    //     let offset = sc.write(&v, &mut v2, 0);
    //     let s2 = StateWithOption::try_from_slice(&v2).unwrap();
    //     assert_eq!(s2.v.unwrap().x, 42);
    //     Ok(())
    // }

    // #[derive(BorshDeserialize, BorshSerialize, Debug, PartialEq, Eq)]
    // enum EnumState {
    //     Foo(u8),
    //     Bar(u64),
    // }

    // #[test]
    // fn enum_test() -> anyhow::Result<()> {
    //     let e = EnumState::Foo(1);
    //     let mut v = borsh::to_vec(&e)?;
    //     let d = &mut v;
    //     let (mut ec, _) = EnumCursor::new(&[CursorType::U8, CursorType::U64], d, 0);
    //     let e2: EnumState = ec.get(d)?;
    //     assert_eq!(e2, e);
    //     ec.set(&EnumState::Bar(42));
    //     let new_size = ec.size();
    //     println!("new size: {}", new_size);
    //     let mut v2 = vec![0u8; new_size];
    //     let offset = ec.write(&v, &mut v2, 0);
    //     let e3 = EnumState::try_from_slice(&v2).unwrap();
    //     assert_eq!(e3, EnumState::Bar(42));

    //     Ok(())
    // }
}
