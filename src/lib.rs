#![warn(missing_docs)]

//! TODO crate level doc


mod system_util;
mod waiting;

pub use system_util::*;
pub use waiting::*;

use std::ops::ControlFlow;

use bevy::ecs::system::{BoxedSystem, RegisteredSystemError, SystemId};
use bevy::ecs::world::Command;
use bevy::prelude::*;
use bevy::utils::all_tuples;

/// `use bevy_coroutine::prelude::*;` to import the plugin, essential types and utility functions.
pub mod prelude
{
	pub use crate::{
		Coroutine,
		CoResult,
		CoroutinePlugin,
		with_input,
		launch_coroutine,
		wait,
	};
}

/// Plugin to enable bevy_coroutine in your [`App`].
pub struct CoroutinePlugin;

impl Plugin for CoroutinePlugin
{
	fn build(&self, app: &mut App)
	{
		app
			.init_resource::<Coroutines>()
			.add_systems(Update, update_coroutines.in_set(CoroutineUpdateSystem))
		;
	}
}

/// The [`SystemSet`] for the system that updates coroutines.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, SystemSet)]
pub struct CoroutineUpdateSystem;

/// A type alias for a boxed system that can be used in a coroutine.
pub type BoxedCoroutine = BoxedSystem<(), CoResult>;

/// The result of a coroutine resumption.
pub struct CoResult
{
	/// Controls the execution of the coroutine.
	pub control_flow: ControlFlow<()>,
	/// The list of coroutines to be run sequentially before resuming this coroutine.
	pub subroutines: Vec<BoxedCoroutine>,
}

impl CoResult
{
	/// Rerun the current coroutine next update.
	pub fn continue_() -> Self
	{
		CoResult
		{
			control_flow: ControlFlow::Continue(()),
			subroutines: Vec::new(),
		}
	}
	
	/// Stops the execution of the current coroutine.
	pub fn break_() -> Self
	{
		CoResult
		{
			control_flow: ControlFlow::Break(()),
			subroutines: Vec::new(),
		}
	}

	/// Adds one or more coroutine systems to run sequentially before resuming the current coroutine.
	pub fn add_subroutines<M>(&mut self, subroutines: impl IntoCoroutines<M>)
	{
		self.add_boxed_subroutines(subroutines.into_coroutines())
	}

	/// Adds one or more coroutine systems to run sequentially before resuming the current coroutine.
	pub fn add_boxed_subroutines(&mut self, subroutines: Vec<BoxedCoroutine>)
	{
		self.subroutines.extend(subroutines);
	}

	/// Builds a new CoResult with the same `control_flow` but the given sub coroutine systems.
	pub fn with_subroutines<M>(&self, subroutines: impl IntoCoroutines<M>) -> Self
	{
		self.with_boxed_subroutines(subroutines.into_coroutines())
	}

	/// Builds a new CoResult with the same `control_flow` but the given sub coroutine systems.
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

			// Last in the stack is always executed first, so the reverse order
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

/// [`Command`] that launches a coroutine.
/// 
/// # Usage
/// 
/// ```
/// # use bevy::prelude::*;
/// # use bevy_coroutine::prelude::*;
/// fn launch_coroutine(
///     mut commands: Commands
/// ) {
///     commands.add(Coroutine::new(my_coroutine));
/// }
/// 
/// fn my_coroutine() -> CoResult {
///     CoResult::break_()
/// }
/// ```
pub struct Coroutine(Vec<BoxedCoroutine>);

impl Coroutine
{
	/// Makes a `Coroutine` [`Command`] that launches the given coroutine systems in sequence.
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
	// Take all registered coroutines
	let mut coroutines = std::mem::take(&mut world.resource_mut::<Coroutines>().0);

	// Execute the coroutines and remove the completed ones
	coroutines.retain_mut(|stack|
		{
			stack.resume(world).unwrap();
			!stack.stack.is_empty()
		});
	
	// Put the coroutines back in the registry, we need to append because coroutines can register new coroutines
	world.resource_mut::<Coroutines>().0.append(&mut coroutines);
}

/// Returns a system that launches the given coroutine when executed.
/// 
/// # Usage
/// 
/// ```
/// # use bevy::prelude::*;
/// # use bevy_coroutine::prelude::*;
/// # let mut app = App::new();
/// app.add_systems(Startup, launch_coroutine(my_coroutine));
/// 
/// fn my_coroutine() -> CoResult {
///     CoResult::break_()
/// }
/// ```
pub fn launch_coroutine<M, C: IntoCoroutines<M> + Clone>(coroutines: C) -> impl Fn(Commands) + Clone
{
	move |mut commands: Commands|
	{
		commands.add(Coroutine::new(coroutines.clone()));
	}
}

/// Conversion trait to turn something into a sequence of [`BoxedCoroutine`].
pub trait IntoCoroutines<Marker>: Sized
{
	/// Turn this value into a sequence of [`BoxedCoroutine`] and put them in the given Vec.
	fn collect_coroutines(self, coroutines: &mut Vec<BoxedCoroutine>);

	/// Turn this value into a sequence of [`BoxedCoroutine`] and returns it as a Vec.
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


#[cfg(test)]
mod test
{
	use crate::prelude::*;
    use bevy::prelude::*;

	#[test]
	fn coroutine_in_coroutine()
	{
		let mut app = App::new();
		app.add_plugins(CoroutinePlugin);

		#[derive(Debug, Default, Resource, PartialEq, Eq)]
		struct TestEvents(Vec<&'static str>);

		app.init_resource::<TestEvents>();

		app.add_systems(Startup, launch_coroutine(|mut events: ResMut<TestEvents>, mut commands: Commands|
		{
			events.0.push("COROUTINE_1");
			commands.add(Coroutine::new(|mut events: ResMut<TestEvents>|
			{
				events.0.push("COROUTINE_2");
				CoResult::break_()
			}));
			CoResult::break_()
		}));

		app.update();
		app.update();

		assert_eq!(app.world().resource::<TestEvents>(), &TestEvents(vec!["COROUTINE_1", "COROUTINE_2"]));
	}
}
