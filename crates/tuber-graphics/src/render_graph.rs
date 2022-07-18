use std::collections::{HashMap, HashSet};
use std::ops::Index;
use std::thread::current;

use crate::resources::Handle;
use crate::shaders::Shader;
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
            shader_handle: None,
            identifier: pass_identifier,
            read_slots: HashSet::new(),
            write_slots: HashSet::new(),
        }
    }

    pub(crate) fn generate_pass(&mut self, render_pass: RenderPass<'a>) -> PassHandle {
        self.passes.push(render_pass);
        let handle = PassHandle(self.passes.len() - 1);
        self.pass_handles.push(handle);
        self.pass_dependencies.insert(handle, HashSet::new());
        self.reversed_pass_dependencies
            .insert(handle, HashSet::new());

        handle
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
    shader_handle: Option<Handle<Shader>>,
    read_slots: HashSet<&'a str>,
    write_slots: HashSet<&'a str>,
}

impl<'a, 'g> PassBuilder<'a, 'g> {
    pub fn using_shader(mut self, shader_handle: Handle<Shader>) -> Self {
        self.shader_handle = Some(shader_handle);
        self
    }

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
        self.render_graph.generate_pass(RenderPass {
            identifier: self.identifier,
            shader_handle: self.shader_handle,
            read_slots: self.read_slots,
            write_slots: self.write_slots,
            dispatch_fn: Box::new(dispatch_fn),
        })
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
    shader_handle: Option<Handle<Shader>>,
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
    use std::marker::PhantomData;

    use crate::Resources;

    use super::*;

    #[test]
    fn render_pass_builder_initial() {
        let mut render_graph = RenderGraph::new();

        let pass_builder = render_graph.add_pass("pass0");

        assert_eq!(pass_builder.identifier, "pass0");
        assert!(matches!(pass_builder.shader_handle, None));
        assert!(pass_builder.write_slots.is_empty());
        assert!(pass_builder.read_slots.is_empty());
    }

    #[test]
    fn render_pass_builder_with_read_slot() {
        let mut render_graph = RenderGraph::new();

        let pass_builder = render_graph.add_pass("pass0").with_read_slot("read_slot");

        assert_eq!(pass_builder.identifier, "pass0");
        assert!(matches!(pass_builder.shader_handle, None));
        assert!(pass_builder.write_slots.is_empty());
        assert!(!pass_builder.read_slots.is_empty());
        assert_eq!(pass_builder.read_slots.iter().next().unwrap(), &"read_slot");
    }

    #[test]
    fn render_pass_builder_with_write_slot() {
        let mut render_graph = RenderGraph::new();

        let pass_builder = render_graph.add_pass("pass0").with_write_slot("write_slot");

        assert_eq!(pass_builder.identifier, "pass0");
        assert!(matches!(pass_builder.shader_handle, None));
        assert!(!pass_builder.write_slots.is_empty());
        assert_eq!(
            pass_builder.write_slots.iter().next().unwrap(),
            &"write_slot"
        );
        assert!(pass_builder.read_slots.is_empty());
    }

    #[test]
    fn render_pass_builder_with_using_shader() {
        let mut render_graph = RenderGraph::new();
        let shader_handle = Handle::<Shader>::dummy();

        let pass_builder = render_graph.add_pass("pass0").using_shader(shader_handle);

        assert_eq!(pass_builder.identifier, "pass0");
        assert!(matches!(pass_builder.shader_handle, Some(shader_handle)));
        assert!(pass_builder.write_slots.is_empty());
        assert!(pass_builder.read_slots.is_empty());
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
        assert_eq!(pass_ordering[0], pass_a);
        assert_eq!(pass_ordering[1], pass_b);
        assert_eq!(pass_ordering[2], pass_d);
        assert_eq!(pass_ordering[3], pass_c);
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
}
