use std::ops::ControlFlow;

use bevy::ecs::system::{BoxedSystem, RegisteredSystemError, SystemId};
use bevy::ecs::world::Command;
use bevy::prelude::*;
use bevy::utils::all_tuples;


pub struct CoroutinePlugin;

impl Plugin for CoroutinePlugin
{
	fn build(&self, app: &mut App)
	{
		app
			.init_resource::<Coroutines>()
			.add_systems(Update, update_coroutines)
		;
	}
}

pub type BoxedCoroutine = BoxedSystem<(), CoResult>;
pub struct CoResult
{
	pub control_flow: ControlFlow<()>,
	pub subroutines: Vec<BoxedCoroutine>,
}

impl CoResult
{
	pub fn continue_() -> Self
	{
		CoResult
		{
			control_flow: ControlFlow::Continue(()),
			subroutines: Vec::new(),
		}
	}
	pub fn break_() -> Self
	{
		CoResult
		{
			control_flow: ControlFlow::Break(()),
			subroutines: Vec::new(),
		}
	}

	pub fn add_subroutines<M>(&mut self, subroutines: impl IntoCoroutines<M>)
	{
		self.add_boxed_subroutines(subroutines.into_coroutines())
	}

	pub fn add_boxed_subroutines(&mut self, subroutines: Vec<BoxedCoroutine>)
	{
		self.subroutines.extend(subroutines);
	}

	pub fn with_subroutines<M>(&self, subroutines: impl IntoCoroutines<M>) -> Self
	{
		self.with_boxed_subroutines(subroutines.into_coroutines())
	}

	pub fn with_boxed_subroutines(&self, subroutines: Vec<BoxedCoroutine>) -> Self
	{
		CoResult
		{
			control_flow: self.control_flow,
			subroutines,
		}
	}
}


struct CoroutineStack
{
	stack: Vec<SystemId<(), CoResult>>,
}

impl CoroutineStack
{
	fn resume(&mut self, world: &mut World) -> Result<(), RegisteredSystemError<(), CoResult>>
	{
		if let Some(co_system) = self.stack.last().copied()
		{
			let output = world.run_system(co_system)?;

			if output.control_flow.is_break()
			{
				world.remove_system(co_system)?;
				debug_assert_eq!(self.stack.pop().unwrap(), co_system);
			}

			// Last in the stack is always executed first, so 
			for subroutine in output.subroutines.into_iter().rev()
			{
				self.stack.push(world.register_boxed_system(subroutine));
			}
		}

		Ok(())
	}
}

#[derive(Resource, Default)]
struct Coroutines(Vec<CoroutineStack>);


pub struct Coroutine(Vec<BoxedCoroutine>);

impl Coroutine
{
	pub fn new<M>(coroutines: impl IntoCoroutines<M>) -> Self
	{
		Coroutine(coroutines.into_coroutines())
	}
}

impl Command for Coroutine
{
	fn apply(self, world: &mut World)
	{
		let Self(coroutines) = self;
		let stack = coroutines.into_iter().rev().map(|coroutine| world.register_boxed_system(coroutine)).collect();

		world.resource_mut::<Coroutines>().0.push(
			CoroutineStack
			{
				stack
			}
		);
	}
}

fn update_coroutines(
	world: &mut World
)
{
	world.resource_scope::<Coroutines, _>(|world, mut coroutines|
	{
		coroutines.0.retain_mut(|stack|
		{
			stack.resume(world).unwrap();
			!stack.stack.is_empty()
		});
	});
}

pub trait IntoCoroutines<Marker>: Sized
{
	fn collect_coroutines(self, coroutines: &mut Vec<BoxedCoroutine>);

	fn into_coroutines(self) -> Vec<BoxedCoroutine>
	{
		let mut coroutines = Vec::new();
		self.collect_coroutines(&mut coroutines);
		coroutines
	}
}

impl<S, M> IntoCoroutines<M> for S
where
	S: IntoSystem<(), CoResult, M> + 'static
{
	fn collect_coroutines(self, coroutines: &mut Vec<BoxedCoroutine>)
	{
		coroutines.push(Box::new(IntoSystem::into_system(self)));
	}
}


#[doc(hidden)]
pub struct CoroutineTupleMarker;

macro_rules! impl_coroutine_collection
{
    ($(($sys: ident, $sys_var: ident, $marker: ident)),*) =>
	{
        impl<$($sys, $marker),*> IntoCoroutines<(CoroutineTupleMarker, $($marker,)*)> for ($($sys,)*)
        where
            $($sys: IntoCoroutines<$marker>),*
        {
			fn collect_coroutines(self, coroutines: &mut Vec<BoxedCoroutine>)
			{
				let ($($sys_var,)*) = self;
				$(
					$sys_var.collect_coroutines(coroutines);
				)*
			}
        }
    }
}

all_tuples!(impl_coroutine_collection, 1, 20, S, s, M);
