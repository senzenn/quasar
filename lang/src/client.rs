//! Off-chain instruction building utilities.
//!
//! This module re-exports [`wincode`] for bincode-compatible serialization and
//! provides three wrapper types that encode Quasar's dynamic wire format:
//!
//! | Type | Wire format |
//! |------|-------------|
//! | [`DynBytes`] | `u32 LE` length prefix + raw bytes |
//! | [`DynVec<T>`] | `u32 LE` length prefix + each item serialized |
//! | [`TailBytes`] | raw bytes (no length prefix) |
//!
//! **This is the only module in `quasar-lang` that allocates** — it uses
//! `alloc::vec::Vec` for instruction data buffers since off-chain code runs
//! in a standard allocator environment.

extern crate alloc;

use alloc::vec::Vec;
use core::mem::MaybeUninit;
use wincode::{
    config::ConfigCore,
    error::{ReadResult, WriteResult},
    io::{Reader, Writer},
    len::{SeqLen, UseIntLen},
    SchemaRead, SchemaWrite,
};

// Re-export wincode for downstream derive macro codegen.
pub use wincode;

// Re-export instruction types used by generated client code.
pub use solana_instruction::{AccountMeta, Instruction};

/// Length encoding: little-endian `u32` prefix (Quasar wire format).
type U32Len = UseIntLen<u32>;

// ---------------------------------------------------------------------------
// DynBytes — u32-prefixed raw byte buffer
// ---------------------------------------------------------------------------

/// A dynamically-sized byte buffer prefixed with a `u32 LE` length.
///
/// Used in generated client code to serialize variable-length byte fields
/// (e.g. `String`, `Vec<u8>`) in instruction data.
pub struct DynBytes(pub Vec<u8>);

unsafe impl<C: ConfigCore> SchemaWrite<C> for DynBytes
where
    U32Len: wincode::len::SeqLen<C>,
{
    type Src = Vec<u8>;

    fn size_of(src: &Self::Src) -> WriteResult<usize> {
        Ok(U32Len::write_bytes_needed(src.len())? + src.len())
    }

    fn write(mut writer: impl Writer, src: &Self::Src) -> WriteResult<()> {
        U32Len::write(writer.by_ref(), src.len())?;
        writer.write(src)?;
        Ok(())
    }
}

unsafe impl<'de, C: ConfigCore> SchemaRead<'de, C> for DynBytes
where
    U32Len: wincode::len::SeqLen<C>,
{
    type Dst = Vec<u8>;

    fn read(mut reader: impl Reader<'de>, dst: &mut MaybeUninit<Self::Dst>) -> ReadResult<()> {
        let len = U32Len::read(reader.by_ref())?;
        let bytes = reader.take_scoped(len)?;
        dst.write(bytes.to_vec());
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// DynVec<T> — u32-prefixed sequence of T
// ---------------------------------------------------------------------------

/// A dynamically-sized vector of `T` prefixed with a `u32 LE` element count.
///
/// Used in generated client code to serialize `Vec<T>` instruction arguments.
pub struct DynVec<T>(core::marker::PhantomData<T>);

unsafe impl<T, C: ConfigCore> SchemaWrite<C> for DynVec<T>
where
    T: SchemaWrite<C>,
    T::Src: Sized,
    U32Len: wincode::len::SeqLen<C>,
{
    type Src = Vec<T::Src>;

    fn size_of(src: &Self::Src) -> WriteResult<usize> {
        let mut total = U32Len::write_bytes_needed(src.len())?;
        for item in src {
            total += T::size_of(item)?;
        }
        Ok(total)
    }

    fn write(mut writer: impl Writer, src: &Self::Src) -> WriteResult<()> {
        U32Len::write(writer.by_ref(), src.len())?;
        for item in src {
            T::write(writer.by_ref(), item)?;
        }
        Ok(())
    }
}

unsafe impl<'de, T, C: ConfigCore> SchemaRead<'de, C> for DynVec<T>
where
    T: SchemaRead<'de, C>,
    U32Len: wincode::len::SeqLen<C>,
{
    type Dst = Vec<T::Dst>;

    fn read(mut reader: impl Reader<'de>, dst: &mut MaybeUninit<Self::Dst>) -> ReadResult<()> {
        let len = U32Len::read(reader.by_ref())?;
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(T::get(reader.by_ref())?);
        }
        dst.write(vec);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// TailBytes — unprefixed trailing bytes
// ---------------------------------------------------------------------------

/// Raw trailing bytes with no length prefix.
///
/// On write, emits the raw bytes. On read, consumes all remaining bytes
/// from the reader. Useful for variable-length trailing data in instruction
/// payloads.
pub struct TailBytes(pub Vec<u8>);

unsafe impl<C: ConfigCore> SchemaWrite<C> for TailBytes {
    type Src = Vec<u8>;

    fn size_of(src: &Self::Src) -> WriteResult<usize> {
        Ok(src.len())
    }

    fn write(mut writer: impl Writer, src: &Self::Src) -> WriteResult<()> {
        writer.write(src)?;
        Ok(())
    }
}

unsafe impl<'de, C: ConfigCore> SchemaRead<'de, C> for TailBytes {
    type Dst = Vec<u8>;

    fn read(mut reader: impl Reader<'de>, dst: &mut MaybeUninit<Self::Dst>) -> ReadResult<()> {
        // Consume all remaining bytes one at a time. This is only used
        // off-chain for instruction data deserialization, so the byte-at-a-time
        // approach is acceptable.
        let mut bytes = Vec::new();
        while let Ok(b) = reader.take_byte() {
            bytes.push(b);
        }
        dst.write(bytes);
        Ok(())
    }
}
