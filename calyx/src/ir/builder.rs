//! IR Builder. Provides convience methods to build various parts of the internal
//! representation.
use crate::frontend::ast::Id;
use crate::ir::{self, RRC};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// An IR builder.
/// Uses internal references to the component to construct and validate
/// constructs when needed.
pub struct Builder<'a> {
    /// Component for which this builder is constructing.
    component: &'a mut ir::Component,
    /// Enable validation of components.
    /// Useful for debugging malformed AST errors.
    pub validate: bool,
}

impl<'a> Builder<'a> {
    /// Instantiate a new builder using for a component.
    pub fn from(component: &'a mut ir::Component, validate: bool) -> Self {
        Self {
            component,
            validate,
        }
    }

    /// Construct a new group using `name` and `attributes` and add it to the
    /// underlying component.
    /// Returns a reference to the group.
    pub fn add_group(
        &mut self,
        name: String,
        attributes: HashMap<String, u64>,
    ) -> RRC<ir::Group> {
        // Check if there is a group with the same name.
        if self.validate {
            if let Some(_) = self
                .component
                .groups
                .iter()
                .find(|&g| g.borrow().name == name)
            {
                panic!(
                    "Group with name `{}' already exists in component",
                    name.clone()
                )
            }
        }
        let group = Rc::new(RefCell::new(ir::Group {
            name: name.into(),
            attributes,
            holes: vec![],
            assignments: vec![],
        }));

        // Add default holes to the group.
        for (name, width) in vec![("go", 1), ("done", 1)] {
            let hole = Rc::new(RefCell::new(ir::Port {
                name: name.into(),
                width,
                direction: ir::Direction::Inout,
                parent: ir::PortParent::Group(Rc::downgrade(&group)),
            }));
            group.borrow_mut().holes.push(hole);
        }

        // Add the group to the component.
        self.component.groups.push(Rc::clone(&group));

        group
    }

    /// Return reference for a constant cell associated with the (val, width)
    /// pair, building and adding it to the component if needed..
    /// If the constant does not exist, it is added to the Context.
    pub fn add_constant(&mut self, val: u64, width: u64) -> RRC<ir::Cell> {
        let name = ir::Cell::constant_name(val, width);
        // If this constant has already been instantiated, return the relevant
        // cell.
        if let Some(cell) = self
            .component
            .cells
            .iter()
            .find(|&c| c.borrow().name == name)
        {
            return Rc::clone(cell);
        }

        // Construct this cell if it's not already present in the context.
        let cell = Self::cell_from_signature(
            name.clone(),
            ir::CellType::Constant { val, width },
            vec![],
            vec![("out".into(), width)],
        );

        // Add constant to the Component.
        self.component.cells.push(Rc::clone(&cell));

        cell
    }

    /// Consturcts a primitive cell of type `primitive` with the name
    /// prefix `name` and paramter bindings `param_bindings`.
    /// Adds this cell to the underlying component and returns a reference
    /// to the Cell.
    ///
    /// For example:
    /// ```
    /// // Construct a std_reg.
    /// builder.add_primitive("fsm", "std_reg", vec![32]);
    /// ```
    pub fn add_primitive<N: Into<Id>, P: Into<Id>>(
        &mut self,
        name: N,
        primitive: P,
        param_binding: &[u64],
    ) -> RRC<ir::Cell> {
        unimplemented!()
    }

    /// Construct an assignment.
    pub fn build_assignment(
        &self,
        dst: RRC<ir::Port>,
        src: RRC<ir::Port>,
        guard: Option<ir::Guard>,
    ) -> ir::Assignment {
        // Valid the ports if required.
        if self.validate {
            self.is_port_well_formed(&dst.borrow());
            self.is_port_well_formed(&src.borrow());
            if let Some(g) = &guard {
                g.all_ports()
                    .into_iter()
                    .for_each(|p| self.is_port_well_formed(&p.borrow()))
            }
        }
        // Validate: Check to see if the cell/group associated with the
        // port is in the component.
        ir::Assignment { dst, src, guard }
    }

    ///////////////////// Internal function ////////////////////
    /// VALIDATE: Check if the component contains the cell/group associated
    /// with the port exists in the Component.
    /// Validate methods panic! in order to generate a stacktrace to the
    /// offending code.
    fn is_port_well_formed(&self, port: &ir::Port) -> () {
        match &port.parent {
            ir::PortParent::Cell(cell_wref) => {
                let cell_ref = cell_wref.upgrade().expect("Weak reference to port's parent cell points to nothing. This usually means that the Component did not retain a pointer to the Cell.");

                let cell_name = &cell_ref.borrow().name;
                self.component.find_cell(cell_name).expect("Port's parent cell not present in the component. Add the cell to the component before using the Port.");
            }
            ir::PortParent::Group(group_wref) => {
                let group_ref = group_wref.upgrade().expect("Weak reference to hole's parent group points to nothing. This usually means that the Component did not retain a pointer to the Group.");

                let group_name = &group_ref.borrow().name;
                self.component.find_group(group_name).expect("Hole's parent cell not present in the component. Add the group to the component before using the Hole.");
            }
        };
    }
    /// Construct a cell from input/output signature.
    /// Input and output port definition in the form (name, width).
    pub(super) fn cell_from_signature(
        name: Id,
        typ: ir::CellType,
        inputs: Vec<(Id, u64)>,
        outputs: Vec<(Id, u64)>,
    ) -> RRC<ir::Cell> {
        let cell = Rc::new(RefCell::new(ir::Cell {
            name,
            ports: vec![],
            prototype: typ,
        }));
        // Construct ports
        for (name, width) in inputs {
            let port = Rc::new(RefCell::new(ir::Port {
                name,
                width,
                direction: ir::Direction::Input,
                parent: ir::PortParent::Cell(Rc::downgrade(&cell)),
            }));
            cell.borrow_mut().ports.push(port);
        }
        for (name, width) in outputs {
            let port = Rc::new(RefCell::new(ir::Port {
                name,
                width,
                direction: ir::Direction::Output,
                parent: ir::PortParent::Cell(Rc::downgrade(&cell)),
            }));
            cell.borrow_mut().ports.push(port);
        }
        cell
    }
}