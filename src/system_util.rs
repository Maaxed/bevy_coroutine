use bevy::prelude::*;
use bevy::ecs::system::{Adapt, AdapterSystem};


pub fn with_input<S, I, O, M>(input: I, system: S) -> AdapterSystem<InputAdapter<I>, S::System>
where
	S: IntoSystem<I, O, M>,
	I: Sync + Send + Clone,
{
	let system = IntoSystem::into_system(system);
	let name = format!("{} with input", system.name());
	AdapterSystem::new(InputAdapter(input), system, name.into())
}

pub struct InputAdapter<I>(I);

impl<S> Adapt<S> for InputAdapter<S::In>
where
	S: System,
	S::In: Sync + Send + Clone,
{
	type In = ();
	type Out = S::Out;

	fn adapt(&mut self, _input: (), run_system: impl FnOnce(S::In) -> S::Out) -> S::Out
	{
		run_system(self.0.clone())
	}
}