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
		.add_systems(Startup, run_coroutines)
		.add_systems(Update, stop_app)
		.run();
}

fn run_coroutines(
	mut commands: Commands
)
{
	// Run two coroutines in parallel
	commands.queue(Coroutine::new(print_numbers));
	commands.queue(Coroutine::new(print_numbers));
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

	co_break()
}

fn stop_app(
	mut exit: MessageWriter<AppExit>,
    time: Res<Time>,
)
{
	// Exit after one second, hopefully all coroutines hae finished
	if time.elapsed_secs() > 1.0
	{
		exit.write(AppExit::Success);
	}
}
