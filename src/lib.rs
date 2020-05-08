use std::collections::{hash_map, HashMap};

use bstr::{BStr, BString, ByteSlice, B};

// #[derive(Debug, Clone, Copy, Default)]
// struct HasherU8(u8);

// impl std::hash::Hasher for HasherU8 {
//     fn finish(&self) -> u64 {
//         self.0 as u64
//     }

//     fn write(&mut self, _bytes: &[u8]) {
//         unimplemented!("HasherU8 only supports `u8` keys")
//     }

//     fn write_u8(&mut self, i: u8) {
//         self.0 = i;
//     }
// }



// #[derive(Clone, Debug)]
// enum Nodes<T> {
//     Empty,
//     Static(HashMap<u8, Node<T>>),
//     Dynamic(Box<Node<T>>)
// }


// impl<T> Nodes<T> {
//     fn add_static(&mut self)

// }

// impl<T> Default for Nodes<T> {
//     fn default() -> Self {
//         Nodes::Empty
//     }
// }

// #[derive(Clone, Debug)]
// enum NodeKind {
//     // Root,
//     Static { path: BString },
//     Parameter,
//     // Parameter { terminator: Option<BString> },
// }

#[derive(Clone, Debug)]
struct Nodes<T> {
    static_: Option<HashMap<u8, StaticNode<T>>>,
    parameter: Option<Box<ParameterNode<T>>>,
}

impl<T> Default for Nodes<T> {
    fn default() -> Self {
        Self {
            static_: None,
            parameter: None,
        }
    }
}

impl<T> Nodes<T> {

    fn add_static_node(&mut self, path_first: u8, path: &[u8]) -> &mut StaticNode<T> {
        let nodes: &mut HashMap<_, _> = self.static_.get_or_insert_with(HashMap::new);
        match nodes.entry(path_first) {
            hash_map::Entry::Occupied(e) => e.into_mut().insert(path),
            hash_map::Entry::Vacant(e) => e.insert(StaticNode::from_path(path)),
        }
    }

    fn add_parameter_node(&mut self) -> &mut ParameterNode<T> {
        self.parameter = Some(Box::new(ParameterNode::default()));
        self.parameter.as_mut().unwrap()
    }

    fn get(&self, key: &[u8]) -> Option<NodeRef<T>> {
        if key.is_empty() { return None; }

        self.static_.as_ref()
            .and_then(|m| m.get(&key[0]) )
            .map(NodeRef::Static)
            .or_else(|| self.parameter.as_ref().map(|n| NodeRef::Parameter(n)) )
    }
}

#[derive(Clone, Debug)]
struct StaticNode<T> {
    data: Option<T>,
    nodes: Nodes<T>,
    path: BString,
    params: Option<Vec<BString>>,
}

impl<T> StaticNode<T> {
    
    fn from_path<U: AsRef<[u8]>>(path: U) -> Self {
        let path = path.as_ref().replace("{{", "{").replace("}}", "}");
        Self { path: path.into(), nodes: Nodes::default(), data: None, params: None }
    }
    
    fn insert(&mut self, path: &[u8]) -> &mut Self {
        let shared = utils::loc(&self.path, path);
        let shared_len = shared.len();

        if shared_len < self.path.len() {
            self.path = self.path[shared_len..].to_owned().into();
            let prev_path_0 = self.path[0];
            let mut node = StaticNode::from_path(shared);
            std::mem::swap(self, &mut node);                    
            self.nodes.static_ = {
                let mut map = HashMap::new();
                map.insert(prev_path_0, node);
                Some(map)
            };
        }
        if shared_len == path.len() {
           self
        } else {
            let path = &path[shared_len..];
            self.nodes.add_static_node(path[0], path)
        }        
    }
}

#[derive(Clone, Debug)]
struct ParameterNode<T> {
    data: Option<T>,
    nodes: Nodes<T>,
    params: Option<Vec<BString>>,
    delimiter: Option<BString>,
}

impl<T> Default for ParameterNode<T> {
    fn default() -> Self {
        Self { data: None, nodes: Nodes::default(), params: None, delimiter: None }
    }
}

impl<T> ParameterNode<T> {
    fn set_delimiter<U: AsRef<[u8]>>(&mut self, delimiter: U) {
        self.delimiter = Some(delimiter.as_ref().replace("{{", "{").replace("}}", "}").into());
    }
}

#[derive(Clone, Debug)]
enum Node<T> {
    Static(StaticNode<T>),
    Parameter(ParameterNode<T>)
}

trait NodeMut<T> {
    fn add_static_node<'b>(&mut self, path_first: u8, path: &'b [u8]) -> &mut StaticNode<T>;
    fn add_parameter_node(&mut self) -> &mut ParameterNode<T>;
    fn set_data(&mut self, data: Option<T>);
    fn set_params(&mut self, params: Option<Vec<BString>>);
    fn as_mut_parameter_node(&mut self) -> Option<&mut ParameterNode<T>> { None }
}

impl<T> NodeMut<T> for StaticNode<T> {
    fn add_static_node<'b>(&mut self, path_first: u8, path: &'b [u8]) -> &mut StaticNode<T> {
        self.nodes.add_static_node(path_first, path)
    }
    fn add_parameter_node(&mut self) -> &mut ParameterNode<T> {
        self.nodes.add_parameter_node()
    }
    fn set_data(&mut self, data: Option<T>) { self.data = data; }
    fn set_params(&mut self, params: Option<Vec<BString>>) { self.params = params; }
}

impl<T> NodeMut<T> for ParameterNode<T> {
    fn add_static_node<'b>(&mut self, path_first: u8, path: &'b [u8]) -> &mut StaticNode<T> {
        self.nodes.add_static_node(path_first, path)
    }
    fn add_parameter_node(&mut self) -> &mut ParameterNode<T> {
        self.nodes.add_parameter_node()
    }
    fn set_data(&mut self, data: Option<T>) { self.data = data; }
    fn set_params(&mut self, params: Option<Vec<BString>>) { self.params = params; }
    fn as_mut_parameter_node(&mut self) -> Option<&mut ParameterNode<T>> { Some(self) }
}

#[derive(Clone, Debug)]
enum NodeRef<'a, T> {
    Static(&'a StaticNode<T>),
    Parameter(&'a ParameterNode<T>)
}

impl<'a, T> NodeRef<'a, T> {

    fn data(&self) -> Option<&'a T> {
        match *self {
            Self::Static(ref node) => node.data.as_ref(),
            Self::Parameter(ref node) => node.data.as_ref(),
        }
    }

    fn params(&self) -> Option<&'a Vec<BString>> {
        match *self {
            Self::Static(ref node) => node.params.as_ref(),
            Self::Parameter(ref node) => node.params.as_ref(),
        }
    }

    fn find<'b>(self, mut path: &'b [u8]) -> Option<(Self, Vec<&'b [u8]>)> {

        let mut params = Vec::new();

        match self {
            Self::Static(ref node) => {
                let shared = utils::loc(&node.path, path);

                let shared_len = shared.len();
                
                if shared_len == 0 {
                    None
                } else if shared_len < node.path.len() {
                    None
                } else if shared_len == node.path.len() && shared_len == path.len() {
                    Some((
                        NodeRef::Static(node),
                        params,
                    ))
                } else {
                    path = &path[shared_len..];
                    if let Some((n, mut ps)) = node.nodes.get(&path)
                        .and_then(|n| n.find(path) ) {
                        params.append(&mut ps);
                        Some((n, params))
                    } else {
                        None
                    }
                }
            }
            Self::Parameter(ref node) => {
                if let Some(del) = node.delimiter.as_ref() {
                    if let Some(idx) = path.find(del) {
                        params.push(&path[..idx]);
                        path = &path[idx..];
                        if let Some((n, mut ps)) = node.nodes.get(&path)
                            .and_then(|n| n.find(path) ) {
                            params.append(&mut ps);
                            Some((n, params))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    params.push(path);
                    Some((
                        NodeRef::Parameter(node),
                        params,
                    ))
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum BuildError {
    AlreadyConsumed,
    UnmatchedBrace(BString),
    InvalidParameterName(BString),
    MissingParameterDelimiter(BString),
    Many(Vec<BuildError>),
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // use std::io::write;
        match self {
            Self::AlreadyConsumed => "Builder has already been consumed".fmt(f),
            Self::UnmatchedBrace(ref path) => write!(f, ""),
            _ => panic!()
        }
    }
}

#[derive(Clone, Debug)]
pub struct Router<T>(StaticNode<T>);

pub struct RouterBuilder<T> {
    inner: Option<Router<T>>,
    errors: Vec<BuildError>,
}

impl<T> RouterBuilder<T> {
    pub fn insert<U: AsRef<[u8]>>(&mut self, path: U, data: T) -> &mut Self {

        let mut path = path.as_ref();
        let mut full = path;

        let mut node = match self.inner.as_mut() {
            Some(router) => &mut router.0,
            _ => {
                self.errors.push(BuildError::AlreadyConsumed);
                return self;
            }
        };

        let mut params: Option<Vec<BString>> = None;

        let mut node = node as &mut dyn NodeMut<T>;

        loop {
            if path.is_empty() { break; }

            match utils::find_solo_byte(path, b'{') {
                Some(bs) => {
                    let mut prefix = &path[..bs];
                    let mut suffix = &path[bs + 1..];

                    if let Some(n) = node.as_mut_parameter_node() {
                        if prefix.is_empty() {
                            self.errors.push(BuildError::MissingParameterDelimiter(full.into()));
                            return self;
                        } else {
                            n.set_delimiter(prefix);
                        }                        
                    }

                    if !prefix.is_empty() {
                        node = node.add_static_node(prefix[0], prefix);
                    }

                    match suffix.find_byte(b'}') {
                        Some(be) => {
                            path = &suffix[be + 1..];
                            suffix = &suffix[..be];

                            if suffix.is_empty() {
                                self.errors.push(BuildError::InvalidParameterName(full.into()));
                                return self;
                            }

                            params.get_or_insert_with(Vec::new).push(suffix.into());
                            node = node.add_parameter_node();
                        }
                        None => {
                            self.errors.push(BuildError::UnmatchedBrace(full.into()));
                            return self;
                        }
                    }
                },
                None => {
                    if utils::find_solo_byte(path, b'}').is_some() {
                        self.errors.push(BuildError::UnmatchedBrace(full.into()));
                        return self;
                    }

                    if let Some(n) = node.as_mut_parameter_node() {
                        n.set_delimiter(path);
                    }

                    node = node.add_static_node(path[0], path);
                    break;   
                }
            }
        }

        node.set_data(Some(data));
        node.set_params(params);
        self
    }

    pub fn finish(&mut self) -> std::result::Result<Router<T>, BuildError> {
        
        if !self.errors.is_empty() {
            return Err(BuildError::Many(self.errors.drain(..).collect()));
        }

        match self.inner.take() {
            Some(router) => Ok(router),
            _ => Err(BuildError::AlreadyConsumed)
        }
    }
}

impl<T> Default for Router<T> {
    fn default() -> Self {
        Self(StaticNode::from_path(vec![]))
    }
}

impl<T> Router<T> {

    pub fn builder() -> RouterBuilder<T> {
        RouterBuilder {
            inner: Some(Router(StaticNode::from_path(vec![]))),
            errors: Vec::new()
        }
    }

    pub fn find<'a, 'b>(&'a self, path: &'b [u8]) -> Option<(&'a T, Vec<(&'a BStr, &'b BStr)>)> {

        if path.is_empty() {
            return self.0.data.as_ref().map(|d| (d, vec![]));
        }

        self.0.nodes.get(path).and_then(|n| n.find(path)).and_then(|(node, params)| {
            node.data().map(|data| {
                (
                    data,
                    node.params().map(|ps| {
                        ps
                            .iter()
                            .zip(params.iter())
                            .map(|(a, b)| (a.as_bstr(), b.as_bstr()))
                            .collect()
                    }).unwrap_or_else(Vec::new)
                )
            })
        })
    }

}

mod utils {

    use bstr::ByteSlice;

    #[inline]
    pub fn loc(s: &[u8], p: &[u8]) -> Vec<u8> {
        s.iter()
            .zip(p)
            .take_while(|(a, b)| a == b)
            .map(|(v, _)| *v)
            .collect()
    }

    pub fn find_solo_byte(path: &[u8], b: u8) -> Option<usize> {        
        fn inner(path: &[u8], b: u8, prev_idx: usize) -> Option<usize> {
            match path.find_byte(b) {
                Some(bs) if path.get(bs + 1) == Some(&b) => inner(&path[bs + 2..], b, bs + 2),
                Some(bs) => Some(bs + prev_idx),
                None => None
            }
        }
        inner(path, b, 0)
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        println!("{:?}", crate::utils::find_solo_byte(b"{{{{", b'{'));
        println!("{:?}", crate::utils::find_solo_byte(b"Hello, {", b'{'));
    }
}
