use crate::ecs::Ecs;
use std::error::Error;

type BoxedSystem = Box<dyn FnMut(&mut Ecs) -> SystemResult>;
pub type SystemResult = Result<(), Box<dyn Error>>;

pub struct SystemBundle {
    systems: Vec<BoxedSystem>,
}

impl SystemBundle {
    pub fn new() -> Self {
        SystemBundle { systems: vec![] }
    }

    pub fn add_system<T, S: IntoSystem<T>>(&mut self, system: S) {
        self.systems.push(system.into_system());
    }

    pub fn step(&mut self, ecs: &mut Ecs) -> Result<(), Box<dyn Error>> {
        for system in &mut self.systems {
            (system)(ecs)?;
        }

        Ok(())
    }
}

pub trait IntoSystem<T> {
    fn into_system(self) -> BoxedSystem;
}

impl<F> IntoSystem<F> for F
where
    F: 'static + FnMut(&mut Ecs) -> SystemResult,
{
    fn into_system(self) -> BoxedSystem {
        Box::new(self)
    }
}
impl<F> IntoSystem<(F,)> for F
where
    F: 'static + FnMut(&mut Ecs) -> (),
{
    fn into_system(mut self) -> BoxedSystem {
        Box::new(move |ecs: &mut Ecs| {
            (self)(ecs);
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::accessors::{R, W};
    use std::collections::HashSet;
    use std::fmt::{Display, Formatter};
    #[derive(Debug)]
    struct AtrociousFailure;
    impl Display for AtrociousFailure {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "ATROCIOUS ERROR")
        }
    }
    impl std::error::Error for AtrociousFailure {}

    #[test]
    fn failing_system() {
        let mut ecs = Ecs::new();
        let mut system = (|_: &mut Ecs| Err(Box::new(AtrociousFailure) as _)).into_system();
        let mut second_system = (|_: &mut Ecs| {}).into_system();

        let result = (system)(&mut ecs);
        let second_result = (second_system)(&mut ecs);

        assert!(result.is_err());
        assert!(second_result.is_ok());
    }

    #[test]
    fn system_into_system() {
        let _ = (|_: &mut Ecs| Ok(())).into_system();
    }

    #[test]
    fn system_bundle_add() {
        let mut system_bundle = SystemBundle::new();
        system_bundle.add_system(|_: &mut Ecs| Ok(()));
        assert_eq!(system_bundle.systems.len(), 1)
    }

    #[test]
    fn system_bundle_step() {
        #[derive(PartialEq, Debug, Eq, Hash, Copy, Clone)]
        struct Value(i32);
        struct OtherComponent;

        let mut ecs = Ecs::new();
        ecs.insert((Value(12),));
        ecs.insert((Value(18), OtherComponent));

        let mut system_bundle = SystemBundle::new();
        system_bundle.add_system(|ecs: &mut Ecs| {
            for (_, (mut v,)) in ecs.query::<(W<Value>,)>() {
                v.0 += 35;
            }
            Ok(())
        });
        system_bundle.add_system(|ecs: &mut Ecs| {
            for (_, (mut v,)) in ecs.query::<(W<Value>,)>() {
                v.0 -= 6;
            }
            Ok(())
        });

        let _ = system_bundle.step(&mut ecs);
        let query_result = ecs.query::<(R<Value>,)>();
        let result_set: HashSet<Value> = query_result.map(|result| *result.1 .0).collect();
        assert!(result_set.contains(&Value(41)));
        assert!(result_set.contains(&Value(47)));
    }
}
