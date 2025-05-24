use bevy::prelude::*;
use rand::Rng;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<LoadingState>()
            .add_systems(Update, (
                update_loading_messages,
                update_loading_bar,
                cleanup_loading_screen,
            ))
            .add_systems(Startup, spawn_loading_screen);
    }
}

#[derive(Resource)]
pub struct LoadingState {
    pub progress: f32, // 0.0 to 1.0
    pub current_message: String,
    pub message_timer: Timer,
    pub bar_animation_time: f32,
    pub is_complete: bool,
    pub world_ready: bool, // True when world generation is done
    pub first_frame_rendered: bool, // True when first frame is actually rendered
}

impl Default for LoadingState {
    fn default() -> Self {
        Self {
            progress: 0.0,
            current_message: get_random_loading_message(),
            message_timer: Timer::from_seconds(0.8, TimerMode::Repeating),
            bar_animation_time: 0.0,
            is_complete: false,
            world_ready: false,
            first_frame_rendered: false,
        }
    }
}

#[derive(Component)]
pub struct LoadingScreen;

#[derive(Component)]
pub struct LoadingBar;

#[derive(Component)]
pub struct LoadingText;

#[derive(Component)]
pub struct LoadingMessage;

fn spawn_loading_screen(mut commands: Commands) {
    // Main loading screen container
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            background_color: Color::srgb(0.1, 0.1, 0.2).into(),
            ..default()
        },
        LoadingScreen,
    )).with_children(|parent| {
        // Title
        parent.spawn((
            TextBundle::from_section(
                "🦎 Creature Simulation 🌍",
                TextStyle {
                    font_size: 48.0,
                    color: Color::srgb(0.9, 0.9, 0.9),
                    ..default()
                },
            ).with_style(Style {
                margin: UiRect::all(Val::Px(20.0)),
                ..default()
            }),
        ));

        // Loading message
        parent.spawn((
            TextBundle::from_section(
                get_random_loading_message(),
                TextStyle {
                    font_size: 20.0,
                    color: Color::srgb(0.7, 0.8, 0.9),
                    ..default()
                },
            ).with_style(Style {
                margin: UiRect::all(Val::Px(10.0)),
                ..default()
            }),
            LoadingMessage,
        ));

        // Loading bar container
        parent.spawn(NodeBundle {
            style: Style {
                width: Val::Px(400.0),
                height: Val::Px(30.0),
                margin: UiRect::all(Val::Px(20.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            border_color: Color::srgb(0.5, 0.5, 0.5).into(),
            background_color: Color::srgb(0.2, 0.2, 0.3).into(),
            ..default()
        }).with_children(|parent| {
            // Loading bar fill
            parent.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(0.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    background_color: Color::srgb(0.2, 0.8, 0.4).into(),
                    ..default()
                },
                LoadingBar,
            ));
        });

        // Fun little creatures
        parent.spawn((
            TextBundle::from_section(
                "🐾 🦎 🌱 🏔️ 🌊 🐾",
                TextStyle {
                    font_size: 24.0,
                    color: Color::srgb(0.8, 0.7, 0.6),
                    ..default()
                },
            ).with_style(Style {
                margin: UiRect::all(Val::Px(20.0)),
                ..default()
            }),
        ));
    });
}

fn update_loading_messages(
    time: Res<Time>,
    mut loading_state: ResMut<LoadingState>,
    mut message_query: Query<&mut Text, With<LoadingMessage>>,
) {
    loading_state.message_timer.tick(time.delta());
    loading_state.bar_animation_time += time.delta_seconds();
    
    // Update progress for rendering phase
    if loading_state.world_ready && !loading_state.first_frame_rendered {
        loading_state.progress = (loading_state.progress + time.delta_seconds() * 0.5).min(0.99);
        
        // Update message for rendering phase
        if loading_state.message_timer.just_finished() {
            let rendering_messages = vec![
                "🎨 Painting the landscape...",
                "🖌️ Adding final details...",
                "✨ Sprinkling magic dust...",
                "🌟 Making everything sparkle...",
                "🎭 Setting the stage...",
                "📸 Adjusting the camera angle...",
                "🌈 Calibrating colors...",
                "🎪 Preparing the grand reveal...",
            ];
            let mut rng = rand::thread_rng();
            loading_state.current_message = rendering_messages[rng.gen_range(0..rendering_messages.len())].to_string();
        }
    } else if loading_state.message_timer.just_finished() && !loading_state.is_complete {
        loading_state.current_message = get_random_loading_message();
    }
    
    // Update text display
    for mut text in message_query.iter_mut() {
        text.sections[0].value = loading_state.current_message.clone();
    }
}

fn update_loading_bar(
    time: Res<Time>,
    loading_state: Res<LoadingState>,
    mut bar_query: Query<(&mut Style, &mut BackgroundColor), With<LoadingBar>>,
) {
    for (mut style, mut color) in bar_query.iter_mut() {
        // Smooth progress bar animation
        let target_width = loading_state.progress * 100.0;
        let current_width = match style.width {
            Val::Percent(w) => w,
            _ => 0.0,
        };
        
        let new_width = current_width + (target_width - current_width) * time.delta_seconds() * 3.0;
        style.width = Val::Percent(new_width.min(100.0));
        
        // Color animation based on progress
        let hue = loading_state.bar_animation_time * 0.5 + loading_state.progress * 120.0;
        let saturation = 0.8;
        let lightness = 0.5 + (loading_state.bar_animation_time * 2.0).sin() * 0.1;
        
        *color = Color::hsl(hue % 360.0, saturation, lightness).into();
    }
}

fn cleanup_loading_screen(
    mut commands: Commands,
    loading_state: Res<LoadingState>,
    loading_screen_query: Query<Entity, With<LoadingScreen>>,
) {
    // Only remove loading screen when everything is actually ready
    if loading_state.is_complete && loading_state.first_frame_rendered {
        for entity in loading_screen_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn get_random_loading_message() -> String {
    let messages = vec![
        "🌱 Planting magical trees...",
        "🏔️ Sculpting majestic mountains...",
        "🌊 Filling oceans with mysterious creatures...",
        "🦎 Teaching lizards how to dance...",
        "🌵 Convincing cacti to be social...",
        "🐸 Installing frog conversation software...",
        "🦋 Calibrating butterfly wingbeats...",
        "🍄 Growing mushrooms with attitude...",
        "🌪️ Brewing perfect weather storms...",
        "🦅 Training eagles in aerial acrobatics...",
        "🐛 Debugging the bugs (literally)...",
        "🌸 Painting flowers in ridiculous colors...",
        "🦀 Teaching crabs sideways philosophy...",
        "🐝 Installing bee-to-flower translation...",
        "🦉 Setting owl wisdom levels to maximum...",
        "🐙 Untangling octopus tentacles...",
        "🌙 Adjusting moon brightness settings...",
        "⭐ Counting stars (again, for accuracy)...",
        "🌈 Mixing rainbow paint buckets...",
        "🎨 Adding finishing touches to sunsets...",
        "🦊 Teaching foxes advanced cunning...",
        "🐺 Organizing wolf pack hierarchies...",
        "🦉 Installing night vision goggles...",
        "🌿 Whispering growth secrets to grass...",
        "🪨 Polishing rocks to perfection...",
        "💧 Quality testing every water drop...",
        "⚡ Supercharging the simulation engine...",
        "🚀 Activating parallel processing magic...",
        "⚙️ Fine-tuning world generation algorithms...",
        "🔥 Igniting multi-threaded chaos...",
        "💫 Sprinkling optimization fairy dust...",
        "🎯 Precision-crafting every biome...",
        "🏃‍♂️ Racing through world creation...",
        "🦎 Lizard management orientation complete!",
        "🎉 Almost ready for the chaos to begin!",
    ];
    
    let mut rng = rand::thread_rng();
    messages[rng.gen_range(0..messages.len())].to_string()
}

// Helper function to update loading progress from other systems
pub fn update_loading_progress(
    mut loading_state: ResMut<LoadingState>,
    time: Res<Time>,
    progress: f32,
    custom_message: Option<String>,
) {
    loading_state.progress = progress.clamp(0.0, 1.0);
    loading_state.bar_animation_time += time.delta_seconds();
    
    if let Some(message) = custom_message {
        loading_state.current_message = message;
    }
    
    if progress >= 1.0 {
        loading_state.is_complete = true;
        loading_state.current_message = "🎉 Welcome to your new world! 🎉".to_string();
    }
}