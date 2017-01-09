use std;
use std::io::{Read, Write, Result};
use super::encodable_types::*;
use super::helpers::*;
use super::node_ids::*;

#[derive(PartialEq, Clone, Debug)]
pub enum Identifier {
    Numeric(UInt64),
    String(UAString),
    Guid(Guid),
    ByteString(UAString),
}

/// An identifier for a node in the address space of an OPC UA Server.
/// Data type ID 17
#[derive(PartialEq, Clone, Debug)]
pub struct NodeId {
    /// The index for a namespace
    pub namespace: UInt16,
    /// The identifier for the node in the address space
    pub identifier: Identifier,
}

impl BinaryEncoder<NodeId> for NodeId {
    fn byte_len(&self) -> usize {
        // Type determines the byte code
        let size: usize = match self.identifier {
            Identifier::Numeric(ref value) => {
                if self.namespace == 0 && *value <= 255 {
                    2
                } else if self.namespace <= 255 && *value <= 65535 {
                    4
                } else {
                    11
                }
            },
            Identifier::String(ref value) => {
                3 + value.byte_len()
            },
            Identifier::Guid(ref value) => {
                3 + value.byte_len()
            },
            Identifier::ByteString(ref value) => {
                3 + value.byte_len()
            }
        };
        size
    }

    fn encode(&self, stream: &mut Write) -> Result<usize> {
        let mut size: usize = 0;
        // Type determines the byte code
        match self.identifier {
            Identifier::Numeric(ref value) => {
                if self.namespace == 0 && *value <= 255 {
                    // node id fits into 2 bytes when the namespace is 0 and the value <= 255
                    size += write_u8(stream, 0x0)?;
                    size += write_u8(stream, *value as u8)?;
                } else if self.namespace <= 255 && *value <= 65535 {
                    // node id fits into 4 bytes when namespace <= 255 and value <= 65535
                    size += write_u8(stream, 0x1)?;
                    size += write_u8(stream, self.namespace as u8)?;
                    size += write_u16(stream, *value as u16)?;
                } else {
                    // full node id
                    size += write_u8(stream, 0x2)?;
                    size += write_u16(stream, self.namespace)?;
                    size += write_u64(stream, *value as u64)?;
                }
            },
            Identifier::String(ref value) => {
                size += write_u8(stream, 0x3)?;
                size += write_u16(stream, self.namespace)?;
                size += value.encode(stream)?;
            },
            Identifier::Guid(ref value) => {
                size += write_u8(stream, 0x4)?;
                size += write_u16(stream, self.namespace)?;
                size += value.encode(stream)?;
            },
            Identifier::ByteString(ref value) => {
                size += write_u8(stream, 0x5)?;
                size += write_u16(stream, self.namespace)?;
                size += value.encode(stream)?;
            }
        }
        assert_eq!(size, self.byte_len());
        Ok(size)
    }

    fn decode(stream: &mut Read) -> Result<NodeId> {
        let identifier = read_u8(stream)?;
        let node_id = match identifier {
            0x0 => {
                let namespace = 0;
                let value = read_u8(stream)? as u64;
                NodeId::new_numeric(namespace, value)
            },
            0x1 => {
                let namespace = read_u8(stream)? as u16;
                let value = read_u16(stream)? as u64;
                NodeId::new_numeric(namespace, value)
            },
            0x2 => {
                let namespace = read_u16(stream)?;
                let value = read_u64(stream)?;
                NodeId::new_numeric(namespace, value)
            },
            0x3 => {
                let namespace = read_u16(stream)?;
                let value = UAString::decode(stream)?;
                NodeId::new_string(namespace, value)
            },
            0x4 => {
                let namespace = read_u16(stream)?;
                let value = Guid::decode(stream)?;
                NodeId::new_guid(namespace, value)
            },
            0x5 => {
                let namespace = read_u16(stream)?;
                let value = ByteString::decode(stream)?;
                NodeId::new_byte_string(namespace, value)
            }
            _ => {
                panic!("Unrecognized node id type");
            }
        };
        Ok(node_id)
    }
}

impl NodeId {
    /// Returns a null node id
    pub fn null() -> NodeId {
        NodeId::new_numeric(0, 0)
    }

    /// Makes a NodeId that holds an ObjectId
    pub fn from_object_id(id: ObjectId) -> NodeId {
        NodeId::new_numeric(0, id as UInt64)
    }

    /// Extracts an ObjectId from a node id, providing the node id holds an object id
    pub fn as_object_id(&self) -> std::result::Result<ObjectId, ()> {
        if self.namespace == 0 {
            match self.identifier {
                Identifier::Numeric(object_id) => ObjectId::from_u64(object_id),
                _ => Err(())
            }
        } else {
            Err(())
        }
    }

    /// Construct a numeric node id
    pub fn new_numeric(namespace: UInt16, value: UInt64) -> NodeId {
        NodeId { namespace: namespace, identifier: Identifier::Numeric(value), }
    }

    /// Construct a string node id
    pub fn new_string(namespace: UInt16, value: UAString) -> NodeId {
        NodeId { namespace: namespace, identifier: Identifier::String(value), }
    }

    /// Construct a guid node id
    pub fn new_guid(namespace: UInt16, value: Guid) -> NodeId {
        NodeId { namespace: namespace, identifier: Identifier::Guid(value), }
    }

    /// Construct a bytestring node id
    pub fn new_byte_string(namespace: UInt16, value: ByteString) -> NodeId {
        NodeId { namespace: namespace, identifier: Identifier::ByteString(value), }
    }

    /// Test if the node id is null, i.e. 0 namespace and 0 identifier
    pub fn is_null(&self) -> bool {
        match self.identifier {
            Identifier::Numeric(id) => { id == 0 && self.namespace == 0 },
            _ => false,
        }
    }

    /// Test if the node id is numeric
    pub fn is_numeric(&self) -> bool {
        match self.identifier {
            Identifier::Numeric(_) => true,
            _ => false,
        }
    }

    /// Test if the node id is a string
    pub fn is_string(&self) -> bool {
        match self.identifier {
            Identifier::String(_) => true,
            _ => false,
        }
    }

    /// Test if the node id is a guid
    pub fn is_guid(&self) -> bool {
        match self.identifier {
            Identifier::Guid(_) => true,
            _ => false,
        }
    }

    /// Test if the node id us a byte string
    pub fn is_byte_string(&self) -> bool {
        match self.identifier {
            Identifier::ByteString(_) => true,
            _ => false,
        }
    }
}

/// A NodeId that allows the namespace URI to be specified instead of an index.
/// Data type ID 18
#[derive(PartialEq, Debug, Clone)]
pub struct ExpandedNodeId {
    pub node_id: NodeId,
    pub namespace_uri: UAString,
    pub server_index: UInt32,
}

impl BinaryEncoder<ExpandedNodeId> for ExpandedNodeId {
    fn byte_len(&self) -> usize {
        unimplemented!();
    }

    fn encode(&self, stream: &mut Write) -> Result<usize> {
        unimplemented!();
    }

    fn decode(stream: &mut Read) -> Result<ExpandedNodeId> {
        unimplemented!();
    }
}