use std::collections::{HashMap, HashSet};
use std::ops::Index;
use std::thread::current;

use crate::{GraphicsResult, WGPURenderPass};

type RenderGraphResult<'a, T> = Result<T, RenderGraphError<'a>>;

#[derive(Debug, Eq, PartialEq, Hash)]
struct EdgeEnd<'a>(PassHandle, &'a str);

#[derive(Debug)]
enum RenderGraphError<'a> {
    EdgeSourceIsNotAPassOutput(EdgeEnd<'a>),
    EdgeDestinationIsNotAPassInput(EdgeEnd<'a>),
}

pub struct RenderGraph<'a> {
    pass_handles: Vec<PassHandle>,
    passes: Vec<RenderPass<'a>>,
    pass_dependencies: HashMap<PassHandle, HashSet<PassHandle>>,
    reversed_pass_dependencies: HashMap<PassHandle, HashSet<PassHandle>>,
    edges: HashMap<EdgeEnd<'a>, EdgeEnd<'a>>,
    reversed_edges: HashMap<EdgeEnd<'a>, EdgeEnd<'a>>,
}

impl<'a> RenderGraph<'a> {
    pub fn new() -> Self {
        Self {
            pass_handles: vec![],
            passes: vec![],
            pass_dependencies: HashMap::new(),
            reversed_pass_dependencies: HashMap::new(),
            edges: HashMap::new(),
            reversed_edges: HashMap::new(),
        }
    }

    pub fn add_pass(&mut self, pass_identifier: &'a str) -> PassBuilder<'a, '_> {
        PassBuilder {
            render_graph: self,
            identifier: pass_identifier,
            read_slots: HashSet::new(),
            write_slots: HashSet::new(),
        }
    }

    pub fn add_edge(
        &mut self,
        src_pass_handle: PassHandle,
        src_slot_identifier: &'a str,
        dst_pass_handle: PassHandle,
        dst_slot_identifier: &'a str,
    ) -> RenderGraphResult<()> {
        let src_pass = &self.passes[src_pass_handle];
        let dst_pass = &self.passes[dst_pass_handle];

        if !src_pass.write_slots.contains(src_slot_identifier) {
            return Err(RenderGraphError::EdgeSourceIsNotAPassOutput(EdgeEnd(
                src_pass_handle,
                src_slot_identifier,
            )));
        }

        if !dst_pass.read_slots.contains(dst_slot_identifier) {
            return Err(RenderGraphError::EdgeDestinationIsNotAPassInput(EdgeEnd(
                dst_pass_handle,
                dst_slot_identifier,
            )));
        }

        self.edges.insert(
            EdgeEnd(src_pass_handle, src_slot_identifier),
            EdgeEnd(dst_pass_handle, dst_slot_identifier),
        );

        self.reversed_edges.insert(
            EdgeEnd(dst_pass_handle, dst_slot_identifier),
            EdgeEnd(src_pass_handle, src_slot_identifier),
        );

        let dst_pass_dependencies = self.pass_dependencies.entry(dst_pass_handle).or_default();
        dst_pass_dependencies.insert(src_pass_handle);
        let reversed_src_pass_dependencies = self
            .reversed_pass_dependencies
            .entry(src_pass_handle)
            .or_default();
        reversed_src_pass_dependencies.insert(dst_pass_handle);

        Ok(())
    }

    pub fn generate_pass_ordering(&mut self) -> Vec<PassHandle> {
        let mut pass_ordering = self
            .pass_handles
            .iter()
            .filter(|handle| self.reversed_pass_dependencies[handle].is_empty())
            .cloned()
            .collect::<Vec<_>>();

        dbg!(&pass_ordering);

        let mut visited = HashSet::new();
        let mut pass_stack = pass_ordering.iter().cloned().collect::<Vec<_>>();
        while let Some(pass_handle) = pass_stack.pop() {
            visited.insert(pass_handle);
            for dependency in &self.pass_dependencies[&pass_handle] {
                if visited.contains(&dependency) {
                    continue;
                }

                pass_ordering.push(*dependency);
                pass_stack.push(*dependency);
            }
        }

        pass_ordering.reverse();
        pass_ordering
    }
}

pub struct PassBuilder<'a, 'g> {
    render_graph: &'g mut RenderGraph<'a>,
    identifier: &'a str,
    read_slots: HashSet<&'a str>,
    write_slots: HashSet<&'a str>,
}

impl<'a, 'g> PassBuilder<'a, 'g> {
    pub fn with_read_slot(mut self, slot_identifier: &'a str) -> Self {
        self.read_slots.insert(slot_identifier);
        self
    }

    pub fn with_write_slot(mut self, slot_identifier: &'a str) -> Self {
        self.write_slots.insert(slot_identifier);
        self
    }

    pub fn dispatch<F>(mut self, dispatch_fn: F) -> PassHandle
    where
        F: for<'p> Fn(WGPURenderPass<'p>) + 'static,
    {
        self.render_graph.passes.push(RenderPass {
            identifier: self.identifier,
            read_slots: self.read_slots,
            write_slots: self.write_slots,
            dispatch_fn: Box::new(dispatch_fn),
        });

        let handle = PassHandle(self.render_graph.passes.len() - 1);
        self.render_graph.pass_handles.push(handle);
        self.render_graph
            .pass_dependencies
            .insert(handle, HashSet::new());
        self.render_graph
            .reversed_pass_dependencies
            .insert(handle, HashSet::new());
        handle
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct PassHandle(usize);

impl From<usize> for PassHandle {
    fn from(id: usize) -> Self {
        PassHandle(id)
    }
}

struct RenderPass<'a> {
    identifier: &'a str,
    read_slots: HashSet<&'a str>,
    write_slots: HashSet<&'a str>,
    dispatch_fn: Box<dyn Fn(WGPURenderPass<'_>)>,
}

impl<'a> Index<PassHandle> for Vec<RenderPass<'a>> {
    type Output = RenderPass<'a>;

    fn index(&self, index: PassHandle) -> &Self::Output {
        &self[index.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_graph_add_pass() {
        let mut render_graph = RenderGraph::new();

        let pass_0 = render_graph
            .add_pass("pass0")
            .with_read_slot("read_slot_0")
            .with_write_slot("write_slot_0")
            .dispatch(|_| {});
        assert_eq!(pass_0, PassHandle(0));

        let pass_1 = render_graph
            .add_pass("pass1")
            .with_read_slot("read_slot_0")
            .with_write_slot("write_slot_0")
            .dispatch(|_| {});
        assert_eq!(pass_1, PassHandle(1));
    }

    #[test]
    fn render_graph_add_edge() {
        let mut render_graph = RenderGraph::new();

        let pass_0 = render_graph
            .add_pass("pass0")
            .with_read_slot("read_slot_0")
            .with_write_slot("write_slot_0")
            .dispatch(|_| {});

        let pass_1 = render_graph
            .add_pass("pass1")
            .with_read_slot("read_slot_0")
            .with_write_slot("write_slot_0")
            .dispatch(|_| {});

        let add_edge_result = render_graph.add_edge(pass_0, "write_slot_0", pass_1, "read_slot_0");
        assert!(add_edge_result.is_ok());
    }

    #[test]
    fn render_graph_generate_pass_ordering() {
        let mut render_graph = RenderGraph::new();

        let pass_0 = render_graph
            .add_pass("pass0")
            .with_read_slot("read_slot_0")
            .with_write_slot("write_slot_0")
            .dispatch(|_| {});

        let pass_1 = render_graph
            .add_pass("pass1")
            .with_read_slot("read_slot_0")
            .with_write_slot("write_slot_0")
            .dispatch(|_| {});

        render_graph
            .add_edge(pass_0, "write_slot_0", pass_1, "read_slot_0")
            .unwrap();
        let pass_ordering = render_graph.generate_pass_ordering();
        assert_eq!(pass_ordering.len(), 2);
        assert_eq!(pass_ordering[0], pass_0);
        assert_eq!(pass_ordering[1], pass_1);
    }

    #[test]
    fn render_graph_generate_pass_ordering_2() {
        let mut render_graph = RenderGraph::new();

        let pass_a = render_graph
            .add_pass("A")
            .with_read_slot("read_slot_0")
            .with_write_slot("write_slot_0")
            .with_write_slot("write_slot_1")
            .dispatch(|_| {});

        let pass_b = render_graph
            .add_pass("B")
            .with_read_slot("read_slot_0")
            .with_write_slot("write_slot_0")
            .dispatch(|_| {});

        let pass_c = render_graph
            .add_pass("C")
            .with_read_slot("read_slot_0")
            .with_read_slot("read_slot_1")
            .with_read_slot("read_slot_2")
            .with_write_slot("write_slot_0")
            .dispatch(|_| {});

        let pass_d = render_graph
            .add_pass("D")
            .with_read_slot("read_slot_0")
            .with_write_slot("write_slot_0")
            .dispatch(|_| {});

        render_graph
            .add_edge(pass_a, "write_slot_0", pass_c, "read_slot_0")
            .unwrap();
        render_graph
            .add_edge(pass_a, "write_slot_1", pass_c, "read_slot_1")
            .unwrap();
        render_graph
            .add_edge(pass_b, "write_slot_0", pass_c, "read_slot_2")
            .unwrap();
        render_graph
            .add_edge(pass_b, "write_slot_0", pass_d, "read_slot_0")
            .unwrap();

        let pass_ordering = render_graph.generate_pass_ordering();
        dbg!(pass_ordering);
    }

    #[test]
    fn render_graph_add_edge_invalid_edge() {
        let mut render_graph = RenderGraph::new();

        let pass_0 = render_graph
            .add_pass("pass0")
            .with_read_slot("read_slot_0")
            .with_write_slot("write_slot_0")
            .dispatch(|_| {});

        let pass_1 = render_graph
            .add_pass("pass1")
            .with_read_slot("read_slot_0")
            .with_write_slot("write_slot_0")
            .dispatch(|_| {});

        let add_edge_result = render_graph.add_edge(pass_0, "write_slot_0", pass_1, "write_slot_0");
        assert!(add_edge_result.is_err());
        assert!(matches!(
            Err(RenderGraphError::EdgeDestinationIsNotAPassInput(EdgeEnd(
                pass_1,
                "write_slot_0"
            ))) as RenderGraphResult<()>,
            add_edge_result
        ))
    }

    /*#[test]
    fn render_graph_new() {
        let render_graph = RenderGraph::new();
        let texture = render_graph.import_texture(&texture);
        let pass_1 = render_graph
            .add_pass("some_first_pass")
            .using_shader(&shader)
            .with_read_slot("in_texture")
            .with_write_slot("out_texture")
            .dispatch(|rpass| {});

        let pass_1_2 = render_graph
            .add_pass("some_other_first_pass")
            .using_shader(&shader)
            .with_read_slot("in_texture")
            .with_write_slot("out_texture")
            .dispatch(|rpass| {});

        let pass_2 = render_graph
            .add_pass("some_second_pass")
            .using_shader(&shader2)
            .read(&texture2)
            .with_read_slot("in_texture_1")
            .with_read_slot("in_texture_2")
            .with_write_slot("out_texture")
            .dispatch(|rpass| {});

        render_graph.add_edge(pass1, "out_texture", pass_2, "in_texture_1");
        render_graph.add_edge(pass1_2, "out_texture", pass_2, "in_texture_2");
        render_graph.set_input_for_read_slot(pass_1, "in_texture", texture);
        render_graph.set_input_for_read_slot(pass_1_2, "in_texture", texture);
    }*/
}
