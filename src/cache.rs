use crate::error::{Error, Result};
use crate::tree::Tree;
use crate::types::Word;

const MAGIC: &[u8; 8] = b"WDLR4\0\0\0";
const VERSION: u32 = 5;
const HEADER_LEN: usize = 56;
const NODE_STRIDE: usize = 11;
const MAX_NODES: u32 = 65_535;
const MAX_EDGES: u32 = MAX_NODES * 64;

#[must_use]
pub fn hash_lists(probes: &[Word], answers: &[Word]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for w in probes.iter().chain(answers.iter()) {
        for b in w.to_le_bytes() {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x100_0000_01b3);
        }
    }
    h
}

fn header() -> Vec<u8> {
    let mut h = Vec::with_capacity(HEADER_LEN);
    h.extend_from_slice(MAGIC);
    h.resize(HEADER_LEN, 0);
    h
}

fn get_u32(buf: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([buf[at], buf[at + 1], buf[at + 2], buf[at + 3]])
}

fn get_u64(buf: &[u8], at: usize) -> u64 {
    let mut a = [0u8; 8];
    a.copy_from_slice(&buf[at..at + 8]);
    u64::from_le_bytes(a)
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Meta {
    pub list_hash: u64,
    pub num_probes: u32,
    pub num_answers: u32,
    pub worst: i32,
    pub total: i64,
}

impl Meta {
    #[must_use]
    pub fn new(probes: usize, answers: usize, worst: i32, total: i64) -> Self {
        Self {
            list_hash: 0,
            num_probes: probes as u32,
            num_answers: answers as u32,
            worst,
            total,
        }
    }

    #[must_use]
    pub fn matches(&self, hash: u64, probes: usize, answers: usize) -> bool {
        self.list_hash == hash
            && self.num_probes as usize == probes
            && self.num_answers as usize == answers
    }
}

pub fn save(path: &str, tree: &Tree, meta: &Meta) -> Result<()> {
    let mut h = header();
    h[8..12].copy_from_slice(&VERSION.to_le_bytes());
    h[12..16].copy_from_slice(&(tree.node_count() as u32).to_le_bytes());
    h[16..20].copy_from_slice(&(tree.edge_count() as u32).to_le_bytes());
    h[20..24].copy_from_slice(&meta.num_probes.to_le_bytes());
    h[24..28].copy_from_slice(&meta.num_answers.to_le_bytes());
    h[28..32].copy_from_slice(&u32::try_from(meta.worst).unwrap_or(0).to_le_bytes());
    h[36..44].copy_from_slice(&meta.list_hash.to_le_bytes());
    h[44..52].copy_from_slice(&u64::try_from(meta.total).unwrap_or(0).to_le_bytes());

    let nodes = tree.node_count();
    let mut buf = Vec::with_capacity(HEADER_LEN + nodes * NODE_STRIDE + tree.edge_count() * 3);
    buf.extend_from_slice(&h);

    for i in 0..nodes {
        buf.extend_from_slice(&tree.guess[i].to_le_bytes());
        buf.extend_from_slice(&tree.offset[i].to_le_bytes());
        buf.push(tree.count[i]);
        buf.extend_from_slice(&tree.weight[i].to_le_bytes());
    }
    buf.extend_from_slice(&tree.pattern);
    for &c in &tree.child {
        buf.extend_from_slice(&c.to_le_bytes());
    }

    std::fs::write(path, buf)?;
    Ok(())
}

pub fn load(path: &str, hash: u64, probes: usize, answers: usize) -> Result<Option<(Tree, Meta)>> {
    let buf = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(Error::Io(e)),
    };

    if buf.len() < HEADER_LEN || &buf[..8] != MAGIC || get_u32(&buf, 8) != VERSION {
        return Ok(None);
    }

    let nodes = get_u32(&buf, 12);
    let edges = get_u32(&buf, 16);
    let meta = Meta {
        list_hash: get_u64(&buf, 36),
        num_probes: get_u32(&buf, 20),
        num_answers: get_u32(&buf, 24),
        worst: i32::try_from(get_u32(&buf, 28)).unwrap_or(0),
        total: i64::try_from(get_u64(&buf, 44)).unwrap_or(0),
    };

    if nodes == 0 || nodes > MAX_NODES || edges > MAX_EDGES || !meta.matches(hash, probes, answers)
    {
        return Ok(None);
    }

    let (n, e) = (nodes as usize, edges as usize);
    let need = HEADER_LEN + n * NODE_STRIDE + e * 3;
    if buf.len() < need {
        return Ok(None);
    }

    let mut at = HEADER_LEN;
    let mut guess = Vec::with_capacity(n);
    let mut offset = Vec::with_capacity(n);
    let mut count = Vec::with_capacity(n);
    let mut weight = Vec::with_capacity(n);
    for _ in 0..n {
        guess.push(u16::from_le_bytes([buf[at], buf[at + 1]]));
        offset.push(get_u32(&buf, at + 2));
        count.push(buf[at + 6]);
        weight.push(get_u32(&buf, at + 7));
        at += NODE_STRIDE;
    }
    let pattern = buf[at..at + e].to_vec();
    at += e;
    let mut child = Vec::with_capacity(e);
    for _ in 0..e {
        child.push(u16::from_le_bytes([buf[at], buf[at + 1]]));
        at += 2;
    }

    let tree = Tree {
        guess,
        offset,
        count,
        pattern,
        child,
        weight,
    };
    if !tree.validate() {
        return Ok(None);
    }
    Ok(Some((tree, meta)))
}
