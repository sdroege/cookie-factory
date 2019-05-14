use gen::GenError;

pub trait SerializeFn<I>: Fn(I) -> Result<I, GenError> {}

impl<I, F:  Fn(I) ->Result<I, GenError>> SerializeFn<I> for F {}


pub fn slice<'a, S: 'a + AsRef<[u8]>>(data: S) -> impl SerializeFn<&'a mut [u8]> {
    let len = data.as_ref().len();

    move |out: &'a mut [u8]| {
        if out.len() < len {
            Err(GenError::BufferTooSmall(len))
        } else {
            (&mut out[..len]).copy_from_slice(data.as_ref());
            Ok(&mut out[len..])
        }
    }
}

pub fn string<'a, S: 'a+AsRef<str>>(data: S) -> impl SerializeFn<&'a mut [u8]> {

    let len = data.as_ref().len();
    move |out: &'a mut [u8]| {
        if out.len() < len {
            Err(GenError::BufferTooSmall(len))
        } else {
            (&mut out[..len]).copy_from_slice(data.as_ref().as_bytes());
            Ok(&mut out[len..])
        }
    }
}

use std::io::{Cursor, Write};
use std::fmt;
pub fn hex<'a, S: 'a + fmt::UpperHex>(data: S) -> impl SerializeFn<&'a mut [u8]> {

  move |out: &'a mut [u8]| {
    let mut c = Cursor::new(out);
    match write!(&mut c, "{:X}", data) {
      Err(_) => Err(GenError::CustomError(42)),
      Ok(_) => {
        let pos = c.position() as usize;
        let out = c.into_inner();
        Ok(&mut out[pos..])
      }
    }
  }
}

pub fn skip<'a>(len: usize) -> impl SerializeFn<&'a mut [u8]> {

    move |out: &'a mut [u8]| {
        if out.len() < len {
            Err(GenError::BufferTooSmall(len))
        } else {
            Ok(&mut out[len..])
        }
    }
}

pub fn position<'a, F>(f: F) -> impl Fn(&'a mut [u8]) -> Result<(&'a mut [u8], &'a mut [u8]), GenError>
  where F: SerializeFn<&'a mut [u8]> {

    move |out: &'a mut [u8]| {
        unsafe {
            let ptr = out.as_mut_ptr();
            let out = f(out)?;
            let len = out.as_ptr() as usize - ptr as usize;

            Ok((std::slice::from_raw_parts_mut(ptr, len), out))
        }
    }
}

fn pair<F, G, I>(first: F, second: G) -> impl SerializeFn<I>
where F: SerializeFn<I>,
      G: SerializeFn<I> {

  move |out: I| {
    let out = first(out)?;
    second(out)
  }
}

fn cond<F, G, I>(condition: bool, f: F) -> impl SerializeFn<I>
where F: SerializeFn<I>, {

  move |out: I| {
    if condition {
      f(out)
    } else {
      Ok(out)
    }
  }
}

pub fn all<'a, 'b, G, I, It>(values: It) -> impl SerializeFn<I> + 'a
  where G: SerializeFn<I> + 'b,
        It: 'a + Clone + Iterator<Item=G> {

  move |mut out: I| {
    let it = values.clone();

    for v in it {
      out = v(out)?;
    }

    Ok(out)
  }
}

pub fn separated_list<'a, 'b, 'c, F, G, I, It: Iterator<Item=G>, Arg: 'a+Clone+IntoIterator<Item=G, IntoIter=It>>(sep: F, values: Arg) -> impl SerializeFn<I> + 'a
  where F: SerializeFn<I> + 'b + 'a,
        G: SerializeFn<I> + 'c {

  move |mut out: I| {
    let mut it = values.clone().into_iter();
    match it.next() {
      None => return Ok(out),
      Some(first) => {
        out = first(out)?;
      }
    }

    for v in it {
      out = sep(out)?;
      out = v(out)?;
    }

    Ok(out)
  }
}

pub fn be_u8<'a>(i: u8) -> impl SerializeFn<&'a mut [u8]> {
   let len = 1;

    move |out: &'a mut [u8]| {
        if out.len() < len {
            Err(GenError::BufferTooSmall(len))
        } else {
            out[0] = i;
            Ok(&mut out[len..])
        }
    }
}

pub fn be_u16<'a>(i: u16) -> impl SerializeFn<&'a mut [u8]> {
   let len = 2;

    move |out: &'a mut [u8]| {
        if out.len() < len {
            Err(GenError::BufferTooSmall(len))
        } else {
            out[0] = ((i >> 8) & 0xff) as u8;
            out[1] = (i        & 0xff) as u8;
            Ok(&mut out[len..])
        }
    }
}

pub fn be_u32<'a>(i: u32) -> impl SerializeFn<&'a mut [u8]> {
   let len = 4;

    move |out: &'a mut [u8]| {
        if out.len() < len {
            Err(GenError::BufferTooSmall(len))
        } else {
            out[0] = ((i >> 24) & 0xff) as u8;
            out[1] = ((i >> 16) & 0xff) as u8;
            out[2] = ((i >> 8)  & 0xff) as u8;
            out[3] = (i         & 0xff) as u8;
            Ok(&mut out[len..])
        }
    }
}

pub fn be_u64<'a>(i: u64) -> impl SerializeFn<&'a mut [u8]> {
   let len = 8;

    move |out: &'a mut [u8]| {
        if out.len() < len {
            Err(GenError::BufferTooSmall(len))
        } else {
            out[0] = ((i >> 56) & 0xff) as u8;
            out[1] = ((i >> 48) & 0xff) as u8;
            out[2] = ((i >> 40) & 0xff) as u8;
            out[3] = ((i >> 32) & 0xff) as u8;
            out[4] = ((i >> 24) & 0xff) as u8;
            out[5] = ((i >> 16) & 0xff) as u8;
            out[6] = ((i >> 8)  & 0xff) as u8;
            out[7] = (i         & 0xff) as u8;
            Ok(&mut out[len..])
        }
    }
}

pub fn be_i8<'a>(i: i8) -> impl SerializeFn<&'a mut [u8]> {
    be_u8(i as u8)
}

pub fn be_i16<'a>(i: i16) -> impl SerializeFn<&'a mut [u8]> {
    be_u16(i as u16)
}

pub fn be_i32<'a>(i: i32) -> impl SerializeFn<&'a mut [u8]> {
    be_u32(i as u32)
}

pub fn be_i64<'a>(i: i64) -> impl SerializeFn<&'a mut [u8]> {
    be_u64(i as u64)
}

pub fn be_f32<'a>(i: f32) -> impl SerializeFn<&'a mut [u8]> {
    be_u32(unsafe { std::mem::transmute::<f32, u32>(i) })
}

pub fn be_f64<'a>(i: f64) -> impl SerializeFn<&'a mut [u8]> {
    be_u64(unsafe { std::mem::transmute::<f64, u64>(i) })
}

pub fn le_u8<'a>(i: u8) -> impl SerializeFn<&'a mut [u8]> {
   let len = 1;

    move |out: &'a mut [u8]| {
        if out.len() < len {
            Err(GenError::BufferTooSmall(len))
        } else {
            out[0] = i;
            Ok(&mut out[len..])
        }
    }
}

pub fn le_u16<'a>(i: u16) -> impl SerializeFn<&'a mut [u8]> {
   let len = 2;

    move |out: &'a mut [u8]| {
        if out.len() < len {
            Err(GenError::BufferTooSmall(len))
        } else {
            out[0] = (i        & 0xff) as u8;
            out[1] = ((i >> 8) & 0xff) as u8;
            Ok(&mut out[len..])
        }
    }
}

pub fn le_u32<'a>(i: u32) -> impl SerializeFn<&'a mut [u8]> {
   let len = 4;

    move |out: &'a mut [u8]| {
        if out.len() < len {
            Err(GenError::BufferTooSmall(len))
        } else {
            out[0] = (i         & 0xff) as u8;
            out[1] = ((i >> 8)  & 0xff) as u8;
            out[2] = ((i >> 16) & 0xff) as u8;
            out[3] = ((i >> 24) & 0xff) as u8;
            Ok(&mut out[len..])
        }
    }
}

pub fn le_u64<'a>(i: u64) -> impl SerializeFn<&'a mut [u8]> {
   let len = 8;

    move |out: &'a mut [u8]| {
        if out.len() < len {
            Err(GenError::BufferTooSmall(len))
        } else {
            out[0] = (i         & 0xff) as u8;
            out[1] = ((i >> 8)  & 0xff) as u8;
            out[2] = ((i >> 16) & 0xff) as u8;
            out[3] = ((i >> 24) & 0xff) as u8;
            out[4] = ((i >> 32) & 0xff) as u8;
            out[5] = ((i >> 40) & 0xff) as u8;
            out[6] = ((i >> 48) & 0xff) as u8;
            out[7] = ((i >> 56) & 0xff) as u8;
            Ok(&mut out[len..])
        }
    }
}

pub fn le_i8<'a>(i: i8) -> impl SerializeFn<&'a mut [u8]> {
    le_u8(i as u8)
}

pub fn le_i16<'a>(i: i16) -> impl SerializeFn<&'a mut [u8]> {
    le_u16(i as u16)
}

pub fn le_i32<'a>(i: i32) -> impl SerializeFn<&'a mut [u8]> {
    le_u32(i as u32)
}

pub fn le_i64<'a>(i: i64) -> impl SerializeFn<&'a mut [u8]> {
    le_u64(i as u64)
}

pub fn le_f32<'a>(i: f32) -> impl SerializeFn<&'a mut [u8]> {
    le_u32(unsafe { std::mem::transmute::<f32, u32>(i) })
}

pub fn le_f64<'a>(i: f64) -> impl SerializeFn<&'a mut [u8]> {
    le_u64(unsafe { std::mem::transmute::<f64, u64>(i) })
}

pub fn many_ref<'a, E, It, I, F, G, O>(items: I, generator: F) -> impl SerializeFn<O> + 'a
where
    It: Iterator<Item = E> + Clone + 'a,
    I: IntoIterator<Item = E, IntoIter = It>,
    F: Fn(E) -> G + 'a,
    G: SerializeFn<O> + 'a,
    O: 'a
{
    let items = items.into_iter();
    move |mut out: O| {
        for item in items.clone() {
            out = generator(item)(out)?;
        }
        Ok(out)
    }
}

pub fn length<'a, F>(f: F) -> impl Fn(&'a mut [u8]) -> Result<(usize, &'a mut [u8]), GenError>
  where F: SerializeFn<&'a mut [u8]> {
  move |out: &'a mut [u8]| {
    let start = out.as_ptr() as usize;

    let out = f(out)?;

    let end = out.as_ptr() as usize;
    Ok((end - start, out))
  }
}
//missing combinators:
//or
//empty
//then
//stream
//length_value
//text print
//text upperhex
//text lowerhex
