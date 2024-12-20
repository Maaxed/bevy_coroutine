# Bevy Coroutine

[![Docs](https://docs.rs/bevy_coroutine/badge.svg)](https://docs.rs/bevy_coroutine/latest/bevy_coroutine/)
[![Crates.io](https://img.shields.io/crates/v/bevy_coroutine.svg)](https://crates.io/crates/bevy_coroutine)
[![Downloads](https://img.shields.io/crates/d/bevy_coroutine.svg)](https://crates.io/crates/bevy_coroutine)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/Maaxed/bevy_coroutine/blob/master/LICENSE)

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
	commands.queue(Coroutine::new(my_coroutine));
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

## Versions

| bevy        | bevy_coroutine |
|-------------|----------------|
| 0.15        | 0.2.0          |
| 0.14        | 0.1.3          |
| 0.14        | 0.1.2          |
| 0.14        | 0.1.1          |
| 0.14.0-rc.4 | 0.1.0          |
