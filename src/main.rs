use bevy::{
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
    sprite::MaterialMesh2dBundle,
};

const TIME_STEP: f32 = 1. / 60.;

const HEIGHT: f32 = 600.;
const WIDTH: f32 = 800.;

const PADDLE_SPEED: f32 = 400.;
const PADDLE_SIZE: Vec3 = Vec3::new(20., 100., 0.);
const UP_PADDLE_BOUND: f32 = HEIGHT / 2. - PADDLE_SIZE.y / 2.;
const DOWN_PADDLE_BOUND: f32 = -HEIGHT / 2. + PADDLE_SIZE.y / 2.;

const BALL_STARTING_POSITION: Vec3 = Vec3::new(0., 0., 1.);
const BALL_SIZE: Vec3 = Vec3::new(30., 30., 0.);

const BALL_SPEED: f32 = 500.;
const INITIAL_BALL_DIRECTION: Vec2 = Vec2::new(0.5, 0.5);

const WALL_WIDTH: f32 = 10.;

const SCOREBOARD_TEXT_PADDING: f32 = 25.;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: String::from("Pong"),
                resizable: false,
                resolution: (WIDTH, HEIGHT).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(Game {
            p1: 0,
            p2: 0,
            paused: true,
        })
        .add_event::<CollisionEvent>()
        .insert_resource(ClearColor(Color::rgb(0., 0., 0.)))
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate,
            (
                check_for_collisions,
                move_ball.before(check_for_collisions),
                move_p1.before(check_for_collisions).after(move_ball),
                move_p2.before(check_for_collisions).after(move_ball),
                ball_collision.after(check_for_collisions),
            ),
        )
        .insert_resource(Time::<Fixed>::from_seconds(TIME_STEP.into()))
        .add_systems(Update, (update_scoreboard, bevy::window::close_on_esc))
        .run();
}

#[derive(Component)]
struct Collider;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct P1;

#[derive(Component)]
struct P2;

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct WallR;

#[derive(Component)]
struct WallL;

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Resource)]
struct Game {
    p1: u32,
    p2: u32,
    paused: bool,
}

#[derive(Default, Event)]
struct CollisionEvent;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2dBundle::default());

    // P1
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::WHITE,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(-(WIDTH / 2.) + PADDLE_SIZE.x, 0., 0.),
                scale: PADDLE_SIZE,
                ..default()
            },
            ..default()
        },
        P1,
        Player,
        Collider,
    ));

    // P2
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::WHITE,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new((WIDTH / 2.) - PADDLE_SIZE.x, 0., 0.),
                scale: PADDLE_SIZE,
                ..default()
            },
            ..default()
        },
        P2,
        Player,
        Collider,
    ));

    // Left wall
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(-(WIDTH / 2.) - WALL_WIDTH, 0., 0.),
                scale: Vec3::new(WALL_WIDTH, HEIGHT, 0.),
                ..default()
            },
            ..default()
        },
        Collider,
        WallL,
    ));

    // Right wall
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new((WIDTH / 2.) + WALL_WIDTH, 0., 0.),
                scale: Vec3::new(WALL_WIDTH, HEIGHT, 0.),
                ..default()
            },
            ..default()
        },
        Collider,
        WallR,
    ));

    // Top wall
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0., (HEIGHT / 2.) + WALL_WIDTH, 0.),
                scale: Vec3::new(WIDTH, WALL_WIDTH, 0.),
                ..default()
            },
            ..default()
        },
        Collider,
    ));

    // Bottom wall
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0., -(HEIGHT / 2.) - WALL_WIDTH, 0.),
                scale: Vec3::new(WIDTH, WALL_WIDTH, 0.),
                ..default()
            },
            ..default()
        },
        Collider,
    ));

    // Ball
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::default().into()).into(),
            material: materials.add(ColorMaterial::from(Color::WHITE)),
            transform: Transform::from_translation(BALL_STARTING_POSITION).with_scale(BALL_SIZE),
            ..default()
        },
        Ball,
        Velocity(INITIAL_BALL_DIRECTION.normalize() * BALL_SPEED),
    ));

    // Score
    commands.spawn(Text2dBundle {
        text: Text::from_section(
            "0 - 0",
            TextStyle {
                font: asset_server.load("fonts/PixeloidSansBold.ttf"),
                font_size: 40.,
                color: Color::WHITE,
            },
        ),
        transform: Transform::from_translation(Vec3::new(
            0.,
            HEIGHT / 2. - SCOREBOARD_TEXT_PADDING,
            0.,
        )),
        ..default()
    });
}

fn move_p1(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<P1>>,
    mut game: ResMut<Game>,
) {
    let mut paddle_transform = query.single_mut();
    let mut direction = 0.;
    if keyboard_input.pressed(KeyCode::W) {
        game.paused = false;
        direction += 1.;
    }

    if keyboard_input.pressed(KeyCode::S) {
        game.paused = false;
        direction -= 1.;
    }

    if !game.paused {
        paddle_transform.translation.y = (paddle_transform.translation.y
            + direction * TIME_STEP * PADDLE_SPEED)
            .clamp(DOWN_PADDLE_BOUND, UP_PADDLE_BOUND);
    }
}

fn move_p2(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<P2>>,
    mut game: ResMut<Game>,
) {
    let mut paddle_transform = query.single_mut();
    let mut direction = 0.;
    if keyboard_input.pressed(KeyCode::Up) {
        game.paused = false;
        direction += 1.;
    }

    if keyboard_input.pressed(KeyCode::Down) {
        game.paused = false;
        direction -= 1.;
    }
    if !game.paused {
        paddle_transform.translation.y = (paddle_transform.translation.y
            + direction * TIME_STEP * PADDLE_SPEED)
            .clamp(DOWN_PADDLE_BOUND, UP_PADDLE_BOUND);
    }
}

fn move_ball(mut query: Query<(&mut Transform, &Velocity), With<Ball>>, game: Res<Game>) {
    if !game.paused {
        let (mut transform, velocity) = query.single_mut();
        transform.translation.x += velocity.x * TIME_STEP;
        transform.translation.y += velocity.y * TIME_STEP;
    }
}

fn check_for_collisions(
    mut ball_query: Query<(&mut Velocity, &Transform), With<Ball>>,
    collider_query: Query<(&Transform, Option<&WallL>, Option<&WallR>), With<Collider>>,
    mut game: ResMut<Game>,
    mut collision_events: EventWriter<CollisionEvent>,
) {
    let (mut ball_velocity, ball_transform) = ball_query.single_mut();
    let ball_size = ball_transform.scale.truncate();

    for (transform, maybe_left, maybe_right) in &collider_query {
        let collision = collide(
            ball_transform.translation,
            ball_size,
            transform.translation,
            transform.scale.truncate(),
        );
        if let Some(collision) = collision {
            if maybe_left.is_some() {
                game.p2 += 1;
                collision_events.send_default();
            }

            if maybe_right.is_some() {
                game.p1 += 1;
                collision_events.send_default();
            }

            let mut reflect_x = false;
            let mut reflect_y = false;

            match collision {
                Collision::Left => reflect_x = ball_velocity.x > 0.,
                Collision::Right => reflect_x = ball_velocity.x < 0.,
                Collision::Top => reflect_y = ball_velocity.y < 0.,
                Collision::Bottom => reflect_y = ball_velocity.y > 0.,
                Collision::Inside => { /* do nothing */ }
            }

            if reflect_x {
                ball_velocity.x = -ball_velocity.x;
            }

            if reflect_y {
                ball_velocity.y = -ball_velocity.y;
            }
        }
    }
}

fn update_scoreboard(game: Res<Game>, mut query: Query<&mut Text>) {
    let mut text = query.single_mut();
    text.sections[0].value = format!("{} - {}", game.p1, game.p2);
}

fn ball_collision(
    mut collision_events: EventReader<CollisionEvent>,
    mut game: ResMut<Game>,
    mut query: Query<(&mut Transform, &Velocity), With<Ball>>,
) {
    if !collision_events.is_empty() {
        collision_events.clear();
        let (mut transform, _velocity) = query.single_mut();
        game.paused = true;
        transform.translation = BALL_STARTING_POSITION;
    }
}
