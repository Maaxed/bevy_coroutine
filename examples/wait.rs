use std::time::Duration;

use bevy::prelude::*;
use bevy::app::ScheduleRunnerPlugin;
use bevy::time::TimePlugin;
use bevy_coroutine::prelude::*;

fn main()
{
	App::new()
		.add_plugins((
			ScheduleRunnerPlugin::default(),
			TimePlugin,
			CoroutinePlugin,
		))
		.add_systems(Startup, launch_coroutine((
			print_numbers,
			stop_app
		)))
		.run();
}

// Prints numbers from 0 to 3, printing a single number every second
fn print_numbers(
) -> CoResult
{
	let mut res = co_break();
	for i in 0..=3
	{
		res.add_subroutines((
			wait(Duration::from_secs(1)),
			move || {
				println!("{}", i);
				co_break()
			}
		));
	}

	res
}

fn stop_app(mut exit: EventWriter<AppExit>) -> CoResult
{
	exit.send(AppExit::Success);
	co_break()
}
