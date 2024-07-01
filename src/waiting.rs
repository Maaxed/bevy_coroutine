use std::time::Duration;

use bevy::ecs::system::{Adapt, AdapterSystem};
use bevy::prelude::*;

use crate::CoResult;

fn break_if(break_: bool) -> CoResult
{
	if break_
	{
		CoResult::break_()
	}
	else
	{
		CoResult::continue_()
	}
}

/// Generates a coroutine system that waits for the given duration then breaks
pub fn wait(duration: Duration) -> impl FnMut(Res<Time>) -> CoResult + Clone
{
	let mut timer = Timer::new(duration, TimerMode::Once);
	move |time: Res<Time>|
	{
		timer.tick(time.delta());
		break_if(timer.just_finished())
	}
}

/// Generates a coroutine system that waits for the given real time duration then breaks
pub fn wait_real_time(duration: Duration) -> impl FnMut(Res<Time<Real>>) -> CoResult + Clone
{
	let mut timer = Timer::new(duration, TimerMode::Once);
	move |time: Res<Time<Real>>|
	{
		timer.tick(time.delta());
		break_if(timer.just_finished())
	}
}

/// Generates a coroutine system that waits until the given system returns true then breaks
pub fn wait_until<M, S: IntoSystem<(), bool, M>>(condition: S) -> AdapterSystem<UntilMarker, S::System>
{
	let system = IntoSystem::into_system(condition);
	let name = format!("wait_until({})", system.name());
	AdapterSystem::new(UntilMarker, system, name.into())
}

/// Used with [`AdapterSystem`] to return `CoResult::break_()` when the system return true
#[doc(hidden)]
#[derive(Copy, Clone)]
pub struct UntilMarker;

impl<S> Adapt<S> for UntilMarker
where
	S: System<Out = bool>
{
	type In = S::In;
	type Out = CoResult;

	fn adapt(&mut self, input: Self::In, run_system: impl FnOnce(<S as System>::In) -> <S as System>::Out) -> Self::Out
	{
		break_if(run_system(input))
	}
}


/// Generates a coroutine system that waits while the given system returns true then breaks
pub fn wait_while<M, S: IntoSystem<(), bool, M>>(condition: S) -> AdapterSystem<WhileMarker, S::System>
{
	let system = IntoSystem::into_system(condition);
	let name = format!("wait_while({})", system.name());
	AdapterSystem::new(WhileMarker, system, name.into())
}

/// Used with [`AdapterSystem`] to return `CoResult::break_()` when the system return false
#[doc(hidden)]
#[derive(Copy, Clone)]
pub struct WhileMarker;

impl<S> Adapt<S> for WhileMarker
where
	S: System<Out = bool>
{
	type In = S::In;
	type Out = CoResult;

	fn adapt(&mut self, input: Self::In, run_system: impl FnOnce(<S as System>::In) -> <S as System>::Out) -> Self::Out
	{
		break_if(!run_system(input))
	}
}
