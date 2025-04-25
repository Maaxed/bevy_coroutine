use std::time::Duration;

use bevy::prelude::*;
use bevy::color::palettes::basic::*;
use bevy::input::mouse::MouseButtonInput;
use bevy::render::camera::ScalingMode;
use bevy::asset::embedded_asset;
use bevy_coroutine::prelude::*;

fn main()
{
	App::new()
		.add_plugins((
			DefaultPlugins.set(ImagePlugin::default_nearest()),
			CoroutinePlugin,
			build_app,
		))
		.add_systems(Startup, setup_player)
		.add_systems(Update, run_coroutines)
		.run();
}

fn build_app(
	app: &mut App,
)
{
    embedded_asset!(app, "examples", "slime.png");
}

#[derive(Component)]
struct Player;

fn setup_player(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
)
{
	commands.spawn((
		Camera2d,
		Projection::Orthographic(OrthographicProjection
		{
			scaling_mode: ScalingMode::WindowSize,
			scale: 1.0 / 4.0,
			.. OrthographicProjection::default_2d()
		}),
	));
	commands.spawn((
		Sprite
		{
			image: asset_server.load("embedded://hurt_animation/slime.png"),
			..default()
		},
		Player
	));
}

fn run_coroutines(
	mut mouse_clicked: EventReader<MouseButtonInput>,
	mut commands: Commands,
	player: Query<Entity, With<Player>>,
) -> Result
{
	for event in mouse_clicked.read()
	{
		if event.state.is_pressed()
		{
			let player = player.single()?;
			// Launch a coroutine
			commands.queue(Coroutine::new(with_input(player, play_hurt_animation)));
		}
	}

	Ok(())
}

fn play_hurt_animation(In(target): In<Entity>) -> CoResult {
	// Start sub coroutines in sequence
	co_break().with_subroutines((
		with_input((target, RED.into()), set_sprite_color),
		wait(Duration::from_secs_f32(0.1)),
		with_input((target, WHITE.into()), set_sprite_color),
	))
}

fn set_sprite_color(
	In((entity, color)): In<(Entity, Color)>,
	mut query: Query<&mut Sprite, With<Player>>,
) -> CoResult {
	let mut sprite = query.get_mut(entity).unwrap();
	sprite.color = color;
	co_break()
}
