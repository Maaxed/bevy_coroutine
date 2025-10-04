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
	// Run 3 coroutines in sequence using the same system
	commands.queue(Coroutine::new((print_numbers, print_numbers, stop_app)));
}

// Prints numbers from 0 to 3, printing a single number each update
fn print_numbers(
	mut i: Local<u32>,
) -> CoResult
{
	if *i <= 3
	{
		println!("{}", *i);
		*i += 1;
		return co_continue();
	}
	println!("STOP");

	co_break()
}

fn stop_app(mut exit: MessageWriter<AppExit>) -> CoResult
{
	exit.write(AppExit::Success);
	co_break()
}
