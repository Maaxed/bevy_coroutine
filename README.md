# Bevy Coroutine

A simple [Bevy](https://bevyengine.org/) library to run coroutines similar to [Unity's coroutines](https://docs.unity3d.com/Manual/Coroutines.html).  
Bevy Coroutine is very experimental and new versions may contain breaking changes !

## Usage

The main motivation behind Bevy Coroutine is to allows you to spread the execution of a system across several frames.
It also helps with managing and running systems in sequence over time.

```rust
use bevy::prelude::*;
use bevy_coroutine::prelude::*;

fn startup_system(mut commands: Commands) {
	// Launch the coroutine from a system
	commands.add(Coroutine::new(my_coroutine));
}

fn my_coroutine(

) -> CoResult {
	let mut res = co_break();
	// Print number from 0 to 3, printing a single number every second
	for i in 0..=3 {
		res.add_subroutines((
			wait(std::time::Duration::from_secs(1)),
			with_input(i, print_number),
		));
	}
	res
}

fn print_number(
	In(i): In<u32>,
) -> CoResult {
	println!("{i}");
	co_break()
}
```

Coroutines are systems. They can access any system parameter like ``Query`` and ``Res``.  
Coroutines don't use a 'yield' statement but instead return a ``CoResult``.
The ``CoResult`` indicates if the coroutine should 'break' and stops its execution or 'continue' and be executed again at the next update.
In addition, a coroutine can run other coroutines and wait for their completion by adding them as subroutines to the ``CoResult``.

The state of execution of your coroutine can be stored between frames using ``Local``s as parameters in the coroutine system.

```rust
use bevy::prelude::*;
use bevy_coroutine::prelude::*;

fn my_coroutine(
	mut i: Local<u32>,
) -> CoResult {
	if *i <= 3
	{
		println!("{}", *i);
		*i += 1;
		return co_continue(); // Rerun the system next frame
	}
	co_break()
}
```

## Performance

Each coroutine run in an exclusive system. They won't run in parallel with each other nor with other bevy systems.  
For CPU intensive tasks, consider using [bevy_tasks](https://docs.rs/bevy_tasks/latest/bevy_tasks/) or [bevy_defer](https://github.com/mintlu8/bevy_defer/).