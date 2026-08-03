use std::{ops::Range, str::FromStr};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum PortDecl {
    Single(u16),
    Range(Range<u16>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortSpec {
    ports: Vec<PortDecl>,
    total: usize,
}

impl std::fmt::Display for PortSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, p) in self.ports.iter().enumerate() {
            if i > 0 {
                f.write_str(",")?;
            }
            match p {
                PortDecl::Single(p) => f.write_str(p.to_string().as_ref())?,
                PortDecl::Range(Range { start, end }) => {
                    f.write_str(start.to_string().as_ref())?;
                    f.write_str("-")?;
                    f.write_str(end.to_string().as_ref())?;
                }
            }
        }
        Ok(())
    }
}

impl PortSpec {
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self {
            ports: Vec::new(),
            total: 0,
        }
    }
    pub fn new_with(port: u16) -> Self {
        Self {
            ports: vec![PortDecl::Single(port)],
            total: 1,
        }
    }

    pub fn add(&mut self, port: u16) -> bool {
        for decl in &self.ports {
            match decl {
                &PortDecl::Single(p) if p == port => return false,
                PortDecl::Range(r) if r.contains(&port) => return false,
                _ => {}
            }
        }

        self.ports.push(PortDecl::Single(port));
        self.total += 1;
        true
    }
    pub fn add_range(&mut self, range: Range<u16>) {
        let mut merged = false;
        self.ports.retain_mut(|decl| match decl {
            // If single-port entry persist in range, remove
            &mut PortDecl::Single(p) if range.contains(&p) => false,
            // If range is fully or partially contained in existing range, merge them into one
            PortDecl::Range(r) if range.contains(&r.start) || range.contains(&r.end) => {
                let old_len = r.len();
                merged = true;
                r.start = range.start.min(r.start);
                r.end = range.end.max(r.end);
                let new_len = r.len();
                self.total += new_len - old_len;
                true
            }
            _ => true,
        });
        if !merged {
            self.total += range.len();
            self.ports.push(PortDecl::Range(range));
        }
    }

    pub const fn contains(&self, port: u16) -> bool {
        let arr = self.ports.as_slice();
        let mut idx = 0;
        while idx < arr.len() {
            match &arr[idx] {
                PortDecl::Single(p) if *p == port => return true,
                PortDecl::Range(r) if r.start <= port && port <= r.end => return true,
                _ => {}
            }
            idx += 1;
        }
        false
    }

    pub const fn length(&self) -> usize {
        let mut length = 0_usize;
        let mut idx = 0;
        let arr = self.ports.as_slice();
        while idx < arr.len() {
            match &arr[idx] {
                PortDecl::Single(_) => length += 1,
                PortDecl::Range(r) => {
                    #[allow(clippy::cast_lossless)]
                    let range_len = (r.end as u32 - r.start as u32 + 1) as usize;
                    length += range_len;
                }
            }
            idx += 1;
        }
        length
    }

    /// Get the first port
    pub const fn first(&self) -> Option<u16> {
        match *self.ports.as_slice() {
            [PortDecl::Single(p), ..] => Some(p),
            [PortDecl::Range(Range { start, .. }), ..] => Some(start),
            _ => None,
        }
    }

    /// Get the last port
    pub const fn last(&self) -> Option<u16> {
        match *self.ports.as_slice() {
            [.., PortDecl::Single(p)] => Some(p),
            [.., PortDecl::Range(Range { end, .. })] => Some(end),
            _ => None,
        }
    }

    pub fn sort(mut self) -> Self {
        self.ports.sort_by_key(|p| match p {
            PortDecl::Single(p) => *p,
            PortDecl::Range(r) => r.start,
        });
        self
    }

    pub fn iter(&self) -> impl Iterator<Item = u16> {
        PortSpecIter {
            spec: self,
            outer_idx: 0,
            inner_idx: 0,
        }
    }

    pub fn iter_raw(&self) -> impl Iterator<Item = &PortDecl> {
        self.ports.iter()
    }
}

impl FromStr for PortSpec {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut spec = Self::new();

        for decl in s.split(',') {
            if let Some((s, e)) = decl.split_once('-') {
                spec.add_range(s.parse()?..e.parse()?);
            } else {
                spec.add(decl.parse()?);
            }
        }
        Ok(spec.sort())
    }
}

struct PortSpecIter<'a> {
    spec: &'a PortSpec,
    outer_idx: usize,
    inner_idx: u32,
}

impl Iterator for PortSpecIter<'_> {
    type Item = u16;

    fn next(&mut self) -> Option<u16> {
        match self.spec.ports.get(self.outer_idx)? {
            PortDecl::Single(port) => {
                self.outer_idx += 1;
                Some(*port)
            }
            PortDecl::Range(r) => {
                let port = u32::from(r.start) + self.inner_idx;
                if port <= u32::from(r.end) {
                    self.inner_idx += 1;
                    #[allow(clippy::cast_possible_truncation)]
                    let port = port as u16;
                    Some(port)
                } else {
                    self.outer_idx += 1;
                    self.inner_idx = 0;
                    self.next()
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_port_spec_contains_no_panic() {
        let spec = PortSpec::new();
        assert!(!spec.contains(80));
        assert_eq!(spec.length(), 0);
        assert!(spec.first().is_none());
        assert!(spec.last().is_none());
    }

    #[test]
    fn port_spec_range_inclusive_semantics() {
        let mut spec = PortSpec::new();
        spec.add_range(100..200);
        assert!(spec.contains(100)); // start
        assert!(spec.contains(200)); // end (was failing with exclusive)
        assert!(!spec.contains(99)); // below
        assert!(!spec.contains(201)); // above
        assert_eq!(spec.length(), 101);
        let collected: Vec<u16> = spec.iter().collect();
        assert_eq!(collected.len(), 101);
        assert_eq!(collected[0], 100);
        assert_eq!(collected[100], 200);
    }

    #[test]
    fn full_range_ports_do_not_overflow() {
        let mut spec = PortSpec::new();
        spec.add_range(1..65535);
        assert_eq!(spec.length(), 65535);
        let all: Vec<u16> = spec.iter().collect();
        assert_eq!(all.len(), 65535);
        assert_eq!(all[0], 1);
        assert_eq!(*all.last().unwrap(), 65535);
    }

    #[test]
    fn full_u16_range_does_not_wrap() {
        let mut spec = PortSpec::new();
        spec.add_range(0..65535);
        assert_eq!(spec.length(), 65536);
        let all: Vec<u16> = spec.iter().collect();
        assert_eq!(all.len(), 65536);
        assert_eq!(all[0], 0);
        assert_eq!(*all.last().unwrap(), 65535);
    }
}
