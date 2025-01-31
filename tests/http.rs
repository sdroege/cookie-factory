extern crate cookie_factory;

#[path="../tests/http/mod.rs"] mod implementation;
use implementation::*;

#[cfg(test)]
mod tests {
  use super::*;
  use std::str::from_utf8;

  #[test]
  fn request() {
    let mut mem: [u8; 1024] = [0; 1024];
    let s = &mut mem[..];

    let request = Request {
      method: "GET",
      uri: "/hello/test/a/b/c?name=value#hash",
      headers: [
        Header { name: "Host", value: "lolcatho.st" },
        Header { name: "User-agent", value: "cookie-factory" },
        Header { name: "Content-Length", value: "13" },
        Header { name: "Connection", value: "Close" },
      ].iter().cloned().collect(),
      body: b"Hello, world!",
    };

    let (_, index) = cf_request((s, 0), &request).unwrap();
    println!("request written by cf:\n{}", from_utf8(&s[..index]).unwrap());

    let mut mem2: [u8; 1024] = [0; 1024];
    let ptr = {
      let s2 = &mut mem2[..];

      let mut sr = fn_request(&request);
      let (res, _) = sr(s2).unwrap();
      res.as_ptr() as usize
    };
    let index2 = ptr - (&mem2[..]).as_ptr() as usize;
    println!("request written by fn:\n{}", from_utf8(&mem2[..index2]).unwrap());
    println!("wrote {} bytes", index2);

    assert_eq!(index, index2);
    assert_eq!(from_utf8(&s[..index]).unwrap(), from_utf8(&mem2[..index2]).unwrap());
  }

  /*
  #[test]
  fn chunked_http() {
    let mut mem: [u8; 1024] = [0; 1024];
    let s = &mut mem[..];

    let request = RequestHeaders {
      method: "GET",
      uri: "/hello/test/a/b/c?name=value#hash",
      headers: [
        Header { name: "Host", value: "lolcatho.st" },
        Header { name: "User-agent", value: "cookie-factory" },
        Header { name: "Content-Length", value: "13" },
        Header { name: "Connection", value: "Close" },
      ].iter().cloned().collect(),
    };

    let mut sr = chunked_request(&request);
    assert_eq!(sr.serialize(&mut s[..132]), Ok((132, Serialized::Continue)));

    // add chunk
    sr.second.push(chunk(&b"Hello "[..]));
    assert_eq!(sr.serialize(&mut s[132..145]), Ok((13, Serialized::Continue)));
    assert_eq!(from_utf8(&s[132..145]).unwrap(), "\r\n\r\n6\r\nHello ");

    // add chunk
    sr.second.push(chunk(&b"world !"[..]));
    // add last chunk
    sr.second.push(chunk(&[]));

    assert_eq!(sr.serialize(&mut s[145..]), Ok((19, Serialized::Done)));
    assert_eq!(from_utf8(&s[136..164]).unwrap(), "6\r\nHello \r\n7\r\nworld !\r\n0\r\n\r\n");

  }*/
}

