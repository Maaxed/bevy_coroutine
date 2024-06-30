use bevy::prelude::*;
use bevy::app::ScheduleRunnerPlugin;
use bevy_coroutine::prelude::*;

fn main()
{
	App::new()
		.add_plugins((
			ScheduleRunnerPlugin::default(),
			CoroutinePlugin,
		))
		.add_systems(Startup, run_coroutines)
		.run();
}

fn run_coroutines(
	mut commands: Commands
)
{
	// Run two coroutine in parallel using the same system
	commands.add(Coroutine::new((print_numbers, stop_app)));
}

// Prints numbers from 0 to 3, printing a single number each update
fn print_numbers(
) -> CoResult
{
	let mut res = CoResult::break_();
	for i in 0..=3
	{
		res.add_subroutines(with_input(i, print_number));
	}

	res
}

fn print_number(
	In(i): In<u32>,
) -> CoResult
{
	println!("{i}");

	CoResult::break_()
}

fn stop_app(mut exit: EventWriter<AppExit>) -> CoResult
{
	exit.send(AppExit::Success);
	CoResult::break_()
}
