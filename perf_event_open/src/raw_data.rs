use std::borrow::Cow;
use std::ops::Range;
use std::cmp::min;
use std::fmt;
use std::mem;

use byteorder::{ByteOrder, NativeEndian};
use crate::utils::HexValue;

impl< 'a > From< &'a Cow< 'a, [u8] > > for RawData< 'a > {
    fn from( data: &'a Cow< 'a, [u8] > ) -> Self {
        match *data {
            Cow::Owned( ref bytes ) => RawData::Single( bytes.as_slice() ),
            Cow::Borrowed( bytes ) => RawData::Single( bytes )
        }
    }
}

impl< 'a > From< &'a [u8] > for RawData< 'a > {
    fn from( bytes: &'a [u8] ) -> Self {
        RawData::Single( bytes )
    }
}

pub enum RawData< 'a > {
    Single( &'a [u8] ),
    Split( &'a [u8], &'a [u8] )
}

impl< 'a > fmt::Debug for RawData< 'a > {
    fn fmt( &self, fmt: &mut fmt::Formatter ) -> Result< (), fmt::Error > {
        match *self {
            RawData::Single( buffer ) => write!( fmt, "RawData::Single( [u8; {}] )", buffer.len() ),
            RawData::Split( left, right ) => write!( fmt, "RawData::Split( [u8; {}], [u8; {}] )", left.len(), right.len() )
        }
    }
}

impl< 'a > RawData< 'a > {
    #[inline]
    pub(crate) fn empty() -> Self {
        RawData::Single( &[] )
    }

    #[inline]
    fn write_into( &self, target: &mut Vec< u8 > ) {
        target.clear();
        match *self {
            RawData::Single( slice ) => target.extend_from_slice( slice ),
            RawData::Split( first, second ) => {
                target.reserve( first.len() + second.len() );
                target.extend_from_slice( first );
                target.extend_from_slice( second );
            }
        }
    }

    pub fn as_slice( &self ) -> Cow< 'a, [u8] > {
        match *self {
            RawData::Single( buffer ) => buffer.into(),
            RawData::Split( .. ) => {
                let mut vec = Vec::new();
                self.write_into( &mut vec );
                vec.into()
            }
        }
    }

    pub fn get( &self, range: Range< usize > ) -> RawData< 'a > {
        match self {
            &RawData::Single( buffer ) => RawData::Single( &buffer[ range ] ),
            &RawData::Split( first, second ) => {
                if range.start >= first.len() {
                    RawData::Single( &second[ range.start - first.len()..range.end - first.len() ] )
                } else if range.end <= first.len() {
                    RawData::Single( &first[ range ] )
                } else {
                    let first = &first[ range.start.. ];
                    let second = &second[ ..min( range.end - first.len(), second.len() ) ];
                    RawData::Split( first, second )
                }
            }
        }
    }

    pub fn len( &self ) -> usize {
        match *self {
            RawData::Single( buffer ) => buffer.len(),
            RawData::Split( first, second ) => first.len() + second.len()
        }
    }
}

pub struct RawRegs< 'a > {
    raw_data: RawData< 'a >
}

impl< 'a > RawRegs< 'a > {
    #[inline]
    pub(crate) fn from_raw_data( raw_data: RawData< 'a > ) -> Self {
        RawRegs { raw_data }
    }

    pub fn len( &self ) -> usize {
        self.raw_data.len() / mem::size_of::< u64 >()
    }

    pub fn get( &self, index: usize ) -> u64 {
        let offset = index * mem::size_of::< u64 >();
        match self.raw_data.get( offset..offset + mem::size_of::< u64 >() ) {
            RawData::Single( buffer ) => NativeEndian::read_u64( buffer ),
            RawData::Split( first, second ) => {
                let mut buffer = [0; 4];
                let mut index = 0;
                for &byte in first {
                    buffer[ index ] = byte;
                    index += 1;
                }
                for &byte in second {
                    buffer[ index ] = byte;
                    index += 1;
                }

                NativeEndian::read_u64( &buffer )
            }
        }
    }
}

impl< 'a > fmt::Debug for RawRegs< 'a > {
    fn fmt( &self, fmt: &mut fmt::Formatter ) -> Result< (), fmt::Error > {
        let mut list = fmt.debug_list();
        for index in 0..self.len() {
            let value = self.get( index );
            list.entry( &HexValue( value ) );
        }

        list.finish()
    }
}
