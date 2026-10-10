const HANDSHAKE: u8 = 0x16;
const CLIENT_HELLO: u8 = 0x01;
const SERVER_NAME: usize = 0x0000;
const HEADER: usize = 5;

fn be16(data: &[u8], at: usize) -> Option<usize> {
    Some(((*data.get(at)? as usize) << 8) | *data.get(at + 1)? as usize)
}

pub fn is_handshake(data: &[u8]) -> bool {
    data.first() == Some(&HANDSHAKE)
}

pub fn record_len(data: &[u8]) -> Option<usize> {
    if !is_handshake(data) {
        return None;
    }
    be16(data, 3).map(|len| HEADER + len)
}

pub fn sni_offset(data: &[u8]) -> Option<usize> {
    if !is_handshake(data) || *data.get(HEADER)? != CLIENT_HELLO {
        return None;
    }
    let mut at = HEADER + 4 + 2 + 32;
    at += 1 + *data.get(at)? as usize;
    at += 2 + be16(data, at)?;
    at += 1 + *data.get(at)? as usize;
    let end = (at + 2 + be16(data, at)?).min(data.len());
    at += 2;
    while at + 4 <= end {
        let kind = be16(data, at)?;
        let len = be16(data, at + 2)?;
        at += 4;
        if kind == SERVER_NAME {
            let name_len = be16(data, at + 3)?;
            let start = at + 5;
            return (start + name_len <= data.len()).then_some(start);
        }
        at += len;
    }
    None
}

pub fn split_record(data: &[u8], at: usize) -> Option<Vec<u8>> {
    let end = record_len(data)?;
    if end > data.len() || at <= HEADER || at >= end {
        return None;
    }
    let mut out = Vec::with_capacity(data.len() + HEADER);
    for part in [&data[HEADER..at], &data[at..end]] {
        out.extend_from_slice(&[HANDSHAKE, data[1], data[2]]);
        out.extend_from_slice(&(part.len() as u16).to_be_bytes());
        out.extend_from_slice(part);
    }
    out.extend_from_slice(&data[end..]);
    Some(out)
}

#[cfg(test)]
pub fn client_hello(host: &str) -> Vec<u8> {
    let name = host.as_bytes();
    let mut sni = Vec::new();
    sni.extend_from_slice(&((name.len() + 3) as u16).to_be_bytes());
    sni.push(0);
    sni.extend_from_slice(&(name.len() as u16).to_be_bytes());
    sni.extend_from_slice(name);

    let mut extensions = Vec::new();
    extensions.extend_from_slice(&[0x00, 0x0b, 0x00, 0x02, 0x01, 0x00]);
    extensions.extend_from_slice(&[0x00, 0x00]);
    extensions.extend_from_slice(&(sni.len() as u16).to_be_bytes());
    extensions.extend_from_slice(&sni);

    let mut body = vec![0x03, 0x03];
    body.extend_from_slice(&[7u8; 32]);
    body.push(0);
    body.extend_from_slice(&[0x00, 0x02, 0x13, 0x01]);
    body.extend_from_slice(&[0x01, 0x00]);
    body.extend_from_slice(&(extensions.len() as u16).to_be_bytes());
    body.extend_from_slice(&extensions);

    let mut handshake = vec![CLIENT_HELLO];
    handshake.extend_from_slice(&(body.len() as u32).to_be_bytes()[1..]);
    handshake.extend_from_slice(&body);

    let mut record = vec![HANDSHAKE, 0x03, 0x01];
    record.extend_from_slice(&(handshake.len() as u16).to_be_bytes());
    record.extend_from_slice(&handshake);
    record
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_server_name() {
        let hello = client_hello("discord.com");
        let at = sni_offset(&hello).unwrap();
        assert_eq!(&hello[at..at + 11], b"discord.com");
    }

    #[test]
    fn rejects_non_tls() {
        assert_eq!(sni_offset(b"GET / HTTP/1.1\r\n\r\n"), None);
        assert_eq!(sni_offset(&[0x16, 0x03]), None);
    }

    #[test]
    fn splits_record_in_two() {
        let hello = client_hello("discord.com");
        let at = sni_offset(&hello).unwrap() + 1;
        let out = split_record(&hello, at).unwrap();
        assert_eq!(out.len(), hello.len() + HEADER);

        let first = record_len(&out).unwrap();
        assert_eq!(first, at);
        let second = &out[first..];
        assert_eq!(record_len(second).unwrap(), second.len());

        let mut joined = out[HEADER..first].to_vec();
        joined.extend_from_slice(&second[HEADER..]);
        assert_eq!(joined, hello[HEADER..]);
    }

    #[test]
    fn refuses_bad_split_points() {
        let hello = client_hello("discord.com");
        assert!(split_record(&hello, 3).is_none());
        assert!(split_record(&hello, hello.len()).is_none());
    }
}
