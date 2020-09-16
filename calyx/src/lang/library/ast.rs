// Abstract Syntax Tree for library declarations in Futil
use crate::errors::{Error, Result};
use crate::lang::{
    ast,
    ast::{Id, Portdef},
};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Primitive {
    pub name: Id,
    pub params: Vec<Id>,
    pub signature: ParamSignature,
    pub attributes: HashMap<String, u64>,
    pub implementation: Vec<Implementation>,
}

#[derive(Clone, Debug)]
pub struct ParamSignature {
    pub inputs: Vec<ParamPortdef>,
    pub outputs: Vec<ParamPortdef>,
}

#[derive(Clone, Debug)]
pub struct ParamPortdef {
    pub name: Id,
    pub width: Width,
}

#[derive(Clone, Debug)]
pub enum Width {
    Const { value: u64 },
    Param { value: Id },
}

impl ParamSignature {
    /// Returns an iterator over the inputs of signature
    pub fn inputs(&self) -> std::slice::Iter<ParamPortdef> {
        self.inputs.iter()
    }

    /// Returns an iterator over the outputs of signature
    pub fn outputs(&self) -> std::slice::Iter<ParamPortdef> {
        self.outputs.iter()
    }

    /// Returns an `ast::Signature` if there are no params in self.
    pub fn to_signature(self) -> Result<ast::Signature> {
        let inputs: Vec<_> = self
            .inputs
            .into_iter()
            .map(|param| match param.width {
                Width::Const { value } => Ok(ast::Portdef {
                    name: param.name,
                    width: value,
                }),
                Width::Param { value } => unimplemented!(
                    "proper error for param being in a place it shouldn't be"
                ),
            })
            .collect::<Result<_>>()?;
        let outputs: Vec<_> = self
            .outputs
            .into_iter()
            .map(|param| match param.width {
                Width::Const { value } => Ok(ast::Portdef {
                    name: param.name,
                    width: value,
                }),
                Width::Param { value } => unimplemented!(
                    "proper error for param being in a place it shouldn't be"
                ),
            })
            .collect::<Result<_>>()?;
        Ok(ast::Signature { inputs, outputs })
    }
}

impl ParamPortdef {
    pub fn resolve(
        &self,
        prim: &Id,
        val_map: &HashMap<&Id, u64>,
    ) -> Result<Portdef> {
        match &self.width {
            Width::Const { value } => Ok(Portdef {
                name: self.name.clone(),
                width: *value,
            }),
            Width::Param { value } => match val_map.get(&value) {
                Some(width) => Ok(Portdef {
                    name: self.name.clone(),
                    width: *width,
                }),
                None => Err(Error::SignatureResolutionFailed(
                    prim.clone(),
                    value.clone(),
                )),
            },
        }
    }
}

// Parsing for providing particular backend implementations for primitive definitions

#[derive(Clone, Debug)]
pub enum Implementation {
    Verilog { data: Verilog },
}

#[derive(Clone, Debug)]
pub struct Verilog {
    pub code: String,
}
